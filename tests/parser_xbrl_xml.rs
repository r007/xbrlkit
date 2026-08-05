//! Regression coverage for traditional XBRL instance documents.
//!
//! SEC instance documents declare the XBRL instance namespace as the *default*
//! namespace, so `<context>` and `<unit>` arrive without a prefix. The parser used
//! to match those elements only in their `xbrli:`-prefixed spelling, which meant
//! every context in these documents was dropped. With no contexts to compare,
//! fact selection lost all period awareness and silently fell back to whichever
//! fact happened to appear last in the document — so a 10-Q reported its prior
//! year end balance sheet instead of the quarter being filed.

use std::fs::read_to_string;
use xbrl::taxonomies::{dei::extract_dei, us_gaap::extract_financials};

const FORM_10Q_FIXTURE: &str = "../fixtures/filings/form_10q.xml";
const FORM_10Q_1_FIXTURE: &str = "../fixtures/filings/form_10q_1.xml";
const FORM_10Q_2_FIXTURE: &str = "../fixtures/filings/form_10q_2.xml";
const FORM_10Q_3_FIXTURE: &str = "../fixtures/filings/form_10q_3.xml";
const FORM_10Q_4_FIXTURE: &str = "../fixtures/filings/form_10q_4.xml";

const ALL_FIXTURES: [&str; 5] = [
    FORM_10Q_FIXTURE,
    FORM_10Q_1_FIXTURE,
    FORM_10Q_2_FIXTURE,
    FORM_10Q_3_FIXTURE,
    FORM_10Q_4_FIXTURE,
];

fn read_fixture(path: &str) -> String {
    read_to_string(path).unwrap_or_else(|e| panic!("Failed to read {path}: {e}"))
}

/// Contexts declared in the default namespace must still be collected.
#[test]
fn test_default_namespace_contexts_are_parsed() {
    for path in ALL_FIXTURES {
        let content = read_fixture(path);
        let xbrl = xbrl::parser::extract_xbrl_data(&content)
            .unwrap_or_else(|e| panic!("Parsing {path} should succeed: {e}"));

        assert!(
            !xbrl.contexts.is_empty(),
            "{path} declares unprefixed <context> elements that must be parsed"
        );
        assert!(
            !xbrl.units.is_empty(),
            "{path} declares unprefixed <unit> elements that must be parsed"
        );

        // Every context a fact points at must actually resolve.
        for fact in &xbrl.facts {
            if let Some(context_ref) = &fact.context_ref {
                assert!(
                    xbrl.contexts.iter().any(|c| &c.id == context_ref),
                    "{path}: fact {} references unknown context {context_ref}",
                    fact.full_name
                );
            }
        }
    }
}

/// Context children must be consumed with their parent rather than harvested as
/// facts in their own right.
#[test]
fn test_context_children_are_not_emitted_as_facts() {
    for path in ALL_FIXTURES {
        let content = read_fixture(path);
        let xbrl = xbrl::parser::extract_xbrl_data(&content).expect("Parsing should succeed");

        let dimension_facts = xbrl
            .facts
            .iter()
            .filter(|f| f.local_name == "explicitMember" || f.local_name == "typedMember")
            .count();

        assert_eq!(
            dimension_facts, 0,
            "{path}: dimension members inside <context> leaked into the fact list"
        );
    }
}

/// The XML fact path must populate `local_name`, which drives lookups by
/// unqualified concept name.
#[test]
fn test_facts_carry_local_names() {
    let content = read_fixture(FORM_10Q_3_FIXTURE);
    let xbrl = xbrl::parser::extract_xbrl_data(&content).expect("Parsing should succeed");

    let missing = xbrl
        .facts
        .iter()
        .filter(|f| f.local_name.is_empty())
        .count();
    assert_eq!(missing, 0, "every fact should expose an unqualified name");

    let assets = xbrl
        .facts
        .iter()
        .find(|f| f.full_name == "us-gaap:Assets")
        .expect("fixture reports total assets");
    assert_eq!(assets.local_name, "Assets");
}

/// Total assets must come from the balance sheet date of the period being filed,
/// not from the prior year end that sits in the comparative column.
#[test]
fn test_balance_sheet_uses_reporting_date_not_comparative() {
    let content = read_fixture(FORM_10Q_3_FIXTURE);
    let context = xbrl::from_xbrl_str(&content).expect("XBRL parsing should succeed");

    let dei = extract_dei(&context).expect("DEI extraction should succeed");
    assert_eq!(
        dei.document.document_period_end_date,
        Some("2025-03-31".to_string())
    );

    let financials = extract_financials(&context).expect("US-GAAP extraction should succeed");

    // The fixture reports Assets twice: 357,335,566 at 2025-03-31 (the quarter
    // being filed) and 354,034,505 at 2024-12-31 (the comparative column).
    assert_eq!(
        financials.balance_sheet.assets,
        Some(357_335_566.0),
        "should report assets as of the balance sheet date"
    );
}

/// Selection must be stable regardless of how the two parser entry points reach
/// the same document.
#[test]
fn test_xml_and_ixbrl_entry_points_agree() {
    for path in ALL_FIXTURES {
        let content = read_fixture(path);

        let from_xml = xbrl::from_xbrl_str(&content).expect("XML parsing should succeed");
        let from_ixbrl = xbrl::from_ixbrl_str(&content).expect("iXBRL parsing should succeed");

        let xml_financials = extract_financials(&from_xml).expect("extraction should succeed");
        let ixbrl_financials = extract_financials(&from_ixbrl).expect("extraction should succeed");

        assert_eq!(
            xml_financials.balance_sheet.assets, ixbrl_financials.balance_sheet.assets,
            "{path}: both entry points should select the same total assets"
        );
    }
}
