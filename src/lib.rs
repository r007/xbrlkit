//! # High-Performance XBRL Parser for SEC Filings
//!
//! A specialized crate for parsing and processing complex XBRL (eXtensible Business Reporting Language)
//! filings from the SEC. This parser is optimized for financial data extraction from regulatory documents,
//! with particular focus on SPAC (Special Purpose Acquisition Company) filings.
//!
//! ## Supported Taxonomies
//!
//! - **US-GAAP**: Complete financial statements (balance sheet, income statement, cash flow)
//! - **DEI**: Document and Entity Information metadata
//! - **ECD**: Executive Compensation Disclosure (minimal subset for SPACs)
//!
//! ## iXBRL Transformation Layer
//!
//! Modern SEC filings use inline XBRL (iXBRL) with transformation format attributes that specify
//! how raw text values should be normalized. This parser includes a comprehensive transformation
//! layer that automatically handles 50+ SEC-specified transformations:
//!
//! - **Boolean transformations**: Checkbox characters (☐☑☒) → boolean values
//! - **Numeric transformations**: "seventy thousand" → "70000", "1,000,000" → "1000000"
//! - **Duration transformations**: "5 years, 2 months" → "P5Y2M" (ISO 8601)
//! - **Date transformations**: Various date formats → normalized dates
//! - **Normalization**: Exchange names, state codes, entity categories
//!
//! The transformation layer operates transparently during parsing - no configuration needed.
//! See `transformations` module for details.
//!
//! ## Reading facts into structs
//!
//! A document is a table of facts — one value per concept, per period, per
//! dimension member. `#[derive(FromXbrl)]` describes a typed view of it, with
//! each field bound to its concepts by `#[xbrl(..)]`. The struct's serde names
//! stay its Rust names, so what reaches JSON and Parquet is `assets`, not
//! `us-gaap:Assets`. See the [`bind`] module.

// `#[derive(FromXbrl)]` expands to paths under `::xbrl`, which this crate's own
// taxonomies use too.
extern crate self as xbrl;

pub mod bind;
pub mod error;
pub mod parser;
pub mod structures;
pub mod taxonomies;
pub mod transformations;

// Re-export key types for consumers of this crate
pub use bind::{Dimension, Fact, FromXbrl, Span, XbrlDataContext};
pub use error::{Result, XbrlError};
pub use structures::Xbrl;
/// Binds a struct's fields to XBRL concepts. See [`bind`].
pub use xbrl_derive::FromXbrl;

/// Parses a traditional XBRL XML document (fallback method).
/// Alias for `from_str` for backward compatibility.
pub fn from_xbrl_str(content: &str) -> Result<XbrlDataContext> {
    let xbrl = parser::extract_xbrl_data(content)?;
    Ok(XbrlDataContext::new(xbrl))
}

/// Parses an iXBRL HTML document (primary method for modern SEC filings).
pub fn from_ixbrl_str(content: &str) -> Result<XbrlDataContext> {
    let xbrl = parser::extract_ixbrl_data(content)?;
    Ok(XbrlDataContext::new(xbrl))
}

// Re-export the high-level taxonomy components for easy access
pub use taxonomies::{
    dei::{DeiInfo, extract_dei},
    us_gaap::{
        BalanceSheet, Breakdowns, CashFlowStatement, Financials, IncomeStatement,
        extract_financials,
    },
};
