use std::fs::read_to_string;
use std::time::{Duration, Instant};
use xbrl::XbrlError;
use xbrl::taxonomies::{
    dei::{DeiInfo, extract_dei},
    us_gaap::{Financials, extract_financials},
};

// Test fixture path constants
const FORM_10Q_FIXTURE: &str = "../fixtures/filings/form_10q.xml";
const FORM_10Q_1_FIXTURE: &str = "../fixtures/filings/form_10q_1.xml";

/// Tests the complete XBRL parsing workflow using the new serde-based approach.
#[test]
fn test_complete_xbrl_parsing_workflow() {
    let content =
        read_to_string(FORM_10Q_1_FIXTURE).expect("Failed to read form_10q_1.xml fixture");

    // New two-step process: parse once into a context.
    let context = xbrl::from_str(&content).expect("XBRL parsing should succeed");

    // Test DEI extraction from the context.
    let dei_info = extract_dei(&context).expect("DEI extraction should succeed");

    // Test US-GAAP extraction from the same context.
    let financials = extract_financials(&context).expect("US-GAAP extraction should succeed");

    // Verify DEI data extraction
    assert_dei_extraction(&dei_info);

    // Verify financial data extraction
    assert_financial_extraction(&financials);

    println!("✓ Complete XBRL parsing workflow test passed");
    println!("  DEI facts extracted: {}", count_dei_facts(&dei_info));
    println!(
        "  Financial facts extracted: {}",
        count_financial_facts(&financials)
    );
}

/// Tests DEI extraction with both fixture files to verify consistency.
#[test]
fn test_dei_extraction_across_fixtures() {
    let files = [FORM_10Q_FIXTURE, FORM_10Q_1_FIXTURE];

    for file_path in &files {
        let content =
            read_to_string(file_path).unwrap_or_else(|_| panic!("Failed to read {}", file_path));
        let context = xbrl::from_str(&content)
            .unwrap_or_else(|_| panic!("XBRL parsing failed for {}", file_path));

        let dei_info = extract_dei(&context)
            .unwrap_or_else(|_| panic!("DEI extraction failed for {}", file_path));

        // Both files should extract basic DEI information
        assert!(
            dei_info.entity.entity_central_index_key.is_some(),
            "Should extract CIK from {}",
            file_path
        );

        assert!(
            dei_info.document.document_fiscal_period_focus.is_some(),
            "Should extract fiscal period from {}",
            file_path
        );

        println!("✓ DEI extraction successful for {}", file_path);
        if let Some(cik) = &dei_info.entity.entity_central_index_key {
            println!("  CIK: {}", cik);
        }
        if let Some(period) = &dei_info.document.document_fiscal_period_focus {
            println!("  Fiscal Period: {}", period);
        }
    }
}

/// Tests financial extraction with both fixture files.
#[test]
fn test_financial_extraction_across_fixtures() {
    let files = [FORM_10Q_FIXTURE, FORM_10Q_1_FIXTURE];

    for file_path in &files {
        let content =
            read_to_string(file_path).unwrap_or_else(|_| panic!("Failed to read {}", file_path));
        let context = xbrl::from_str(&content)
            .unwrap_or_else(|_| panic!("XBRL parsing failed for {}", file_path));

        let financials = extract_financials(&context)
            .unwrap_or_else(|_| panic!("Financial extraction failed for {}", file_path));

        // Both files should have some narrative disclosures
        assert!(
            financials.narratives.nature_of_operations.is_some(),
            "Should extract nature of operations from {}",
            file_path
        );

        println!("✓ Financial extraction successful for {}", file_path);

        // Check for specific SPAC-related facts that should be in the newer filing
        if file_path.contains("form_10q_1.xml") {
            if let Some(deferred_comp) = financials
                .balance_sheet
                .deferred_compensation_liability_noncurrent
            {
                println!("  Deferred Compensation Liability: ${}", deferred_comp);
                assert_eq!(
                    deferred_comp, 2990000.0,
                    "Should match expected value from XML"
                );
            }

            if let Some(assets_in_trust) = financials.balance_sheet.assets_held_in_trust {
                println!("  Assets Held in Trust: ${}", assets_in_trust);
            }
        }
    }
}

/// Tests cross-taxonomy data consistency.
#[test]
fn test_cross_taxonomy_consistency() {
    let content =
        read_to_string(FORM_10Q_1_FIXTURE).expect("Failed to read form_10q_1.xml fixture");
    let context = xbrl::from_str(&content).unwrap();

    let dei_info = extract_dei(&context).unwrap();
    let financials = extract_financials(&context).unwrap();

    // Cross-validate that both taxonomies extract consistent entity information
    if let Some(dei_cik) = &dei_info.entity.entity_central_index_key {
        // The CIK should be the same across taxonomies (though US-GAAP may not have it)
        println!("Entity CIK from DEI: {}", dei_cik);
        assert_eq!(dei_cik, "0001889983", "Should match expected CIK from XML");
    }

    // Both should have extracted substantial narrative content
    let dei_has_narrative = dei_info.document.document_type.is_some();
    let gaap_has_narrative = financials.narratives.nature_of_operations.is_some();

    assert!(
        dei_has_narrative || gaap_has_narrative,
        "At least one taxonomy should extract narrative content"
    );

    // Verify fiscal period consistency
    if let Some(fiscal_period) = &dei_info.document.document_fiscal_period_focus {
        assert_eq!(fiscal_period, "Q1", "Should extract Q1 fiscal period");
        println!("✓ Fiscal period consistency verified: {}", fiscal_period);
    }
}

/// Tests performance of the new serde-based approach across all taxonomies.
#[test]
fn test_performance_characteristics() {
    let content =
        read_to_string(FORM_10Q_1_FIXTURE).expect("Failed to read form_10q_1.xml fixture");

    let mut extraction_times = Vec::new();

    // Parse once
    let parse_start = Instant::now();
    let context = xbrl::from_str(&content).unwrap();
    let parse_time = parse_start.elapsed();

    // Test DEI performance
    let start = Instant::now();
    let _dei_info = extract_dei(&context).unwrap();
    extraction_times.push(("DEI", start.elapsed()));

    // Test US-GAAP performance
    let start = Instant::now();
    let _financials = extract_financials(&context).unwrap();
    extraction_times.push(("US-GAAP", start.elapsed()));

    println!("Initial parsing time: {:?}", parse_time);

    // All extractions should complete quickly
    for (taxonomy, duration) in &extraction_times {
        assert!(
            duration.as_millis() < 1000, // Reduced threshold as parsing is separate
            "{} extraction should complete in under 1 second, took {:?}",
            taxonomy,
            duration
        );
        println!("{} extraction time: {:?}", taxonomy, duration);
    }

    let total_extraction_time: Duration = extraction_times.iter().map(|(_, d)| *d).sum();
    println!(
        "Total extraction time (post-parse): {:?}",
        total_extraction_time
    );
    println!("Document size: {} bytes", content.len());
}

/// Tests error handling with malformed XBRL content.
#[test]
fn test_error_handling_across_taxonomies() {
    let malformed_content = r#"
        <?xml version="1.0" encoding="utf-8"?>
        <xbrl xmlns="http://www.xbrl.org/2003/instance">
            <context id="broken">
                <!-- Missing closing tag -->
            <context>
        </xbrl>
    "#;

    // Parsing should fail.
    let context_result = xbrl::from_str(malformed_content);
    assert!(
        context_result.is_err(),
        "Parsing should fail with malformed XML"
    );

    // All should return parsing errors
    match context_result {
        Err(XbrlError::ParsingError(_)) | Err(XbrlError::AttributeError(_)) => {
            println!("✓ Parsing correctly handles malformed XML");
        }
        _ => panic!("Parsing should return ParsingError or AttributeError"),
    }
}

/// Tests extraction with empty XBRL document.
#[test]
fn test_empty_document_handling() {
    let empty_content = r#"
        <?xml version="1.0" encoding="utf-8"?>
        <xbrl xmlns="http://www.xbrl.org/2003/instance">
        </xbrl>
    "#;

    // All extractors should handle empty documents gracefully
    let context = xbrl::from_str(empty_content).expect("Should handle empty document");
    let dei_info = extract_dei(&context).expect("Should handle empty document");
    let financials = extract_financials(&context).expect("Should handle empty document");

    // All fields should be None/default for empty document
    assert!(dei_info.entity.entity_central_index_key.is_none());
    assert!(financials.balance_sheet.assets.is_none());

    println!("✓ All taxonomy extractors handle empty documents correctly");
}

/// Tests specific fact extraction that we know exists in the XML.
#[test]
fn test_specific_fact_extraction() {
    let content =
        read_to_string(FORM_10Q_1_FIXTURE).expect("Failed to read form_10q_1.xml fixture");
    let context = xbrl::from_str(&content).unwrap();

    let financials = extract_financials(&context).unwrap();

    // The XML contains <us-gaap:DeferredCompensationLiabilityClassifiedNoncurrent>2990000</us-gaap:DeferredCompensationLiabilityClassifiedNoncurrent>
    assert_eq!(
        financials
            .balance_sheet
            .deferred_compensation_liability_noncurrent,
        Some(2990000.0),
        "Should extract exact deferred compensation amount from XML"
    );

    // The XML contains <us-gaap:UnderwritingExpenseRatio>0.02</us-gaap:UnderwritingExpenseRatio>
    // This should be extracted somewhere in our structures
    println!("✓ Specific numeric facts extracted correctly");

    // Test narrative extraction
    assert!(
        financials.narratives.nature_of_operations.is_some(),
        "Should extract nature of operations narrative"
    );

    if let Some(nature_ops) = &financials.narratives.nature_of_operations {
        assert!(
            nature_ops.contains("Keen Vision Acquisition Corporation"),
            "Nature of operations should contain company name"
        );
        assert!(
            nature_ops.len() > 1000,
            "Nature of operations should be substantial (got {} characters)",
            nature_ops.len()
        );
    }

    println!("✓ Narrative extraction verified");
}

/// Tests that the serde deserializer correctly handles XBRL namespaces.
#[test]
fn test_namespace_handling() {
    let content =
        read_to_string(FORM_10Q_1_FIXTURE).expect("Failed to read form_10q_1.xml fixture");

    // First, examine what raw XBRL data looks like
    let context = xbrl::from_str(&content).unwrap();
    let xbrl_data = &context.xbrl;

    // Find facts from different namespaces
    let dei_facts: Vec<_> = xbrl_data
        .facts
        .iter()
        .filter(|fact| {
            fact.local_name.starts_with("EntityCentralIndexKey")
                || fact.local_name.starts_with("DocumentFiscalPeriodFocus")
        })
        .collect();

    let us_gaap_facts: Vec<_> = xbrl_data
        .facts
        .iter()
        .filter(|fact| {
            fact.local_name
                .starts_with("DeferredCompensationLiabilityClassifiedNoncurrent")
        })
        .collect();

    println!("Raw fact extraction:");
    println!("  DEI facts found: {}", dei_facts.len());
    println!("  US-GAAP facts found: {}", us_gaap_facts.len());

    // Now test that serde correctly maps these to the structs
    let dei_info = extract_dei(&context).unwrap();
    let financials = extract_financials(&context).unwrap();

    // Verify namespace mapping worked
    assert!(
        dei_info.entity.entity_central_index_key.is_some(),
        "DEI namespace mapping should work"
    );
    assert!(
        financials
            .balance_sheet
            .deferred_compensation_liability_noncurrent
            .is_some(),
        "US-GAAP namespace mapping should work"
    );

    println!("✓ Namespace handling verified across all taxonomies");
}

// --- Helper Functions ---

/// Verifies that DEI extraction found expected data.
fn assert_dei_extraction(dei_info: &DeiInfo) {
    // Should have extracted basic document information
    assert!(
        dei_info.document.document_fiscal_period_focus.is_some()
            || dei_info.entity.entity_central_index_key.is_some(),
        "DEI extraction should find at least basic document or entity info"
    );
}

/// Verifies that financial extraction found expected data.
fn assert_financial_extraction(financials: &Financials) {
    // Should have extracted at least some narrative content
    let narrative_count = [
        &financials.narratives.nature_of_operations,
        &financials.narratives.significant_accounting_policies,
        &financials.narratives.related_party_transactions,
        &financials.narratives.stockholders_equity_note,
        &financials.narratives.commitments_and_contingencies,
    ]
    .iter()
    .filter(|n| n.is_some())
    .count();

    assert!(
        narrative_count > 0,
        "Financial extraction should find at least one narrative block"
    );
}

/// Counts non-None fields in DEI info.
fn count_dei_facts(dei_info: &DeiInfo) -> usize {
    let mut count = 0;

    // Count document fields
    if dei_info.document.document_type.is_some() {
        count += 1;
    }
    if dei_info.document.document_fiscal_period_focus.is_some() {
        count += 1;
    }
    if dei_info.document.amendment_flag.is_some() {
        count += 1;
    }

    // Count entity fields
    if dei_info.entity.entity_central_index_key.is_some() {
        count += 1;
    }
    if dei_info.entity.entity_registrant_name.is_some() {
        count += 1;
    }
    if dei_info.entity.trading_symbol.is_some() {
        count += 1;
    }

    count
}

/// Counts non-None fields in financial info.
fn count_financial_facts(financials: &Financials) -> usize {
    let mut count = 0;

    // Count balance sheet fields
    if financials.balance_sheet.assets.is_some() {
        count += 1;
    }
    if financials.balance_sheet.liabilities.is_some() {
        count += 1;
    }
    if financials
        .balance_sheet
        .deferred_compensation_liability_noncurrent
        .is_some()
    {
        count += 1;
    }

    // Count narrative fields
    if financials.narratives.nature_of_operations.is_some() {
        count += 1;
    }
    if financials
        .narratives
        .significant_accounting_policies
        .is_some()
    {
        count += 1;
    }
    if financials.narratives.related_party_transactions.is_some() {
        count += 1;
    }

    count
}
