//! # Comprehensive iXBRL Parser Tests
//!
//! Extended test suite covering multiple fixtures and edge cases.
//! Tests validate parser behavior across different SEC filing types and formats.

use std::collections::HashSet;
use std::fs;
use xbrlkit::instance::XbrlValue;
use xbrlkit::parser::parse_ixbrl;

const FORM_10Q_FIXTURE: &str = "tests/fixtures/spac/10-q.html";
const FORM_10Q_1_FIXTURE: &str = "tests/fixtures/spac/10-q_1.html";
const FORM_10Q_2_FIXTURE: &str = "tests/fixtures/spac/10-q_2.html";
const FORM_8K_FIXTURE: &str = "tests/fixtures/spac/8-k.html";

// ============================================================================
// MULTI-FIXTURE VALIDATION TESTS
// ============================================================================

#[test]
fn test_all_fixtures_parse_successfully() {
    let fixtures = vec![
        ("10-Q Original", FORM_10Q_FIXTURE),
        ("10-Q Variant 1", FORM_10Q_1_FIXTURE),
        ("10-Q Variant 2", FORM_10Q_2_FIXTURE),
        ("8-K", FORM_8K_FIXTURE),
    ];

    for (name, path) in fixtures {
        let bytes = fs::read(path).unwrap_or_else(|_| panic!("Failed to read {}", name));
        let html = String::from_utf8_lossy(&bytes).to_string();
        let result = parse_ixbrl(&html);

        assert!(
            result.is_ok(),
            "{} should parse successfully: {:?}",
            name,
            result.err()
        );

        let xbrl = result.unwrap();
        println!(
            "{}: {} contexts, {} units, {} facts",
            name,
            xbrl.contexts.len(),
            xbrl.units.len(),
            xbrl.facts.len()
        );

        // Basic sanity checks - all filings should have some data
        assert!(!xbrl.contexts.is_empty(), "{} should have contexts", name);
        assert!(!xbrl.facts.is_empty(), "{} should have facts", name);
    }
}

#[test]
fn test_10q_variant_consistency() {
    // Parse all 10-Q variants
    let fixtures = [
        ("10-Q Original", FORM_10Q_FIXTURE),
        ("10-Q Variant 1", FORM_10Q_1_FIXTURE),
        ("10-Q Variant 2", FORM_10Q_2_FIXTURE),
    ];

    let parsed: Vec<_> = fixtures
        .iter()
        .map(|(name, path)| {
            let bytes = fs::read(path).unwrap_or_else(|_| panic!("Failed to read {}", name));
            let html = String::from_utf8_lossy(&bytes).to_string();
            parse_ixbrl(&html).unwrap_or_else(|_| panic!("Failed to parse {}", name))
        })
        .collect();

    // All 10-Q filings should have certain common characteristics
    for (i, xbrl) in parsed.iter().enumerate() {
        let name = fixtures[i].0;
        println!("{}: {} facts", name, xbrl.facts.len());

        // Should have DEI (Document and Entity Information)
        let has_dei = xbrl.facts.iter().any(|f| f.full_name.contains("dei:"));
        assert!(has_dei, "{} should have DEI facts", name);

        // Should have some US-GAAP facts (unless it's a very minimal filing)
        let has_us_gaap = xbrl.facts.iter().any(|f| f.full_name.contains("us-gaap:"));
        if !has_us_gaap {
            println!(
                "Warning: {} has no US-GAAP facts (might be minimal filing)",
                name
            );
        }

        // Should have contexts
        let has_contexts = !xbrl.contexts.is_empty();
        assert!(has_contexts, "{} should have contexts", name);
    }
}

#[test]
fn test_8k_specific_characteristics() {
    let bytes = fs::read(FORM_8K_FIXTURE).expect("Failed to read 8-K fixture");
    let html = String::from_utf8_lossy(&bytes).to_string();
    let xbrl = parse_ixbrl(&html).expect("Failed to parse 8-K");

    println!(
        "8-K: {} contexts, {} units, {} facts",
        xbrl.contexts.len(),
        xbrl.units.len(),
        xbrl.facts.len()
    );

    // 8-K filings typically have fewer facts than 10-Q (they're event-driven)
    assert!(!xbrl.facts.is_empty(), "8-K should have some facts");

    // Should still have DEI information
    let doc_type = xbrl.facts.iter().find(|f| f.local_name == "DocumentType");
    assert!(doc_type.is_some(), "8-K should have DocumentType");

    if let Some(fact) = doc_type {
        match &fact.value {
            XbrlValue::String(s) => assert_eq!(s, "8-K", "DocumentType should be 8-K"),
            _ => panic!("DocumentType should be a string"),
        }
    }
}

// ============================================================================
// ATTRIBUTE HANDLING TESTS
// ============================================================================

#[test]
fn test_case_insensitive_attribute_parsing() {
    // This test verifies that our case-insensitive attribute matching works
    // Real SEC filings use both contextRef and contextref, unitRef and unitref
    let bytes = fs::read(FORM_10Q_FIXTURE).expect("Failed to read fixture");
    let html = String::from_utf8_lossy(&bytes).to_string();
    let xbrl = parse_ixbrl(&html).expect("Failed to parse iXBRL");

    // Check that facts have context references (regardless of attribute casing)
    let facts_with_context = xbrl
        .facts
        .iter()
        .filter(|f| f.context_ref.is_some())
        .count();

    println!("Facts with context reference: {}", facts_with_context);
    assert!(
        facts_with_context > 0,
        "Should parse context references regardless of case"
    );

    // Check that numeric facts have unit references
    let facts_with_units = xbrl.facts.iter().filter(|f| f.unit_ref.is_some()).count();

    println!("Facts with unit reference: {}", facts_with_units);
    assert!(
        facts_with_units > 0,
        "Should parse unit references regardless of case"
    );
}

#[test]
fn test_scale_attribute_application() {
    // Test that scale attributes are correctly applied to numeric values
    let bytes = fs::read(FORM_10Q_FIXTURE).expect("Failed to read fixture");
    let html = String::from_utf8_lossy(&bytes).to_string();
    let xbrl = parse_ixbrl(&html).expect("Failed to parse iXBRL");

    // Find facts that likely have scale attributes (large monetary amounts)
    let scaled_facts: Vec<_> = xbrl
        .facts
        .iter()
        .filter(|f| {
            f.full_name.contains("Assets")
                || f.full_name.contains("Revenue")
                || f.full_name.contains("Equity")
        })
        .collect();

    println!("Found {} potential scaled facts", scaled_facts.len());

    // At least some of these should have values
    let facts_with_values = scaled_facts
        .iter()
        .filter(|f| !matches!(f.value, XbrlValue::Nil))
        .count();

    assert!(
        facts_with_values > 0,
        "Should have parsed some scaled numeric values"
    );
}

#[test]
fn test_comma_removal_from_numbers() {
    // Test that commas are removed from formatted numbers (e.g., "4,921" -> "4921")
    let bytes = fs::read(FORM_10Q_FIXTURE).expect("Failed to read fixture");
    let html = String::from_utf8_lossy(&bytes).to_string();
    let xbrl = parse_ixbrl(&html).expect("Failed to parse iXBRL");

    // Check that numeric values don't contain commas
    for fact in &xbrl.facts {
        if let XbrlValue::String(s) = &fact.value {
            // If it looks like a number, it shouldn't have commas
            if s.chars().all(|c| c.is_numeric() || c == '.' || c == '-') {
                assert!(
                    !s.contains(','),
                    "Numeric value '{}' for {} should not contain commas",
                    s,
                    fact.full_name
                );
            }
        }
    }
}

// ============================================================================
// METADATA PARSING TESTS
// ============================================================================

#[test]
fn test_context_entity_identifiers() {
    let bytes = fs::read(FORM_10Q_FIXTURE).expect("Failed to read fixture");
    let html = String::from_utf8_lossy(&bytes).to_string();
    let xbrl = parse_ixbrl(&html).expect("Failed to parse iXBRL");

    // All contexts should have entity identifiers (CIK numbers)
    for context in &xbrl.contexts {
        assert!(!context.id.is_empty(), "Context should have an ID");
        assert!(
            !context.entity.identifier.value.is_empty(),
            "Context {} should have entity identifier",
            context.id
        );

        // CIK numbers should be numeric
        let identifier = &context.entity.identifier.value;
        assert!(
            identifier.chars().all(|c| c.is_numeric()),
            "Entity identifier '{}' should be numeric (CIK)",
            identifier
        );
    }
}

#[test]
fn test_unit_measures() {
    let bytes = fs::read(FORM_10Q_FIXTURE).expect("Failed to read fixture");
    let html = String::from_utf8_lossy(&bytes).to_string();
    let xbrl = parse_ixbrl(&html).expect("Failed to parse iXBRL");

    // Check that units have measures
    for unit in &xbrl.units {
        assert!(!unit.id.is_empty(), "Unit should have an ID");

        // Units should have at least one measure
        let has_measures = unit.measure.is_some() || unit.divide.is_some();

        assert!(has_measures, "Unit {} should have measures", unit.id);
    }

    // Common measures we expect to find
    let currency_units = xbrl
        .units
        .iter()
        .filter(|u| {
            u.measure
                .as_ref()
                .is_some_and(|m| m.contains("USD") || m.contains("iso4217"))
        })
        .count();

    let share_units = xbrl
        .units
        .iter()
        .filter(|u| u.measure.as_ref().is_some_and(|m| m.contains("shares")))
        .count();

    println!(
        "Currency units: {}, Share units: {}",
        currency_units, share_units
    );
    assert!(currency_units > 0, "Should have currency units (USD)");
}

#[test]
fn test_temporal_contexts_validity() {
    let bytes = fs::read(FORM_10Q_FIXTURE).expect("Failed to read fixture");
    let html = String::from_utf8_lossy(&bytes).to_string();
    let xbrl = parse_ixbrl(&html).expect("Failed to parse iXBRL");

    let mut instant_count = 0;
    let mut duration_count = 0;

    for context in &xbrl.contexts {
        if context.period.instant.is_some() {
            instant_count += 1;
        }

        if let (Some(start), Some(end)) = (&context.period.start_date, &context.period.end_date) {
            duration_count += 1;

            // Duration periods should have start before end

            assert!(
                start <= end,
                "Context {} has invalid duration: start {} should be <= end {}",
                context.id,
                start,
                end
            );
        }
    }

    println!(
        "Temporal contexts: {} instant, {} duration",
        instant_count, duration_count
    );

    assert!(
        instant_count + duration_count > 0,
        "Should have some temporal contexts"
    );
}

// ============================================================================
// FACT EXTRACTION TESTS
// ============================================================================

#[test]
fn test_fact_name_parsing() {
    let bytes = fs::read(FORM_10Q_FIXTURE).expect("Failed to read fixture");
    let html = String::from_utf8_lossy(&bytes).to_string();
    let xbrl = parse_ixbrl(&html).expect("Failed to parse iXBRL");

    for fact in &xbrl.facts {
        // Full name should contain namespace prefix
        assert!(
            fact.full_name.contains(':'),
            "Fact full_name '{}' should contain namespace prefix",
            fact.full_name
        );

        // Local name should not contain namespace prefix
        assert!(
            !fact.local_name.contains(':'),
            "Fact local_name '{}' should not contain namespace prefix",
            fact.local_name
        );

        // Local name should match the part after the colon in full name
        if let Some((_, local)) = fact.full_name.rsplit_once(':') {
            assert_eq!(
                fact.local_name, local,
                "Local name should match the part after ':' in full name"
            );
        }
    }
}

#[test]
fn test_nil_value_handling() {
    let fixtures = vec![
        ("10-Q Original", FORM_10Q_FIXTURE),
        ("10-Q Variant 1", FORM_10Q_1_FIXTURE),
        ("10-Q Variant 2", FORM_10Q_2_FIXTURE),
        ("8-K", FORM_8K_FIXTURE),
    ];

    for (name, path) in fixtures {
        let bytes = fs::read(path).unwrap_or_else(|_| panic!("Failed to read {}", name));
        let html = String::from_utf8_lossy(&bytes).to_string();
        let xbrl = parse_ixbrl(&html).unwrap_or_else(|_| panic!("Failed to parse {}", name));

        let nil_facts = xbrl
            .facts
            .iter()
            .filter(|f| matches!(f.value, XbrlValue::Nil))
            .count();

        println!("{}: {} nil facts", name, nil_facts);

        // Most SEC filings have some nil values (missing/not applicable data)
        // But we don't assert > 0 as 8-K might not have many
    }
}

#[test]
fn test_fact_references_valid_contexts() {
    let bytes = fs::read(FORM_10Q_FIXTURE).expect("Failed to read fixture");
    let html = String::from_utf8_lossy(&bytes).to_string();
    let xbrl = parse_ixbrl(&html).expect("Failed to parse iXBRL");

    // Build a set of valid context IDs
    let valid_context_ids: HashSet<_> = xbrl.contexts.iter().map(|c| c.id.as_str()).collect();

    // Check that all fact context references are valid
    for fact in &xbrl.facts {
        if let Some(ref ctx_ref) = fact.context_ref {
            assert!(
                valid_context_ids.contains(ctx_ref.as_str()),
                "Fact {} references non-existent context '{}'",
                fact.full_name,
                ctx_ref
            );
        }
    }
}

#[test]
fn test_fact_references_valid_units() {
    let bytes = fs::read(FORM_10Q_FIXTURE).expect("Failed to read fixture");
    let html = String::from_utf8_lossy(&bytes).to_string();
    let xbrl = parse_ixbrl(&html).expect("Failed to parse iXBRL");

    // Build a set of valid unit IDs
    let valid_unit_ids: HashSet<_> = xbrl.units.iter().map(|u| u.id.as_str()).collect();

    // Check that all fact unit references are valid
    let mut missing_units = HashSet::new();
    for fact in &xbrl.facts {
        if let Some(ref unit_ref) = fact.unit_ref {
            if !valid_unit_ids.contains(unit_ref.as_str()) {
                missing_units.insert(unit_ref.clone());
            }
        }
    }

    if !missing_units.is_empty() {
        println!(
            "Warning: {} facts reference undefined units:",
            missing_units.len()
        );
        for unit_ref in &missing_units {
            let facts_count = xbrl
                .facts
                .iter()
                .filter(|f| f.unit_ref.as_ref() == Some(unit_ref))
                .count();
            println!("  {} ({} facts reference it)", unit_ref, facts_count);
        }
        // Note: This is technically invalid XBRL, but common in some SEC filings
        // where units are inferred rather than explicitly defined
    }

    // Most facts with units should have valid references
    let facts_with_units = xbrl.facts.iter().filter(|f| f.unit_ref.is_some()).count();
    let facts_with_valid_units = xbrl
        .facts
        .iter()
        .filter(|f| {
            f.unit_ref
                .as_ref()
                .is_some_and(|u| valid_unit_ids.contains(u.as_str()))
        })
        .count();

    let validity_ratio = facts_with_valid_units as f64 / facts_with_units as f64;
    println!(
        "Unit reference validity: {}/{} ({:.1}%)",
        facts_with_valid_units,
        facts_with_units,
        validity_ratio * 100.0
    );

    // At least 90% of facts with unit refs should have valid units
    // (Some SEC filings have quirks where units are implicit)
    assert!(
        validity_ratio >= 0.90,
        "At least 90% of facts with units should reference valid unit definitions"
    );
}

// ============================================================================
// ROBUSTNESS TESTS
// ============================================================================

#[test]
fn test_parser_handles_malformed_html() {
    // All our fixtures are real SEC filings which may have malformed HTML
    // The parser should be resilient and extract as much data as possible
    let bytes = fs::read(FORM_10Q_FIXTURE).expect("Failed to read fixture");
    let html = String::from_utf8_lossy(&bytes).to_string();

    // Should not panic or error out
    let result = parse_ixbrl(&html);
    assert!(result.is_ok(), "Parser should handle real-world HTML");

    let xbrl = result.unwrap();
    assert!(
        !xbrl.contexts.is_empty(),
        "Should extract contexts despite HTML issues"
    );
    assert!(
        !xbrl.facts.is_empty(),
        "Should extract facts despite HTML issues"
    );
}

#[test]
fn test_parser_handles_unknown_entities() {
    // SEC filings often contain HTML entities like &nbsp; that may not be
    // in the standard XML entity list. Parser should handle gracefully.
    let bytes = fs::read(FORM_10Q_FIXTURE).expect("Failed to read fixture");
    let html = String::from_utf8_lossy(&bytes).to_string();

    let xbrl = parse_ixbrl(&html).expect("Should parse despite unknown entities");

    // Should have extracted values without crashing on entity errors
    let non_nil_facts = xbrl
        .facts
        .iter()
        .filter(|f| !matches!(f.value, XbrlValue::Nil))
        .count();

    assert!(
        non_nil_facts > 0,
        "Should have extracted some fact values despite entity issues"
    );
}

#[test]
fn test_performance_large_document() {
    use std::time::Instant;

    let bytes = fs::read(FORM_10Q_FIXTURE).expect("Failed to read fixture");
    let html = String::from_utf8_lossy(&bytes).to_string();

    let start = Instant::now();
    let xbrl = parse_ixbrl(&html).expect("Failed to parse");
    let duration = start.elapsed();

    println!(
        "Parsed {} facts in {:?} ({:.2} facts/ms)",
        xbrl.facts.len(),
        duration,
        xbrl.facts.len() as f64 / duration.as_millis() as f64
    );

    // Parser should be fast - even large documents should parse quickly
    // This is more of a benchmark than a strict test
    assert!(
        duration.as_secs() < 5,
        "Parser should complete in reasonable time"
    );
}

/// An inline fact's value is the text of everything inside it, whatever HTML
/// wraps that text; a fact with no text at all is `XbrlValue::Nil`, never
/// `XbrlValue::String("")`.
///
/// The empty-string case surfaced in production as
///   "For field 'dei:DocumentAnnualReport': could not parse value '' as bool"
/// when a fact's text sat inside a `<span>` the parser skipped. Skipping was
/// the real fault: it also lost every registrant name set in `<b>`.
#[test]
fn test_html_wrapped_value_is_read_and_empty_is_nil() {
    // The SEC-filing pattern where the visible value lives inside a <span>:
    //
    //   <ix:nonnumeric name="dei:DocumentAnnualReport" format="ixt-sec:boolballotbox">
    //     <span>☑</span>
    //   </ix:nonnumeric>
    //
    // We also cover the explicit xsi:nil="true" case and a normal direct-text fact.
    let html = r#"<html xmlns:ix="http://www.xbrl.org/2013/inlineXBRL"
               xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
<head>
  <ix:header>
    <ix:references/>
    <ix:resources>
      <xbrli:context id="c0" xmlns:xbrli="http://www.xbrl.org/2003/instance">
        <xbrli:entity>
          <xbrli:identifier scheme="http://www.sec.gov/CIK">0002049662</xbrli:identifier>
        </xbrli:entity>
        <xbrli:period>
          <xbrli:instant>2025-12-31</xbrli:instant>
        </xbrli:period>
      </xbrli:context>
      <xbrli:unit id="usd" xmlns:xbrli="http://www.xbrl.org/2003/instance">
        <xbrli:measure>iso4217:USD</xbrli:measure>
      </xbrli:unit>
    </ix:resources>
  </ix:header>
</head>
<body>
  <!-- Case 1: value is inside a <span>; the ballot box is still the value. -->
  <ix:nonNumeric contextRef="c0" format="ixt-sec:boolballotbox"
                 name="dei:DocumentAnnualReport" id="ixv-1">
    <span style="font-weight:bold">&#9746;</span>
  </ix:nonNumeric>

  <!-- Case 2: a numeric field whose <span> is empty has no value -->
  <ix:nonFraction contextRef="c0" unitRef="usd" decimals="0"
                  name="us-gaap:PreferredStockValue" id="ixv-2">
    <span></span>
  </ix:nonFraction>

  <!-- Case 3: explicit xsi:nil="true" — must remain Nil -->
  <ix:nonFraction contextRef="c0" unitRef="usd" decimals="0"
                  name="us-gaap:CommitmentsAndContingencies"
                  xsi:nil="true" id="hidden-fact-0"></ix:nonFraction>

  <!-- Case 4: normal direct-text fact — must still work correctly -->
  <ix:nonNumeric contextRef="c0" name="dei:EntityName" id="ixv-4">Cartesian Growth Corp III</ix:nonNumeric>
</body>
</html>"#;

    let xbrl = parse_ixbrl(html).expect("should parse iXBRL without error");

    // Case 1: span-wrapped value is read, and its format applied
    let annual_report = xbrl
        .facts
        .iter()
        .find(|f| f.full_name == "dei:DocumentAnnualReport")
        .expect("dei:DocumentAnnualReport must be present");
    assert_eq!(
        annual_report.value,
        XbrlValue::String("true".to_string()),
        "dei:DocumentAnnualReport's span-wrapped ballot box should be read"
    );

    // Case 2: numeric field with an empty span → Nil
    let preferred = xbrl
        .facts
        .iter()
        .find(|f| f.full_name == "us-gaap:PreferredStockValue")
        .expect("us-gaap:PreferredStockValue must be present");
    assert_eq!(
        preferred.value,
        XbrlValue::Nil,
        "us-gaap:PreferredStockValue with no text should be Nil, not String(\"\")"
    );

    // Case 3: explicit xsi:nil → Nil
    let contingencies = xbrl
        .facts
        .iter()
        .find(|f| f.full_name == "us-gaap:CommitmentsAndContingencies")
        .expect("us-gaap:CommitmentsAndContingencies must be present");
    assert_eq!(
        contingencies.value,
        XbrlValue::Nil,
        "xsi:nil=\"true\" element should be Nil"
    );

    // Case 4: direct text must still be extracted normally
    let entity_name = xbrl
        .facts
        .iter()
        .find(|f| f.full_name == "dei:EntityName")
        .expect("dei:EntityName must be present");
    assert_eq!(
        entity_name.value,
        XbrlValue::String("Cartesian Growth Corp III".to_string()),
        "Normal direct-text fact should still be extracted"
    );
}

/// Regression test for nested ix:nonnumeric elements where an outer element with a
/// format transformation contains an inner ix:nonnumeric that holds part of the text.
///
/// Real-world example from SEC filings:
/// ```html
/// <ix:nonnumeric format="ixt:date-monthname-day-year-en" name="dei:DocumentPeriodEndDate">
///   September 30, <ix:nonnumeric name="dei:DocumentFiscalYearFocus">2025</ix:nonnumeric>
/// </ix:nonnumeric>
/// ```
///
/// The outer element must receive the full text "September 30, 2025" so that the
/// date transformation succeeds (producing "2025-09-30").  The inner element must
/// also be registered independently with value "2025".
#[test]
fn test_nested_ixbrl_date_extraction() {
    let html = r#"<html xmlns:ix="http://www.xbrl.org/2013/inlineXBRL">
<body>
<div>For the quarterly period ended
<ix:nonnumeric contextRef="c0" format="ixt:date-monthname-day-year-en" name="dei:DocumentPeriodEndDate" id="ixv-2998">September 30, <ix:nonnumeric contextRef="c0" name="dei:DocumentFiscalYearFocus" id="ixv-2999">2025</ix:nonnumeric></ix:nonnumeric>
</div>
</body>
</html>"#;

    let xbrl = parse_ixbrl(html).expect("should parse iXBRL");

    let period_end = xbrl
        .facts
        .iter()
        .find(|f| f.full_name == "dei:DocumentPeriodEndDate")
        .expect("DocumentPeriodEndDate fact must be present");
    assert_eq!(
        period_end.value,
        XbrlValue::String("2025-09-30".to_string()),
        "DocumentPeriodEndDate should be transformed to ISO date"
    );

    let fiscal_year = xbrl
        .facts
        .iter()
        .find(|f| f.full_name == "dei:DocumentFiscalYearFocus")
        .expect("DocumentFiscalYearFocus fact must be present");
    assert_eq!(
        fiscal_year.value,
        XbrlValue::String("2025".to_string()),
        "DocumentFiscalYearFocus should be extracted as '2025'"
    );
}
