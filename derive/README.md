# xbrlkit-derive

`#[derive(FromXbrl)]` for [xbrlkit](https://crates.io/crates/xbrlkit): binds a
struct's fields to XBRL concepts.

This crate is an implementation detail of `xbrlkit`. Depend on `xbrlkit`,
which re-exports the derive under its default `derive` feature:

```rust
use xbrlkit::{Document, FromXbrl};

#[derive(Default, FromXbrl)]
struct BalanceSheet {
    #[xbrl(concept = "us-gaap:Assets")]
    assets: Option<f64>,

    #[xbrl(concept = "us-gaap:Liabilities")]
    liabilities: Option<f64>,
}

let sheet: BalanceSheet = Document::parse(&html)?.extract()?;
```

The attributes are documented in
[`xbrlkit::bind`](https://docs.rs/xbrlkit/latest/xbrlkit/bind/).

## License

MIT.
