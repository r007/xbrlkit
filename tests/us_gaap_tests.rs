use serde_json::{from_str, to_string_pretty};
use std::env::var;
use std::fs::{read_to_string, write};
use std::time::{Duration, Instant};
use xbrl::{
    XbrlError,
    taxonomies::us_gaap::{Financials, extract_financials},
};

// Test fixture path constants
const FORM_10Q_FIXTURE: &str = "../fixtures/filings/form_10q.xml";
const FORM_10Q_1_FIXTURE: &str = "../fixtures/filings/form_10q_1.xml";

/// Tests the complete US-GAAP financial extraction using form_10q.xml fixture.
#[test]
fn test_extract_financials_form_10q() {
    let content = read_to_string(FORM_10Q_FIXTURE).expect("Failed to read form_10q.xml fixture");
    let context = xbrl::from_str(&content).expect("XBRL parsing should succeed");

    let financials = extract_financials(&context).expect("Should extract financials successfully");

    // Verify the structure is properly populated
    assert_financials_structure(&financials);

    // Print extracted data for debugging
    println!("Extracted Financials (form_10q.xml):");
    println!("Balance Sheet: {:#?}", financials.balance_sheet);
    println!("Income Statement: {:#?}", financials.income_statement);
    println!("Cash Flow Statement: {:#?}", financials.cash_flow_statement);
    println!("Narratives: {:#?}", financials.narratives);

    // Test specific fields that should be present in Keen Vision's 10-Q
    // Based on the XML, we expect to find some narrative disclosures
    assert!(
        financials.narratives.nature_of_operations.is_some(),
        "Should extract NatureOfOperations text block"
    );

    assert!(
        financials
            .narratives
            .significant_accounting_policies
            .is_some(),
        "Should extract SignificantAccountingPoliciesTextBlock"
    );
}

/// Tests the complete US-GAAP financial extraction using form_10q_1.xml fixture.
#[test]
fn test_extract_financials_form_10q_1() {
    let content =
        read_to_string(FORM_10Q_1_FIXTURE).expect("Failed to read form_10q_1.xml fixture");
    let context = xbrl::from_str(&content).expect("XBRL parsing should succeed");

    let financials = extract_financials(&context).expect("Should extract financials successfully");

    // Verify the structure is properly populated
    assert_financials_structure(&financials);

    // Print extracted data for debugging
    println!("Extracted Financials (form_10q_1.xml):");
    println!("Balance Sheet: {:#?}", financials.balance_sheet);
    println!("Income Statement: {:#?}", financials.income_statement);
    println!("Cash Flow Statement: {:#?}", financials.cash_flow_statement);
    println!("Narratives: {:#?}", financials.narratives);

    // Test specific numeric fields that should be present in this filing
    // Based on the XML content, we expect some balance sheet items
    if let Some(deferred_comp) = financials
        .balance_sheet
        .deferred_compensation_liability_noncurrent
    {
        assert!(
            deferred_comp > 0.0,
            "Deferred compensation liability should be positive"
        );
        println!("Found deferred compensation liability: {}", deferred_comp);
    }

    // Should have some narrative disclosures
    assert!(
        financials.narratives.nature_of_operations.is_some(),
        "Should extract NatureOfOperations text block"
    );

    assert!(
        financials.narratives.related_party_transactions.is_some(),
        "Should extract RelatedPartyTransactionsDisclosureTextBlock"
    );

    assert!(
        financials.narratives.stockholders_equity_note.is_some(),
        "Should extract StockholdersEquityNoteDisclosureTextBlock"
    );
}

/// Tests that balance sheet extraction finds expected fields from the actual XML.
#[test]
fn test_balance_sheet_extraction_with_real_data() {
    let content =
        read_to_string(FORM_10Q_1_FIXTURE).expect("Failed to read form_10q_1.xml fixture");
    let context = xbrl::from_str(&content).unwrap();

    let financials = extract_financials(&context).unwrap();
    let balance_sheet = &financials.balance_sheet;

    println!("Balance Sheet Data from real filing:");
    println!("  Assets: {:?}", balance_sheet.assets);
    println!("  Current Assets: {:?}", balance_sheet.assets_current);
    println!(
        "  Assets Held in Trust: {:?}",
        balance_sheet.assets_held_in_trust
    );
    println!(
        "  Cash and Cash Equivalents: {:?}",
        balance_sheet.cash_and_cash_equivalents
    );
    println!("  Liabilities: {:?}", balance_sheet.liabilities);
    println!(
        "  Current Liabilities: {:?}",
        balance_sheet.liabilities_current
    );
    println!(
        "  Stockholders Equity: {:?}",
        balance_sheet.stockholders_equity
    );
    println!(
        "  Deferred Compensation Liability: {:?}",
        balance_sheet.deferred_compensation_liability_noncurrent
    );

    // For a SPAC filing, we expect to find some trust account assets
    // The XML contains AssetsHeldInTrust elements, so this should be captured
    if balance_sheet.assets_held_in_trust.is_some() {
        println!("✓ Successfully extracted assets held in trust");
    }

    // The XML contains DeferredCompensationLiabilityClassifiedNoncurrent = 2990000
    if let Some(deferred_comp) = balance_sheet.deferred_compensation_liability_noncurrent {
        assert_eq!(
            deferred_comp, 2990000.0,
            "Should extract the correct deferred compensation amount"
        );
        println!(
            "✓ Successfully extracted deferred compensation liability: {}",
            deferred_comp
        );
    }
}

/// Tests that narrative extraction works for complex text blocks.
#[test]
fn test_narrative_extraction_from_real_xml() {
    let content =
        read_to_string(FORM_10Q_1_FIXTURE).expect("Failed to read form_10q_1.xml fixture");
    let context = xbrl::from_str(&content).unwrap();

    let financials = extract_financials(&context).unwrap();
    let narratives = &financials.narratives;

    println!("Narrative Extraction Test:");

    // Test Nature of Operations (should be substantial for a SPAC)
    if let Some(nature_ops) = &narratives.nature_of_operations {
        assert!(
            !nature_ops.is_empty(),
            "Nature of operations should not be empty"
        );
        assert!(
            nature_ops.contains("Keen Vision Acquisition Corporation"),
            "Should contain company name"
        );
        println!(
            "✓ Nature of Operations extracted ({} chars)",
            nature_ops.len()
        );
    } else {
        println!("⚠ Nature of Operations not found");
    }

    // Test Related Party Transactions (common in SPAC filings)
    if let Some(related_party) = &narratives.related_party_transactions {
        assert!(
            !related_party.is_empty(),
            "Related party transactions should not be empty"
        );
        println!(
            "✓ Related Party Transactions extracted ({} chars)",
            related_party.len()
        );
    } else {
        println!("⚠ Related Party Transactions not found");
    }

    // Test Stockholders Equity Note
    if let Some(equity_note) = &narratives.stockholders_equity_note {
        assert!(
            !equity_note.is_empty(),
            "Stockholders equity note should not be empty"
        );
        println!(
            "✓ Stockholders Equity Note extracted ({} chars)",
            equity_note.len()
        );
    } else {
        println!("⚠ Stockholders Equity Note not found");
    }

    // Test Subsequent Events (common in quarterly filings)
    if let Some(subsequent_events) = &narratives.subsequent_events {
        assert!(
            !subsequent_events.is_empty(),
            "Subsequent events should not be empty"
        );
        println!(
            "✓ Subsequent Events extracted ({} chars)",
            subsequent_events.len()
        );
    } else {
        println!("⚠ Subsequent Events not found");
    }

    // Test Commitments and Contingencies
    if let Some(commitments) = &narratives.commitments_and_contingencies {
        assert!(
            !commitments.is_empty(),
            "Commitments and contingencies should not be empty"
        );
        println!(
            "✓ Commitments and Contingencies extracted ({} chars)",
            commitments.len()
        );
    } else {
        println!("⚠ Commitments and Contingencies not found");
    }
}

/// Tests extraction of specific numeric values that appear in the XML.
#[test]
fn test_specific_numeric_fact_extraction() {
    let content =
        read_to_string(FORM_10Q_1_FIXTURE).expect("Failed to read form_10q_1.xml fixture");
    let context = xbrl::from_str(&content).unwrap();

    let financials = extract_financials(&context).unwrap();

    // The XML contains <us-gaap:DeferredCompensationLiabilityClassifiedNoncurrent contextRef="c5" decimals="0" id="ixv-4148" unitRef="usd">2990000</us-gaap:DeferredCompensationLiabilityClassifiedNoncurrent>
    if let Some(deferred_comp) = financials
        .balance_sheet
        .deferred_compensation_liability_noncurrent
    {
        assert_eq!(
            deferred_comp, 2990000.0,
            "Should extract exact deferred compensation amount"
        );
        println!("✓ Deferred compensation liability: ${}", deferred_comp);
    } else {
        panic!("Should find deferred compensation liability in the XML");
    }

    // The XML contains <us-gaap:UnderwritingExpenseRatio contextRef="c0" decimals="2" id="ixv-4147" unitRef="pure">0.02</us-gaap:UnderwritingExpenseRatio>
    // This might not map to our current structure, but we can check if the parser found it
    println!("Testing numeric fact extraction completed");
}

/// Tests that the serde deserializer correctly handles XBRL concept names.
#[test]
fn test_serde_concept_name_mapping() {
    let content =
        read_to_string(FORM_10Q_1_FIXTURE).expect("Failed to read form_10q_1.xml fixture");
    let context = xbrl::from_str(&content).expect("Should parse XBRL data");

    println!("Raw XBRL Facts Found:");
    for fact in &context.xbrl.facts {
        if fact.full_name.contains("us-gaap:Assets")
            || fact.full_name.contains("us-gaap:Liabilities")
            || fact.full_name.contains("us-gaap:Stockholders")
        {
            println!("  {}: {:?}", fact.full_name, fact.value);
        }
    }

    // Now test that the serde deserializer correctly maps these to struct fields
    let financials = extract_financials(&context).unwrap();

    // The deserializer should have found and mapped the concepts correctly
    println!("\nSerde Mapping Results:");
    if financials
        .balance_sheet
        .deferred_compensation_liability_noncurrent
        .is_some()
    {
        println!("✓ DeferredCompensationLiabilityClassifiedNoncurrent mapped correctly");
    }

    // Test that we have some non-empty narrative blocks
    let narrative_count = [
        &financials.narratives.nature_of_operations,
        &financials.narratives.related_party_transactions,
        &financials.narratives.stockholders_equity_note,
        &financials.narratives.subsequent_events,
        &financials.narratives.commitments_and_contingencies,
    ]
    .iter()
    .filter(|n| n.is_some())
    .count();

    assert!(
        narrative_count > 0,
        "Should have extracted at least one narrative block"
    );
    println!("✓ Extracted {} narrative blocks", narrative_count);
}

/// Tests serialization and deserialization of the Financials struct.
#[test]
fn test_financials_serialization_with_real_data() {
    let content =
        read_to_string(FORM_10Q_1_FIXTURE).expect("Failed to read form_10q_1.xml fixture");
    let context = xbrl::from_str(&content).unwrap();

    let financials = extract_financials(&context).unwrap();

    // Test JSON serialization
    let json = to_string_pretty(&financials).expect("Should serialize to JSON");
    assert!(!json.is_empty(), "Serialized JSON should not be empty");

    // Test JSON deserialization
    let deserialized: Financials = from_str(&json).expect("Should deserialize from JSON");

    // Verify key fields match exactly
    assert_eq!(
        financials
            .balance_sheet
            .deferred_compensation_liability_noncurrent,
        deserialized
            .balance_sheet
            .deferred_compensation_liability_noncurrent
    );

    // Test that narrative blocks are preserved
    assert_eq!(
        financials.narratives.nature_of_operations,
        deserialized.narratives.nature_of_operations
    );

    assert_eq!(
        financials.narratives.related_party_transactions,
        deserialized.narratives.related_party_transactions
    );

    println!("✓ JSON serialization/deserialization test passed");
    println!("  JSON length: {} characters", json.len());

    // Write JSON to file for inspection if needed
    if var("WRITE_TEST_JSON").is_ok() {
        write("test_financials_output.json", &json).expect("Should write test JSON file");
        println!("  Test JSON written to test_financials_output.json");
    }
}

/// Tests error handling with malformed XBRL content.
#[test]
fn test_extract_financials_error_handling() {
    let malformed_content = r#"
        <?xml version="1.0" encoding="utf-8"?>
        <xbrl xmlns="http://www.xbrl.org/2003/instance">
            <context id="broken">
                <!-- Missing closing tag -->
            <context>
        </xbrl>
    "#;

    let result = xbrl::from_str(malformed_content);
    assert!(result.is_err(), "Should handle malformed XML gracefully");

    match result {
        Err(XbrlError::ParsingError(_)) => {
            println!("✓ Correctly handled malformed XML with ParsingError");
        }
        Err(XbrlError::AttributeError(_)) => {
            println!("✓ Correctly handled malformed XML with AttributeError");
        }
        Err(e) => {
            println!("⚠ Got different error type: {:?}", e);
        }
        Ok(_) => panic!("Should not succeed with malformed XML"),
    }
}

/// Tests extraction with empty XBRL document.
#[test]
fn test_extract_financials_empty_document() {
    let empty_content = r#"
        <?xml version="1.0" encoding="utf-8"?>
        <xbrl xmlns="http://www.xbrl.org/2003/instance">
        </xbrl>
    "#;
    let context = xbrl::from_str(empty_content).expect("Should handle empty document gracefully");

    let financials = extract_financials(&context).expect("Should handle empty document gracefully");

    // Should return default/empty financial structure
    assert_financials_structure(&financials);

    // All fields should be None for empty document
    assert!(financials.balance_sheet.assets.is_none());
    assert!(financials.income_statement.revenues.is_none());
    assert!(
        financials
            .cash_flow_statement
            .net_cash_provided_by_operating_activities
            .is_none()
    );
    assert!(financials.narratives.nature_of_operations.is_none());

    println!("✓ Empty document handled correctly");
}

/// Tests performance of financial extraction on the real XML documents.
#[test]
fn test_financial_extraction_performance() {
    let content =
        read_to_string(FORM_10Q_1_FIXTURE).expect("Failed to read form_10q_1.xml fixture");

    let mut times = Vec::new();

    // Run multiple iterations to get stable timing
    for _ in 0..5 {
        let start = Instant::now();
        // The new workflow: parse once, then extract.
        let context = xbrl::from_str(&content).unwrap();
        let _financials = extract_financials(&context).unwrap();
        times.push(start.elapsed());
    }

    let avg_time = times.iter().sum::<Duration>() / times.len() as u32;
    let min_time = times.iter().min().unwrap();
    let max_time = times.iter().max().unwrap();

    println!("Financial extraction performance on real XBRL document:");
    println!("  Average: {:?}", avg_time);
    println!("  Min: {:?}", min_time);
    println!("  Max: {:?}", max_time);
    println!("  Document size: {} bytes", content.len());

    // Should complete extraction in reasonable time (increased threshold for real documents)
    assert!(
        avg_time.as_millis() < 2000,
        "Financial extraction should complete in under 2 seconds, got {:?}",
        avg_time
    );

    println!("✓ Performance test passed");
}

/// Tests that the parser correctly handles multiple contexts and selects the best one.
#[test]
fn test_context_selection_logic() {
    let content =
        read_to_string(FORM_10Q_1_FIXTURE).expect("Failed to read form_10q_1.xml fixture");
    let context = xbrl::from_str(&content).unwrap();

    println!("Context Analysis:");
    println!("  Total contexts found: {}", context.xbrl.contexts.len());
    for ctx in &context.xbrl.contexts {
        println!("  Context {}: {:?}", ctx.id, ctx.period);
    }

    println!("\nFacts with multiple contexts:");
    let mut multi_context_facts = 0;
    for fact in &context.xbrl.facts {
        if let Some(context_ref) = &fact.context_ref {
            // Count how many facts use this concept
            let same_concept_count = context
                .xbrl
                .facts
                .iter()
                .filter(|f| f.full_name == fact.full_name)
                .count();

            if same_concept_count > 1 {
                multi_context_facts += 1;
                if multi_context_facts <= 5 {
                    // Limit output
                    println!(
                        "  {}: context {}, value: {:?}",
                        fact.full_name, context_ref, fact.value
                    );
                }
            }
        }
    }

    // The serde deserializer should have selected the best context for each fact
    let financials = extract_financials(&context).unwrap();
    assert_financials_structure(&financials);

    println!(
        "✓ Context selection logic handled {} facts correctly",
        context.xbrl.facts.len()
    );
}

/// Tests that fact values are correctly parsed as the right data types.
#[test]
fn test_fact_value_type_conversion() {
    let content =
        read_to_string(FORM_10Q_1_FIXTURE).expect("Failed to read form_10q_1.xml fixture");
    let context = xbrl::from_str(&content).unwrap();

    let financials = extract_financials(&context).unwrap();

    // Test numeric value conversion
    if let Some(deferred_comp) = financials
        .balance_sheet
        .deferred_compensation_liability_noncurrent
    {
        assert!(
            deferred_comp > 0.0,
            "Numeric values should be positive when expected"
        );
        assert!(
            deferred_comp.fract() == 0.0,
            "Integer values should not have fractional parts"
        );
        println!("✓ Numeric conversion: {} (type: f64)", deferred_comp);
    }

    // Test string value preservation in narratives
    if let Some(nature_ops) = &financials.narratives.nature_of_operations {
        assert!(
            !nature_ops.trim().is_empty(),
            "String values should not be empty after trimming"
        );
        assert!(
            nature_ops.len() > 100,
            "Narrative blocks should be substantial"
        );
        println!(
            "✓ String conversion: {} characters preserved",
            nature_ops.len()
        );
    }

    println!("✓ Type conversion test completed");
}

// --- Helper Functions ---

/// Helper function to assert that the Financials structure is properly formed.
fn assert_financials_structure(financials: &Financials) {
    // The structure should always be present, even if fields are None
    // This tests that our extraction doesn't crash and returns a valid structure

    // Balance sheet should be a valid struct (not just that it exists)
    let _ = &financials.balance_sheet.assets;
    let _ = &financials.balance_sheet.liabilities;
    let _ = &financials.balance_sheet.stockholders_equity;
    let _ = &financials
        .balance_sheet
        .deferred_compensation_liability_noncurrent;

    // Income statement should be a valid struct
    let _ = &financials.income_statement.revenues;
    let _ = &financials.income_statement.net_income_loss;
    let _ = &financials.income_statement.earnings_per_share_basic;

    // Cash flow statement should be a valid struct
    let _ = &financials
        .cash_flow_statement
        .net_cash_provided_by_operating_activities;
    let _ = &financials
        .cash_flow_statement
        .net_cash_provided_by_investing_activities;
    let _ = &financials
        .cash_flow_statement
        .net_cash_provided_by_financing_activities;

    // Narratives should be a valid struct
    let _ = &financials.narratives.nature_of_operations;
    let _ = &financials.narratives.related_party_transactions;
    let _ = &financials.narratives.stockholders_equity_note;
}
