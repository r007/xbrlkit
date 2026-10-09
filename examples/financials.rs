//! Prints the cover page and the three financial statements of a filing.
//!
//! ```text
//! cargo run --example financials -- path/to/filing.htm
//! ```
//!
//! With no argument it reads the Apple 10-K in `tests/fixtures`.

use xbrlkit::Document;
use xbrlkit::taxonomies::{dei::DeiInfo, us_gaap::Financials};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "tests/fixtures/aapl-10k-2025.htm".to_string());
    let content = std::fs::read_to_string(&path)?;

    // Inline XBRL or an XML instance: `parse` tells them apart.
    let doc = Document::parse(&content)?;
    println!(
        "{path}: {} facts in {} contexts",
        doc.facts().len(),
        doc.contexts().len()
    );

    // `extract_lenient` keeps the struct when a filer tags something odd, and
    // says what it skipped.
    let (dei, problems) = doc.extract_lenient::<DeiInfo>();
    for problem in &problems {
        eprintln!("skipped: {problem}");
    }
    println!(
        "{} (CIK {}), form {} for the period ending {}",
        dei.entity.entity_registrant_name.as_deref().unwrap_or("?"),
        dei.entity
            .entity_central_index_key
            .as_deref()
            .unwrap_or("?"),
        dei.document.document_type.as_deref().unwrap_or("?"),
        dei.document
            .document_period_end_date
            .as_deref()
            .unwrap_or("?"),
    );

    let financials: Financials = doc.extract()?;

    // One per date the filing reports any balance sheet figure for. Dates
    // that appear only in the statement of shareholders' equity have an
    // equity figure and little else, so keep the ones that are whole.
    println!("\nBalance sheets");
    for sheet in financials
        .balance_sheets
        .iter()
        .filter(|sheet| sheet.assets.is_some())
    {
        println!(
            "  {}  assets {:>18}  liabilities {:>18}  equity {:>18}",
            sheet.as_of.as_deref().unwrap_or("?"),
            money(sheet.assets),
            money(sheet.liabilities),
            money(sheet.total_equity),
        );
    }

    println!("\nIncome statements");
    for statement in &financials.income_statements {
        println!(
            "  {} to {}  revenue {:>18}  net income {:>18}  diluted EPS {}",
            statement.period_start.as_deref().unwrap_or("?"),
            statement.period_end.as_deref().unwrap_or("?"),
            money(statement.revenue),
            money(statement.net_income),
            statement
                .earnings_per_share_diluted
                .map_or("-".to_string(), |eps| format!("{eps:.2}")),
        );
    }

    println!("\nCash flow statements");
    for statement in &financials.cash_flow_statements {
        println!(
            "  {} to {}  operating {:>18}  investing {:>18}  financing {:>18}",
            statement.period_start.as_deref().unwrap_or("?"),
            statement.period_end.as_deref().unwrap_or("?"),
            money(statement.operating_activities),
            money(statement.investing_activities),
            money(statement.financing_activities),
        );
    }

    Ok(())
}

/// `1234567.0` → `1,234,567`.
fn money(value: Option<f64>) -> String {
    let Some(value) = value else {
        return "-".to_string();
    };
    let digits = format!("{:.0}", value.abs());
    let mut grouped = String::new();
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    match value < 0.0 {
        true => format!("-{grouped}"),
        false => grouped,
    }
}
