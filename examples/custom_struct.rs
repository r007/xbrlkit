//! Declares its own view of a filing and prints it as JSON.
//!
//! The ready-made structs in `xbrlkit::taxonomies` cover the face of the
//! statements. Anything else — a ratio's inputs, a note's figures, a per-class
//! breakdown — is a struct away.
//!
//! ```text
//! cargo run --example custom_struct -- path/to/filing.htm
//! ```
//!
//! With no argument it reads the Apple 10-K in `tests/fixtures`.

use serde::Serialize;
use xbrlkit::{Document, Fact, FromXbrl};

/// What it takes to compute a few liquidity and return ratios.
#[derive(Debug, Default, Serialize, FromXbrl)]
struct Snapshot {
    #[xbrl(concept = "dei:EntityRegistrantName")]
    company: Option<String>,

    #[xbrl(concept = "dei:DocumentPeriodEndDate")]
    period_end: Option<String>,

    #[xbrl(concept = "us-gaap:AssetsCurrent")]
    current_assets: Option<f64>,

    #[xbrl(concept = "us-gaap:LiabilitiesCurrent")]
    current_liabilities: Option<f64>,

    /// Filers pick among several concepts for the same line. List them in
    /// order of preference; the first one the filing reports is used.
    #[xbrl(
        concept = "us-gaap:RevenueFromContractWithCustomerExcludingAssessedTax",
        alias = "us-gaap:Revenues",
        alias = "us-gaap:SalesRevenueNet"
    )]
    revenue: Option<f64>,

    /// `Option<Fact<T>>` keeps the context: which period, what unit, how exact.
    #[xbrl(concept = "us-gaap:NetIncomeLoss")]
    net_income: Option<Fact<f64>>,

    /// `Vec<Fact<T>>` keeps every period and every dimension member: here,
    /// revenue for each product line and region the filing breaks out.
    #[xbrl(concept = "us-gaap:RevenueFromContractWithCustomerExcludingAssessedTax")]
    revenue_breakdown: Vec<Fact<f64>>,

    /// One struct per period: R&D for each year the filing reports.
    #[xbrl(each_period)]
    research: Vec<Research>,

    /// No attribute: left at its default for the caller to fill.
    current_ratio: Option<f64>,
}

#[derive(Debug, Default, Serialize, FromXbrl)]
#[xbrl(duration)]
struct Research {
    #[xbrl(period_start)]
    from: Option<String>,

    #[xbrl(period_end)]
    to: Option<String>,

    #[xbrl(concept = "us-gaap:ResearchAndDevelopmentExpense")]
    expense: Option<f64>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "tests/fixtures/aapl-10k-2025.htm".to_string());
    let doc = Document::parse(&std::fs::read_to_string(path)?)?;

    let mut snapshot: Snapshot = doc.extract()?;

    snapshot.current_ratio = match (snapshot.current_assets, snapshot.current_liabilities) {
        (Some(assets), Some(liabilities)) if liabilities != 0.0 => Some(assets / liabilities),
        _ => None,
    };
    // Keep the breakdown for the reporting period only, and only the facts
    // that are narrowed by a dimension member.
    let period_end = snapshot.period_end.clone();
    snapshot
        .revenue_breakdown
        .retain(|fact| fact.period_end == period_end && !fact.dimensions.is_empty());

    // The struct's serde names are its field names: no `us-gaap:` in the JSON.
    println!("{}", serde_json::to_string_pretty(&snapshot)?);
    Ok(())
}
