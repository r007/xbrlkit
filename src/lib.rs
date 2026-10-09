//! Read SEC XBRL filings into typed Rust structs.
//!
//! `xbrlkit` reads the two forms an SEC filing carries its tagged data in —
//! inline XBRL, embedded in the HTML of a 10-K, 10-Q or 8-K, and the
//! traditional XML instance — and binds the facts to plain structs with
//! `#[derive(FromXbrl)]`.
//!
//! ```
//! use xbrlkit::{Document, FromXbrl};
//!
//! #[derive(Debug, Default, FromXbrl)]
//! struct Summary {
//!     #[xbrl(concept = "dei:EntityRegistrantName")]
//!     name: Option<String>,
//!
//!     #[xbrl(concept = "us-gaap:Assets")]
//!     assets: Option<f64>,
//!
//!     // The first concept the filing reports wins.
//!     #[xbrl(
//!         concept = "us-gaap:RevenueFromContractWithCustomerExcludingAssessedTax",
//!         alias = "us-gaap:Revenues"
//!     )]
//!     revenue: Option<f64>,
//! }
//!
//! # let html = r#"<html xmlns:ix="http://www.xbrl.org/2013/inlineXBRL"><body>
//! # <ix:header><ix:resources>
//! #   <xbrli:context id="fy"><xbrli:entity><xbrli:identifier scheme="http://www.sec.gov/CIK">0000320193</xbrli:identifier></xbrli:entity>
//! #     <xbrli:period><xbrli:startDate>2024-09-29</xbrli:startDate><xbrli:endDate>2025-09-27</xbrli:endDate></xbrli:period></xbrli:context>
//! #   <xbrli:context id="now"><xbrli:entity><xbrli:identifier scheme="http://www.sec.gov/CIK">0000320193</xbrli:identifier></xbrli:entity>
//! #     <xbrli:period><xbrli:instant>2025-09-27</xbrli:instant></xbrli:period></xbrli:context>
//! #   <xbrli:unit id="usd"><xbrli:measure>iso4217:USD</xbrli:measure></xbrli:unit>
//! # </ix:resources></ix:header>
//! # <ix:nonNumeric name="dei:EntityRegistrantName" contextRef="fy">Apple Inc.</ix:nonNumeric>
//! # <ix:nonNumeric name="dei:DocumentPeriodEndDate" contextRef="fy" format="ixt:date-monthname-day-year-en">September 27, 2025</ix:nonNumeric>
//! # <ix:nonFraction name="us-gaap:Assets" contextRef="now" unitRef="usd" scale="6" decimals="-6" format="ixt:num-dot-decimal">359,241</ix:nonFraction>
//! # <ix:nonFraction name="us-gaap:RevenueFromContractWithCustomerExcludingAssessedTax" contextRef="fy" unitRef="usd" scale="6" decimals="-6" format="ixt:num-dot-decimal">416,161</ix:nonFraction>
//! # </body></html>"#;
//! // `html` is a filing's primary document, as EDGAR serves it.
//! let doc = Document::parse(html)?;
//! let summary: Summary = doc.extract()?;
//!
//! assert_eq!(summary.name.as_deref(), Some("Apple Inc."));
//! assert_eq!(summary.assets, Some(359_241_000_000.0));
//! assert_eq!(summary.revenue, Some(416_161_000_000.0));
//! # Ok::<(), xbrlkit::XbrlError>(())
//! ```
//!
//! ## Two layers
//!
//! **The parser** ([`parser`]) keeps everything the filing tags: every
//! context, unit and fact, in an [`Instance`]. For inline XBRL a fact's value
//! is the text of everything inside its tag — whatever HTML wraps it,
//! continued through any `ix:continuation` — with its `format`
//! [transformation](transformations), `scale` and `sign` applied, so a loss
//! printed as `(1,234)` in thousands is `-1234000`. It is built for filings as
//! they are, malformed HTML included.
//!
//! **The binding** ([`bind`]) reads a struct from that table of facts. A
//! filing reports each concept many times over — this year and last, the
//! quarter and the year to date, the whole company and each segment — and a
//! field needs one of them. [`Document::extract`] picks, per field, the fact
//! for the period the filing reports on; `#[xbrl(each_period)]` gives a
//! statement per period instead.
//!
//! ## Works with serde
//!
//! The concept a field reads from lives in `#[xbrl(..)]`, not in a serde
//! rename, so a struct's serde names are its Rust names. Derive `Serialize`
//! next to `FromXbrl` and what reaches JSON, Arrow or Parquet is `assets`,
//! not `us-gaap:Assets`.
//!
//! ## Features
//!
//! | Feature      | Default | Adds                                                              |
//! | ------------ | ------- | ----------------------------------------------------------------- |
//! | `derive`     | yes     | `#[derive(FromXbrl)]`                                             |
//! | `taxonomies` | yes     | [`taxonomies::dei`] and [`taxonomies::us_gaap`], ready-made structs |
//!
//! Without either, the crate is the parser, the transformations and the
//! [`FromXbrl`] trait to implement by hand.
//!
//! ## What it is not
//!
//! `xbrlkit` extracts; it does not validate. It reads no taxonomy schemas or
//! linkbases, so it does not check calculations, resolve labels or know a
//! concept's type beyond what the fact itself says.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

// `#[derive(FromXbrl)]` expands to paths under `::xbrlkit`, which this
// crate's own taxonomies use too.
extern crate self as xbrlkit;

pub mod bind;
pub mod error;
pub mod instance;
pub mod parser;
#[cfg(feature = "taxonomies")]
#[cfg_attr(docsrs, doc(cfg(feature = "taxonomies")))]
pub mod taxonomies;
pub mod transformations;

pub use bind::{Dimension, Document, Fact, FromXbrl, Span};
pub use error::{Result, XbrlError};
pub use instance::{Instance, RawFact, XbrlValue};

/// Binds a struct's fields to XBRL concepts. See the [`bind`] module.
#[cfg(feature = "derive")]
#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
pub use xbrlkit_derive::FromXbrl;
