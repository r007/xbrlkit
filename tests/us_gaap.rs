//! The ready-made US-GAAP statements, read from real filings and held to the
//! figures printed in them.
#![cfg(feature = "taxonomies")]

use std::fs::read_to_string;
use xbrlkit::Document;
use xbrlkit::taxonomies::us_gaap::{Financials, IncomeStatement, extract_financials};

fn document(file: &str) -> Document {
    let content = read_to_string(format!("tests/fixtures/{file}")).expect("fixture is readable");
    Document::parse(&content).expect("fixture parses")
}

const MILLION: f64 = 1_000_000.0;

#[test]
fn an_annual_report_gives_the_statements_as_printed() {
    let financials = extract_financials(&document("aapl-10k-2025.htm")).unwrap();

    let sheet = &financials.balance_sheet;
    assert_eq!(sheet.assets, Some(359_241.0 * MILLION));
    assert_eq!(sheet.liabilities, Some(285_508.0 * MILLION));
    assert_eq!(sheet.stockholders_equity, Some(73_733.0 * MILLION));
    assert_eq!(sheet.total_equity, sheet.stockholders_equity);
    assert_eq!(sheet.liabilities_and_equity, sheet.assets);

    let income = &financials.income_statement;
    assert_eq!(income.revenue, Some(416_161.0 * MILLION));
    assert_eq!(income.net_income, Some(112_010.0 * MILLION));
    assert_eq!(income.earnings_per_share_diluted, Some(7.46));

    let cash = &financials.cash_flow_statement;
    assert_eq!(cash.operating_activities, Some(111_482.0 * MILLION));
    assert_eq!(cash.financing_activities, Some(-120_686.0 * MILLION));
}

#[test]
fn an_annual_report_carries_three_years_of_income() {
    let financials: Financials = document("aapl-10k-2025.htm").extract().unwrap();

    let years: Vec<(&str, f64)> = financials
        .income_statements
        .iter()
        .map(|s| (s.period_end.as_deref().unwrap(), s.revenue.unwrap()))
        .collect();
    assert_eq!(
        years,
        vec![
            ("2025-09-27", 416_161.0 * MILLION),
            ("2024-09-28", 391_035.0 * MILLION),
            ("2023-09-30", 383_285.0 * MILLION),
        ]
    );

    // Two balance sheets are printed in full; earlier dates appear only in
    // the statement of shareholders' equity.
    let with_assets: Vec<&str> = financials
        .balance_sheets
        .iter()
        .filter(|sheet| sheet.assets.is_some())
        .map(|sheet| sheet.as_of.as_deref().unwrap())
        .collect();
    assert_eq!(with_assets, vec!["2025-09-27", "2024-09-28"]);
}

#[test]
fn a_quarterly_report_keeps_the_quarter_apart_from_the_year_to_date() {
    let financials: Financials = document("tsla-10q-2026q2.htm").extract().unwrap();

    let periods: Vec<(&str, &str, f64)> = financials
        .income_statements
        .iter()
        .map(|s| {
            (
                s.period_start.as_deref().unwrap(),
                s.period_end.as_deref().unwrap(),
                s.revenue.unwrap(),
            )
        })
        .collect();
    assert_eq!(
        periods,
        vec![
            ("2026-04-01", "2026-06-30", 28_236.0 * MILLION),
            ("2026-01-01", "2026-06-30", 50_623.0 * MILLION),
            ("2025-04-01", "2025-06-30", 22_496.0 * MILLION),
            ("2025-01-01", "2025-06-30", 41_831.0 * MILLION),
        ]
    );

    // Read on its own, the statement is for the period the filing reports
    // on, which for a 10-Q is the year to date.
    assert_eq!(
        financials.income_statement.revenue,
        Some(50_623.0 * MILLION)
    );
    assert_eq!(financials.balance_sheet.assets, Some(148_524.0 * MILLION));

    // A cash flow statement is only ever reported for the year to date.
    let cash_flow_starts: Vec<&str> = financials
        .cash_flow_statements
        .iter()
        .map(|s| s.period_start.as_deref().unwrap())
        .collect();
    assert_eq!(cash_flow_starts, vec!["2026-01-01", "2025-01-01"]);
}

#[test]
fn the_inline_document_and_the_instance_give_the_same_statements() {
    for name in ["aapl-10k-2025", "tsla-10q-2026q2"] {
        let inline: Financials = document(&format!("{name}.htm")).extract().unwrap();
        let instance: Financials = document(&format!("{name}_htm.xml")).extract().unwrap();
        assert_eq!(inline, instance, "{name}");
    }
}

#[test]
fn statements_serialize_under_their_rust_names() {
    let financials: Financials = document("aapl-10k-2025.htm").extract().unwrap();

    let json = serde_json::to_value(&financials).unwrap();
    assert_eq!(json["balance_sheet"]["assets"], 359_241_000_000.0);
    assert_eq!(json["income_statements"][0]["period_end"], "2025-09-27");
    assert!(!json.to_string().contains("us-gaap:"));

    let back: Financials = serde_json::from_value(json).unwrap();
    assert_eq!(back, financials);
}

#[test]
fn one_statement_can_be_read_for_one_period() {
    let doc = document("tsla-10q-2026q2.htm");
    let quarter = xbrlkit::Span::duration("2026-04-01", "2026-06-30");

    let (statement, problems) = doc.extract_for::<IncomeStatement>(&quarter);
    assert!(problems.is_empty());
    assert_eq!(statement.revenue, Some(28_236.0 * MILLION));
    assert_eq!(statement.period_start.as_deref(), Some("2026-04-01"));

    let (each, _) = doc.extract_each_period::<IncomeStatement>();
    assert_eq!(each.len(), 4);
    assert_eq!(each[0], statement);
}

#[test]
fn a_filing_that_tags_no_statements_gives_empty_ones() {
    // An 8-K tags its cover page and nothing else.
    let financials: Financials = document("aapl-8k-2026.htm").extract().unwrap();
    assert_eq!(financials, Financials::default());
}
