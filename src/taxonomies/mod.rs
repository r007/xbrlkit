//! # Ready-made structs
//!
//! Views of a filing that most callers want, already declared:
//!
//! - [`dei`]: the cover page — who filed, what, for which period
//! - [`us_gaap`]: the balance sheet, income statement and cash flow statement
//!
//! They are ordinary `#[derive(FromXbrl)]` structs with no special access to
//! the crate. Read them as examples as much as use them: a struct of your own
//! is declared the same way.

pub mod dei;
pub mod us_gaap;

pub use dei::{DeiInfo, extract_dei};
pub use us_gaap::{
    BalanceSheet, CashFlowStatement, Financials, IncomeStatement, extract_financials,
};
