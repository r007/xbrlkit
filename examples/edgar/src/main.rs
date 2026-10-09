//! Fetches a company's latest annual or quarterly report from EDGAR with
//! [edgarkit](https://crates.io/crates/edgarkit) and prints its statements.
//!
//! ```text
//! cd examples/edgar
//! cargo run -- "Your Name you@example.com" 320193 10-K
//! ```
//!
//! The first argument is the User-Agent the SEC asks every client to send: a
//! name and a contact address. The second is the company's CIK, the third the
//! form (`10-K` or `10-Q`).

use edgarkit::{Edgar, FilingOperations};
use xbrlkit::Document;
use xbrlkit::taxonomies::{dei::DeiInfo, us_gaap::Financials};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let (Some(user_agent), Some(cik)) = (args.next(), args.next()) else {
        eprintln!("usage: edgar-financials \"Name contact@example.com\" <cik> [10-K|10-Q]");
        std::process::exit(2);
    };
    let form = args.next().unwrap_or_else(|| "10-K".to_string());

    // edgarkit finds the filing and downloads its primary document...
    let edgar = Edgar::new(&user_agent)?;
    let html = edgar.get_latest_filing_content(&cik, &[form.as_str()]).await?;

    // ...and xbrlkit reads the inline XBRL in it.
    let doc = Document::from_ixbrl(&html)?;
    let (dei, _) = doc.extract_lenient::<DeiInfo>();
    let (financials, problems) = doc.extract_lenient::<Financials>();
    for problem in &problems {
        eprintln!("skipped: {problem}");
    }

    println!(
        "{} — {} for the period ending {}",
        dei.entity.entity_registrant_name.as_deref().unwrap_or("?"),
        dei.document.document_type.as_deref().unwrap_or(&form),
        dei.document.document_period_end_date.as_deref().unwrap_or("?"),
    );
    for statement in &financials.income_statements {
        println!(
            "  {} to {}: revenue {:?}, net income {:?}",
            statement.period_start.as_deref().unwrap_or("?"),
            statement.period_end.as_deref().unwrap_or("?"),
            statement.revenue,
            statement.net_income,
        );
    }
    Ok(())
}
