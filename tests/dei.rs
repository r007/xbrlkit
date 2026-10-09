//! The cover page, read from real filings.
#![cfg(feature = "taxonomies")]

use std::fs::read_to_string;
use xbrlkit::taxonomies::dei::{DeiInfo, extract_dei};
use xbrlkit::{Document, Span};

fn document(file: &str) -> Document {
    let content = read_to_string(format!("tests/fixtures/{file}")).expect("fixture is readable");
    Document::parse(&content).expect("fixture parses")
}

#[test]
fn an_annual_report_names_its_filer_period_and_auditor() {
    let doc = document("aapl-10k-2025.htm");
    let dei = extract_dei(&doc).unwrap();

    assert_eq!(dei.document.document_type.as_deref(), Some("10-K"));
    assert_eq!(dei.document.document_annual_report, Some(true));
    assert_eq!(dei.document.amendment_flag, Some(false));
    assert_eq!(
        dei.document.document_period_end_date.as_deref(),
        Some("2025-09-27")
    );
    assert_eq!(
        dei.document.document_fiscal_period_focus.as_deref(),
        Some("FY")
    );

    assert_eq!(
        dei.entity.entity_registrant_name.as_deref(),
        Some("Apple Inc.")
    );
    assert_eq!(
        dei.entity.entity_central_index_key.as_deref(),
        Some("0000320193")
    );
    assert_eq!(
        dei.entity
            .entity_incorporation_state_country_code
            .as_deref(),
        Some("CA")
    );
    assert_eq!(dei.entity.security_exchange_name.as_deref(), Some("NASDAQ"));
    assert_eq!(
        dei.entity.entity_filer_category.as_deref(),
        Some("Large Accelerated Filer")
    );
    assert_eq!(dei.entity.entity_shell_company, Some(false));
    // "September 27", normalised to a month and day.
    assert_eq!(
        dei.entity.current_fiscal_year_end_date.as_deref(),
        Some("--09-27")
    );
    assert!(dei.entity.entity_public_float.unwrap() > 1e12);

    assert_eq!(
        dei.entity_address.entity_address_city_or_town.as_deref(),
        Some("Cupertino")
    );
    assert_eq!(dei.audit.auditor_name.as_deref(), Some("Ernst & Young LLP"));
    assert_eq!(dei.audit.auditor_firm_id.as_deref(), Some("42"));
}

#[test]
fn the_reporting_period_is_the_cover_page_s_context() {
    assert_eq!(
        document("aapl-10k-2025.htm").reporting_period(),
        Some(&Span::duration("2024-09-29", "2025-09-27"))
    );
    // A 10-Q's required context is the year to date, not the quarter.
    assert_eq!(
        document("tsla-10q-2026q2.htm").reporting_period(),
        Some(&Span::duration("2026-01-01", "2026-06-30"))
    );
}

#[test]
fn a_current_report_has_a_cover_page_too() {
    let dei: DeiInfo = document("aapl-8k-2026.htm").extract().unwrap();

    assert_eq!(dei.document.document_type.as_deref(), Some("8-K"));
    assert_eq!(
        dei.document.document_period_end_date.as_deref(),
        Some("2026-07-30")
    );
    assert_eq!(
        dei.entity.entity_registrant_name.as_deref(),
        Some("Apple Inc.")
    );
    assert_eq!(dei.entity.entity_emerging_growth_company, Some(false));
    // Not an annual report: no auditor is named.
    assert_eq!(dei.audit.auditor_name, None);
}

#[test]
fn shares_outstanding_come_with_the_date_they_are_as_of() {
    let dei: DeiInfo = document("aapl-10k-2025.htm").extract().unwrap();

    let by_class = &dei.entity.common_stock_shares_outstanding_by_class;
    assert_eq!(by_class.len(), 1);
    assert_eq!(
        Some(by_class[0].value),
        dei.entity.entity_common_stock_shares_outstanding
    );
    assert_eq!(by_class[0].unit.as_deref(), Some("shares"));
    // The cover page counts shares as of a date after the fiscal year ends.
    assert!(by_class[0].period_end.as_deref() > Some("2025-09-27"));
    assert_eq!(by_class[0].period_start, None);
}
