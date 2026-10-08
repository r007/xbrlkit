# `xbrl`

Reads the financial statements a filing tags in XBRL — the traditional XML
instance or the inline iXBRL embedded in a 10-K or 10-Q's HTML — into typed
figures. It is how a periodic report's numbers reach the record without a model
call: the 10-K, 10-Q and 8-K processors read their financials through it, and
every `SpacSnapshot` on a SPAC's history is built from those figures alone.

| API                                         | Reads                                                                |
| ------------------------------------------- | -------------------------------------------------------------------- |
| `extract_xbrl_data()` / `from_xbrl_str()`   | A traditional XBRL XML instance                                      |
| `extract_ixbrl_data()` / `from_ixbrl_str()` | Inline XBRL in a filing's HTML                                       |
| `XbrlDataContext`                           | The parsed document, indexed: `extract()`, `extract_lenient()`       |
| `#[derive(FromXbrl)]`                       | A struct from the document, each field bound to its concepts         |
| `us_gaap::Financials`                       | The balance sheet, income statement and cash flows — per period too  |
| `dei::DeiInfo`                              | Document and Entity Information: CIK, entity name, period, auditor   |

## Two layers

**The parser** (`parser.rs`) keeps everything the filing tags: every context,
unit and fact. An inline fact's value is the text of everything inside it,
whatever HTML wraps it, continued through any `ix:continuation`, with its
`format` transformation (50+ of the SEC's), `scale` and `sign` applied — a loss
printed as `(1,234)` in thousands is `-1234000`. Facts nested in another fact
are facts too, which is every figure in a note tagged as a text block.

**The binding** (`bind.rs`) reads a struct from that table of facts:

```rust
#[derive(Default, Serialize, Deserialize, FromXbrl)]
#[xbrl(instant)]
pub struct BalanceSheet {
    #[xbrl(period_end)]
    pub as_of: Option<String>,

    #[xbrl(concept = "us-gaap:Assets")]
    pub assets: Option<f64>,

    // The first concept the filing reports wins.
    #[xbrl(concept = "us-gaap:AssetsHeldInTrustNoncurrent", alias = "us-gaap:AssetsHeldInTrust")]
    pub trust: Option<f64>,
}
```

The concept lives in `#[xbrl(..)]` and nowhere else, so the struct's serde
names are its Rust names: what reaches JSON and Parquet is `assets`, not
`us-gaap:Assets`. Every field's type is fixed, so `serde_arrow` derives the
schema from the type alone.

| A field of type                 | Holds                                                  |
| ------------------------------- | ------------------------------------------------------ |
| `Option<T>`                     | the value of the one best fact                         |
| `Option<Fact<T>>`               | that fact with its period, unit and dimensions         |
| `Vec<Fact<T>>`                  | every period and dimension member the concept has      |
| a struct, `#[xbrl(nested)]`     | more fields, read the same way                         |
| `Vec` of one, `#[xbrl(each_period)]` | that struct once per period the filing reports    |

## Which fact

Read on its own, a struct takes each field from the fact that best matches the
period the filing reports on — in a 10-Q, the year to date — falling back to a
comparative or a dimensional breakdown when that is all there is. Its fields
are chosen independently and can come from different periods.

Read through `each_period`, every field comes from one period's consolidated
contexts, and what the filing does not report for that period is `None`. Use it
when the period matters: `Financials::income_statements` holds the quarter
beside the year to date, each with last year's comparative.

Neither view holds a figure the filing reports only per class. A SPAC tags its
redeemable shares per share class and its warrants per warrant class, and
often tags no figure for the entity: the single-valued field then holds
whichever member came last. `Financials::breakdowns` keeps those concepts as
`Vec<Fact<f64>>` — the trust, the redeemable shares and their redemption
price, the warrants, the common shares, the related-party borrowings — every
date and member, for a reader that asks for one date and one class.
