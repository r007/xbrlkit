//! # XBRL Taxonomy Extractors
//!
//! This module contains high-level extractors for different XBRL taxonomies.
//! Each taxonomy provides strongly-typed structs and extraction functions.

pub mod dei;
pub mod us_gaap;

// Re-export the main types and functions for easy access
pub use dei::{DeiInfo, extract_dei};
pub use us_gaap::{
    BalanceSheet, CashFlowStatement, Financials, IncomeStatement, extract_financials,
};
