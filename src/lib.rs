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

pub mod error;
pub mod parser;
pub mod serde_xbrl;
pub mod structures;
pub mod taxonomies;

// Re-export key types for consumers of this crate
pub use error::{Result, XbrlError};
pub use serde_xbrl::XbrlDataContext;
pub use structures::Xbrl;

/// Parses an XBRL document from a string into a reusable data context.
/// This is the primary entry point for all XBRL processing.
pub fn from_str(content: &str) -> Result<XbrlDataContext> {
    let xbrl = parser::extract_xbrl_data(content)?;
    Ok(XbrlDataContext::new(xbrl))
}

// Re-export the high-level taxonomy components for easy access
pub use taxonomies::{
    dei::{DeiInfo, extract_dei},
    us_gaap::{BalanceSheet, CashFlowStatement, Financials, IncomeStatement, extract_financials},
};
