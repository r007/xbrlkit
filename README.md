# xbrlkit

[![Crates.io](https://img.shields.io/crates/v/xbrlkit.svg)](https://crates.io/crates/xbrlkit)
[![Documentation](https://docs.rs/xbrlkit/badge.svg)](https://docs.rs/xbrlkit)
[![CI](https://github.com/r007/xbrlkit/actions/workflows/ci.yml/badge.svg)](https://github.com/r007/xbrlkit/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Read SEC XBRL filings into typed Rust structs.

`xbrlkit` reads the tagged data of a 10-K, 10-Q or 8-K — inline XBRL in the
filing's HTML, or a traditional XML instance — and binds it to plain structs
with `#[derive(FromXbrl)]`.

```rust
use xbrlkit::{Document, FromXbrl};

#[derive(Debug, Default, FromXbrl)]
struct Summary {
    #[xbrl(concept = "dei:EntityRegistrantName")]
    name: Option<String>,

    #[xbrl(concept = "us-gaap:Assets")]
    assets: Option<f64>,

    // Filers pick among synonyms. The first concept the filing reports wins.
    #[xbrl(
        concept = "us-gaap:RevenueFromContractWithCustomerExcludingAssessedTax",
        alias = "us-gaap:Revenues"
    )]
    revenue: Option<f64>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Apple's 10-K for fiscal 2025, as EDGAR serves it.
    let html = std::fs::read_to_string("aapl-20250927.htm")?;

    let doc = Document::parse(&html)?;
    let summary: Summary = doc.extract()?;

    println!("{summary:#?}");
    // Summary {
    //     name: Some("Apple Inc."),
    //     assets: Some(359241000000.0),
    //     revenue: Some(416161000000.0),
    // }
    Ok(())
}
```

## Install

```toml
[dependencies]
xbrlkit = "0.1"
```

Rust 1.85 or later.

## Why

A filing's numbers are already structured: every figure on the face of the
statements, and most of the ones in the notes, is tagged with a concept, a
period, a unit and the dimension members it applies to. Reading them should
not take an LLM or a table scraper. In practice it takes more than an XML
parser, for two reasons this crate exists to handle.

**Inline XBRL is HTML, and the tag holds what the reader sees, not the
value.** A loss is printed as `(1,234)` in a column headed "in thousands", with
the parentheses outside the tag; the value is `-1234000`. A date is
`September 30, 2025`, a check box is `☒`, a term is `five years`. Values run on
across page breaks through `ix:continuation`, sit inside `<b>` and `<span>`,
and nest inside the text block of the note that discusses them. The parser
applies the `format` transformation, `scale` and `sign` of each fact and gives
you `-1234000`, `2025-09-30`, `true` and `P5Y`.

**A filing reports each concept many times.** `us-gaap:Assets` appears for
this year end and last; revenue for the quarter, the year to date and both of
last year's; shares outstanding once per class. A field needs one of them, and
"the last one in the document" is the prior year more often than not. Each
field takes the fact for the period the filing reports on, and a struct can
ask for one copy of itself per period instead.

## Usage

### Ready-made statements

The cover page and the three primary statements are already declared.

```rust
use xbrlkit::Document;
use xbrlkit::taxonomies::{dei::DeiInfo, us_gaap::Financials};

let doc = Document::parse(&html)?;

let dei: DeiInfo = doc.extract()?;
println!("{:?} filed a {:?}", dei.entity.entity_registrant_name, dei.document.document_type);

let financials: Financials = doc.extract()?;
println!("total assets: {:?}", financials.balance_sheet.assets);

// One statement per period the filing reports. In a 10-Q that is the quarter
// and the year to date, each beside last year's.
for statement in &financials.income_statements {
    println!(
        "{:?} to {:?}: revenue {:?}, net income {:?}",
        statement.period_start, statement.period_end, statement.revenue, statement.net_income
    );
}
```

```text
Some("2026-04-01") to Some("2026-06-30"): revenue Some(28236000000.0), net income Some(1114000000.0)
Some("2026-01-01") to Some("2026-06-30"): revenue Some(50623000000.0), net income Some(1591000000.0)
Some("2025-04-01") to Some("2025-06-30"): revenue Some(22496000000.0), net income Some(1172000000.0)
Some("2025-01-01") to Some("2025-06-30"): revenue Some(41831000000.0), net income Some(1581000000.0)
```

`us_gaap::Financials` holds the line items most commercial filers report. It
is not the whole taxonomy and is not meant to be: when a field you need is
missing, declare a struct of your own.

### Your own struct

```rust
use serde::Serialize;
use xbrlkit::{Fact, FromXbrl};

#[derive(Default, Serialize, FromXbrl)]
#[xbrl(instant)]
struct BalanceSheet {
    /// The date the figures are as of.
    #[xbrl(period_end)]
    as_of: Option<String>,

    #[xbrl(concept = "us-gaap:Assets")]
    assets: Option<f64>,

    #[xbrl(concept = "us-gaap:Liabilities")]
    liabilities: Option<f64>,
}

#[derive(Default, Serialize, FromXbrl)]
struct Report {
    /// Field by field, the best the filing offers for its reporting period.
    #[xbrl(nested)]
    latest: BalanceSheet,

    /// One balance sheet per date the filing reports, latest first.
    #[xbrl(each_period)]
    balance_sheets: Vec<BalanceSheet>,

    /// With its period, unit and precision.
    #[xbrl(concept = "us-gaap:NetIncomeLoss")]
    net_income: Option<Fact<f64>>,

    /// Every period and every dimension member: revenue by product line and
    /// by region, this year and last.
    #[xbrl(concept = "us-gaap:RevenueFromContractWithCustomerExcludingAssessedTax")]
    revenue_breakdown: Vec<Fact<f64>>,
}

let report: Report = doc.extract()?;
```

The type of a field decides how much of the filing it sees:

| A field of type                 | Holds                                                  |
| ------------------------------- | ------------------------------------------------------ |
| `Option<T>`                     | the value of the one best fact                         |
| `Option<Fact<T>>`               | that fact with its period, unit, precision, dimensions |
| `Vec<Fact<T>>`                  | every period and dimension member the concept has      |
| a struct, `#[xbrl(nested)]`     | more fields, read the same way                         |
| `Vec` of one, `#[xbrl(each_period)]` | that struct once per period the filing reports    |

where `T` is `f64`, `i64`, `i32`, `u64`, `u32`, `bool` or `String`.

### Which fact a field gets

Read on its own, a struct takes each field from the fact that best matches the
period the filing reports on — in a 10-Q, the year to date — falling back to a
comparative or a dimensional breakdown when that is all there is. Its fields
are chosen independently and can come from different periods.

Read through `each_period`, every field comes from one period's consolidated
figures, and what the filing does not report for that period is `None`. Use it
when the period matters.

Where a filing prints one figure twice, exactly on the statement and rounded
in a note, the exact one is the value.

### One concept, no struct

```rust
let assets: Option<f64> = doc.get(&["us-gaap:Assets"]);
let shares_by_class: Vec<Fact<f64>> = doc.get(&["dei:EntityCommonStockSharesOutstanding"]);
```

### Everything the filing tags

```rust
for fact in doc.facts() {
    println!("{} = {:?}", fact.full_name, fact.value.as_str());
}
println!("{} contexts, {} units", doc.contexts().len(), doc.units().len());
```

### A value that does not convert

Filers tag `N/A` under numeric concepts. `extract` fails on the first value
that does not fit its field; `extract_lenient` leaves that field empty and
hands back what it skipped, so one bad tag does not cost a filing its balance
sheet.

```rust
let (financials, problems) = doc.extract_lenient::<Financials>();
for problem in &problems {
    eprintln!("skipped: {problem}");
}
```

### With edgarkit

[edgarkit](https://github.com/r007/edgarkit) fetches filings from EDGAR;
`xbrlkit` reads them.

```rust
use edgarkit::{Edgar, FilingOperations};

let edgar = Edgar::new("Your Name you@example.com")?;
let html = edgar.get_latest_filing_content("320193", &["10-K"]).await?;

let doc = xbrlkit::Document::from_ixbrl(&html)?;
let financials: Financials = doc.extract()?;
```

[`examples/edgar`](examples/edgar) is this as a runnable project.

## Works with serde

The concept a field reads from lives in `#[xbrl(..)]`, not in a serde rename,
so a struct's serde names are its Rust names. Derive `Serialize` beside
`FromXbrl` and what reaches JSON, Arrow or Parquet is `assets`, not
`us-gaap:Assets`:

```json
{
  "as_of": "2025-09-27",
  "assets": 359241000000.0,
  "liabilities": 285508000000.0
}
```

Every field has a fixed type, so tools that derive a schema from the struct
(`serde_arrow`, `schemars`) need nothing from the data. The ready-made structs
all implement `Serialize` and `Deserialize`.

## Tested on real filings

The crate was extracted from a production pipeline that follows US-listed
SPACs, where it has been reading 10-K, 10-Q and 8-K filings since 2025. Most
of what the parser steps over — unclosed tags, values wrapped in formatting,
facts nested in text blocks, contexts declared in the default namespace — is
there because a filing did it.

The test suite also holds the inline parser to the SEC's own reading. EDGAR
extracts an XML instance from every inline filing it accepts, with each
`format`, `scale` and `sign` already applied. Across the reports of eleven
large filers — Apple, Microsoft, JPMorgan, Exxon, Berkshire Hathaway, Tesla,
Nvidia, Walmart, Coca-Cola, Spotify's 20-F and an 8-K — `xbrlkit` read the
same number of facts as EDGAR from every inline document, and 22,835 of the
22,836 numeric and formatted values were equal. The one that differs is a
country name under `ixt-sec:edgarprovcountryen`, which this crate normalises
without mapping it to EDGAR's two-character code. Two of those filings are in
[`tests/fixtures`](tests/fixtures) and
[`tests/sec_conformance.rs`](tests/sec_conformance.rs) repeats the comparison
on every run.

## Performance

One pass over the text, no DOM. Measured with `cargo bench` on an Intel Core
i5-14400F, single-threaded, median of repeated runs:

| Document                          | Size    | Facts | Parse   | Throughput | Index   | Cover page + statements |
| --------------------------------- | ------- | ----- | ------- | ---------- | ------- | ----------------------- |
| Apple 10-K, inline XBRL           | 1.52 MB | 1,131 | 3.72 ms | 409 MB/s   | 0.18 ms | 0.36 ms                 |
| Apple 10-K, XML instance          | 1.42 MB | 1,131 | 1.98 ms | 715 MB/s   | 0.19 ms | 0.34 ms                 |
| Tesla 10-Q, inline XBRL           | 1.57 MB | 1,221 | 4.25 ms | 370 MB/s   | 0.19 ms | 0.48 ms                 |
| Tesla 10-Q, XML instance          | 1.47 MB | 1,221 | 2.61 ms | 565 MB/s   | 0.18 ms | 0.47 ms                 |
| A SPAC's 10-Q, inline XBRL        | 1.02 MB | 575   | 2.68 ms | 381 MB/s   | 0.11 ms | 0.41 ms                 |

"Parse" is text to facts; "index" builds the lookups a struct is read
through; the last column reads `DeiInfo` and `Financials`, every period
included. End to end, Apple's 10-K goes from HTML to typed statements in under
five milliseconds. JPMorgan's 13 MB 10-K, with 8,442 facts, parses in 34 ms.

Run it on your own filings:

```bash
cargo bench -- path/to/filing.htm
```

## Other Rust XBRL crates

Compared in October 2026.

|                                   | xbrlkit 0.1 | [crabrl] 0.1.0                      | [xbrl-rs] 0.3.0         |
| --------------------------------- | ----------- | ----------------------------------- | ----------------------- |
| Inline XBRL (filings since 2019)  | yes         | planned                             | no                      |
| XML instances                     | yes         | yes                                 | yes                     |
| Inline transformations            | yes         | —                                   | —                       |
| Typed structs, derive             | yes         | no                                  | no                      |
| Taxonomy schemas and linkbases    | no          | calculation, presentation, labels   | yes, with DTS discovery |
| Validation                        | no          | yes, with SEC rules                 | yes, against a taxonomy |
| License                           | MIT         | AGPL-3.0                            | Apache-2.0              |

They answer different questions. `crabrl` and `xbrl-rs` work with the
taxonomy as well as the instance and can tell you whether a document is valid.
`xbrlkit` does not load a taxonomy at all: it reads what the filing tags and
hands it to you as structs. If you need validation or the full specification,
those crates, or [Arelle], are the tools; if you need the numbers out of SEC
filings, inline ones included, this one is.

On raw parsing of XML instances, the one thing all three do,
[`benches/compare`](benches/compare) reads the same files with each, through
its documented entry point and default configuration:

| XML instance                        | Facts in file | xbrlkit          | crabrl           | xbrl-rs  |
| ----------------------------------- | ------------- | ---------------- | ---------------- | -------- |
| Apple 10-K (1.42 MB)                | 1,131         | 2.11 ms, 1,131   | 2.34 ms, 1,032   | rejected |
| Tesla 10-Q (1.47 MB)                | 1,221         | 2.58 ms, 1,221   | 2.51 ms, 1,049   | rejected |
| A SPAC's 10-Q (0.72 MB)             | 479           | 1.13 ms, 479     | 1.46 ms, 361     | rejected |

Each cell is the median time and the number of facts returned. `xbrlkit` and
`crabrl` are both built on `quick-xml` and parse at about the same speed.
`xbrl-rs` checks more as it reads and declines these files, the instances
EDGAR extracts, over the unprefixed unit measure `shares`.

[crabrl]: https://github.com/stefanoamorelli/crabrl
[xbrl-rs]: https://github.com/quambene/xbrl-rs
[Arelle]: https://arelle.org

## Scope

`xbrlkit` extracts; it does not validate.

- No taxonomy schemas or linkbases are read, so there are no labels, no
  calculation checks and no presentation trees
- Facts are found by concept name. A filer's own extension concept
  (`aapl:…`) is read like any other, by its name
- Tuples and footnotes are not modelled
- Inline transformations cover the SEC's registry and the English and numeric
  rules of the Transformation Rules Registry; other languages are passed
  through as the text the filing shows
- Built for and tested on SEC filings. Inline XBRL from other regulators
  (ESEF, HMRC) follows the same specification and may well parse, but nothing
  here is tested against it

## Features

| Feature      | Default | Adds                                                   |
| ------------ | ------- | ------------------------------------------------------ |
| `derive`     | yes     | `#[derive(FromXbrl)]`                                  |
| `taxonomies` | yes     | `taxonomies::dei` and `taxonomies::us_gaap`            |

With `default-features = false` the crate is the parser, the transformations
and the `FromXbrl` trait.

## Examples

```bash
cargo run --example financials -- path/to/filing.htm   # cover page and statements
cargo run --example custom_struct                      # your own view, as JSON
cargo run --example facts -- filing.htm us-gaap:Assets # every fact for a concept
```

With no path they read the Apple 10-K in `tests/fixtures`.

## Contributing

Issues and pull requests are welcome; see [CONTRIBUTING.md](CONTRIBUTING.md).
A filing that reads wrongly is the most useful report there is: its accession
number and the concept are enough.

## License

MIT. See [LICENSE](LICENSE).

This project is not affiliated with or endorsed by the U.S. Securities and
Exchange Commission. The filings in `tests/fixtures` are public documents
obtained from EDGAR.
