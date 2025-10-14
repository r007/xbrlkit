//! # Comprehensive iXBRL Transformation Tests
//!
//! This test suite validates the transformation layer's ability to handle real SEC filing data
//! with various format attributes. Tests cover both unit transformations (individual functions)
//! and integration scenarios (parsing complete iXBRL documents).
//!
//! ## Fixtures Used
//!
//! We test against 6 real SEC filing fixtures covering different filing types and companies:
//! - **10-Q filings** (4 fixtures): Quarterly reports from different SPACs
//!   - 10-q.html: EQV Ventures Acquisition Corp. 2
//!   - 10-q_1.html: Social Capital Hedosophia Holdings Corp. IV
//!   - 10-q_2.html: Dune Acquisition Corporation
//!   - 10-q_3.html: Another SPAC quarterly filing
//! - **8-K filings** (2 fixtures): Current reports for material events
//!   - 8-k.html: Alchemy Investments Acquisition Corp 1
//!   - 8-k_1.html: BOWX Acquisition Corp.
//!
//! ## Test Coverage
//!
//! 1. **Unit Transformations**: Individual transformation function tests
//! 2. **Integration Tests**: Real document parsing with transformation application
//! 3. **Edge Cases**: Whitespace, case sensitivity, error handling
//! 4. **Cross-Fixture Validation**: Ensuring consistency across different filings

use std::collections::HashMap;
use transformations::{
    apply_transformation, bool_ballot_box, dur_words_en, num_dot_decimal, num_words_en,
};
use xbrl::{from_ixbrl_str, structures::XbrlValue, transformations};

// ============================================================================
// FIXTURE CONSTANTS
// ============================================================================

/// 10-Q filing from EQV Ventures Acquisition Corp. 2 (Q2 2025)
const FIXTURE_10Q: &str = include_str!("../../fixtures/html/10-q.html");

/// 10-Q filing from Social Capital Hedosophia Holdings Corp. IV
const FIXTURE_10Q_1: &str = include_str!("../../fixtures/html/10-q_1.html");

/// 10-Q filing from Dune Acquisition Corporation (Q2 2025)
const FIXTURE_10Q_2: &str = include_str!("../../fixtures/html/10-q_2.html");

/// 10-Q filing (variant 3)
const FIXTURE_10Q_3: &str = include_str!("../../fixtures/html/10-q_3.html");

/// 8-K filing from Alchemy Investments Acquisition Corp 1 (Aug 2025)
const FIXTURE_8K: &str = include_str!("../../fixtures/html/8-k.html");

/// 8-K filing from BOWX Acquisition Corp. (Nov 2021)
const FIXTURE_8K_1: &str = include_str!("../../fixtures/html/8-k_1.html");

// ============================================================================
// UNIT TRANSFORMATION TESTS
// ============================================================================

// ============================================================================
// UNIT TRANSFORMATION TESTS
// ============================================================================

/// Tests the `boolballotbox` transformation with checkbox Unicode characters.
/// Used in SEC filings for yes/no questions (e.g., "Quarterly Report" checkboxes).
#[test]
fn test_bool_ballot_box_transformation() {
    // Empty checkbox (☐ U+2610) -> false
    assert_eq!(
        bool_ballot_box("☐").unwrap(),
        "false",
        "Empty checkbox (U+2610) should be false"
    );

    // Checked checkbox (☑ U+2611) -> true
    assert_eq!(
        bool_ballot_box("☑").unwrap(),
        "true",
        "Checked checkbox (U+2611) should be true"
    );

    // X-marked checkbox (☒ U+2612) -> true
    assert_eq!(
        bool_ballot_box("☒").unwrap(),
        "true",
        "X-marked checkbox (U+2612) should be true"
    );
}

/// Tests the `numwordsen` transformation converting English number words to digits.
/// Supports numbers from zero to trillions, handles "and", commas, and hyphens.
#[test]
fn test_num_words_en_transformation() {
    // Simple single-word numbers
    assert_eq!(num_words_en("zero").unwrap(), "0");
    assert_eq!(num_words_en("one").unwrap(), "1");
    assert_eq!(num_words_en("twelve").unwrap(), "12");
    assert_eq!(num_words_en("ninety").unwrap(), "90");

    // Compound numbers with "and"
    assert_eq!(num_words_en("seventy thousand and one").unwrap(), "70001");

    // Compound numbers with hyphens (common in formal writing)
    assert_eq!(num_words_en("nineteen hundred forty-four").unwrap(), "1944");

    // Large numbers (millions and billions)
    assert_eq!(
        num_words_en("six million four hundred thousand five").unwrap(),
        "6400005"
    );
    assert_eq!(num_words_en("one billion").unwrap(), "1000000000");

    // Edge cases: "no", "none", "nil" all map to zero
    assert_eq!(num_words_en("no").unwrap(), "0");
    assert_eq!(num_words_en("none").unwrap(), "0");
    assert_eq!(num_words_en("nil").unwrap(), "0");
}

/// Tests the `durwordsen` transformation converting duration phrases to ISO 8601.
/// Supports years, months, and days expressed in words or numbers.
#[test]
fn test_dur_words_en_transformation() {
    // Simple duration with numeric values
    assert_eq!(
        dur_words_en("9 years, 2 months").unwrap(),
        "P9Y2M",
        "Numeric years and months with comma"
    );

    // Duration with word numbers
    assert_eq!(
        dur_words_en("Five years, two months").unwrap(),
        "P5Y2M",
        "Word-based years and months"
    );

    // Complex duration with "no" for zero days
    assert_eq!(
        dur_words_en("three years four months no days").unwrap(),
        "P3Y4M",
        "Complex phrase with 'no days' (excluded from output)"
    );

    // Single unit duration
    assert_eq!(
        dur_words_en("1 year").unwrap(),
        "P1Y",
        "Single year duration"
    );

    // Duration with "and" conjunction
    assert_eq!(
        dur_words_en("5 years and 3 months").unwrap(),
        "P5Y3M",
        "Duration with 'and' conjunction"
    );
}

/// Tests the `num-dot-decimal` transformation removing thousand separators.
/// Most common transformation in SEC filings (1,262 uses across our fixtures).
#[test]
fn test_num_dot_decimal_transformation() {
    // Large numbers with comma separators
    assert_eq!(
        num_dot_decimal("1,000,000").unwrap(),
        "1000000",
        "Should remove all comma separators"
    );

    // Numbers with both commas and decimal points
    assert_eq!(
        num_dot_decimal("123,456.789").unwrap(),
        "123456.789",
        "Should remove commas but preserve decimal point"
    );

    // Numbers without commas (pass through)
    assert_eq!(
        num_dot_decimal("42").unwrap(),
        "42",
        "Numbers without commas should pass through unchanged"
    );
}

/// Tests the transformation registry dispatch system.
/// The registry maps transformation names (e.g., "ixt-sec:numwordsen") to functions.
#[test]
fn test_apply_transformation() {
    // SEC-specific transformations (ixt-sec namespace)
    assert_eq!(
        apply_transformation("one", "ixt-sec:numwordsen").unwrap(),
        "1",
        "Number words transformation"
    );
    assert_eq!(
        apply_transformation("☐", "ixt-sec:boolballotbox").unwrap(),
        "false",
        "Ballot box transformation"
    );
    assert_eq!(
        apply_transformation("5 years", "ixt-sec:durwordsen").unwrap(),
        "P5Y",
        "Duration words transformation"
    );

    // Standard iXBRL transformations (ixt namespace)
    assert_eq!(
        apply_transformation("1,000", "ixt:num-dot-decimal").unwrap(),
        "1000",
        "Numeric comma removal"
    );

    // Unknown transformation (should pass through unchanged)
    assert_eq!(
        apply_transformation("value", "unknown:transform").unwrap(),
        "value",
        "Unknown transformations pass through"
    );

    // Fixed value transformations (always return same value regardless of input)
    assert_eq!(
        apply_transformation("anything", "ixt:fixed-true").unwrap(),
        "true"
    );
    assert_eq!(
        apply_transformation("anything", "ixt:fixed-false").unwrap(),
        "false"
    );
    assert_eq!(
        apply_transformation("anything", "ixt:fixed-zero").unwrap(),
        "0"
    );
}

// ============================================================================
// INTEGRATION TESTS - MULTI-FIXTURE VALIDATION
// ============================================================================

/// Tests transformation extraction across all 6 fixtures.
/// Validates that transformations are correctly applied during document parsing.
#[test]
fn test_transformations_across_all_fixtures() {
    let fixtures = vec![
        ("10-q.html", FIXTURE_10Q),
        ("10-q_1.html", FIXTURE_10Q_1),
        ("10-q_2.html", FIXTURE_10Q_2),
        ("10-q_3.html", FIXTURE_10Q_3),
        ("8-k.html", FIXTURE_8K),
        ("8-k_1.html", FIXTURE_8K_1),
    ];

    for (name, content) in fixtures {
        println!("\n=== Testing transformations in {} ===", name);

        let result = from_ixbrl_str(content);
        assert!(
            result.is_ok(),
            "Failed to parse {}: {:?}",
            name,
            result.err()
        );

        let context = result.unwrap();
        let xbrl = &context.xbrl;

        // All fixtures should have some facts
        assert!(!xbrl.facts.is_empty(), "{} should contain facts", name);

        // Find facts with format attributes (transformations)
        let facts_with_format: Vec<_> = xbrl.facts.iter().filter(|f| f.format.is_some()).collect();

        println!("  Total facts: {}", xbrl.facts.len());
        println!("  Facts with transformations: {}", facts_with_format.len());

        // Count transformation types
        let mut transformation_counts = HashMap::new();
        for fact in &facts_with_format {
            if let Some(fmt) = &fact.format {
                *transformation_counts.entry(fmt.as_str()).or_insert(0) += 1;
            }
        }

        // Print transformation distribution
        let mut sorted_transformations: Vec<_> = transformation_counts.iter().collect();
        sorted_transformations.sort_by(|a, b| b.1.cmp(a.1));

        for (transform, count) in sorted_transformations.iter().take(5) {
            println!("    {}: {} uses", transform, count);
        }

        // At least 10-Q filings should have format attributes
        if name.starts_with("10-q") {
            assert!(
                !facts_with_format.is_empty(),
                "{} should have facts with format attributes",
                name
            );
        }
    }
}

/// Tests specific transformation types found in real SEC filings.
/// Focuses on the most commonly used transformations.
#[test]
fn test_common_transformations_in_fixtures() {
    // Test on the most comprehensive fixture (10-q_2.html)
    let result = from_ixbrl_str(FIXTURE_10Q_2);
    assert!(result.is_ok(), "Failed to parse 10-q_2.html");

    let context = result.unwrap();
    let xbrl = &context.xbrl;

    // Find ballot box transformations (used for yes/no questions)
    let ballot_box_facts: Vec<_> = xbrl
        .facts
        .iter()
        .filter(|f| {
            f.format
                .as_ref()
                .map_or(false, |fmt| fmt.contains("boolballotbox"))
        })
        .collect();

    println!("\n=== Ballot Box Transformations ===");
    println!("Found {} ballot box facts", ballot_box_facts.len());
    for fact in ballot_box_facts.iter().take(3) {
        let value_str = match &fact.value {
            XbrlValue::String(s) => s.as_str(),
            XbrlValue::Nil => "nil",
        };
        println!(
            "  - {}: {} (format: {:?})",
            fact.local_name, value_str, fact.format
        );
    }

    // Find numeric transformations (most common: num-dot-decimal)
    let num_decimal_facts: Vec<_> = xbrl
        .facts
        .iter()
        .filter(|f| {
            f.format.as_ref().map_or(false, |fmt| {
                fmt.contains("num-dot-decimal") || fmt.contains("numdotdecimal")
            })
        })
        .collect();

    println!("\n=== Numeric Transformations (comma removal) ===");
    println!("Found {} num-dot-decimal facts", num_decimal_facts.len());
    for fact in num_decimal_facts.iter().take(3) {
        let value_str = match &fact.value {
            XbrlValue::String(s) => s.as_str(),
            XbrlValue::Nil => "nil",
        };
        println!(
            "  - {}: {} (format: {:?})",
            fact.local_name, value_str, fact.format
        );
    }

    // Find exchange name transformations
    let exchange_facts: Vec<_> = xbrl
        .facts
        .iter()
        .filter(|f| {
            f.format
                .as_ref()
                .map_or(false, |fmt| fmt.contains("exchnameen"))
        })
        .collect();

    if !exchange_facts.is_empty() {
        println!("\n=== Exchange Name Transformations ===");
        println!("Found {} exchange name facts", exchange_facts.len());
        for fact in exchange_facts.iter() {
            let value_str = match &fact.value {
                XbrlValue::String(s) => s.as_str(),
                XbrlValue::Nil => "nil",
            };
            println!(
                "  - {}: {} (format: {:?})",
                fact.local_name, value_str, fact.format
            );
        }
    }
}

/// Tests that transformations handle edge cases in 8-K filings.
/// 8-K filings are event-driven and have different fact patterns than 10-Q.
#[test]
fn test_transformations_in_8k_filings() {
    let fixtures_8k = vec![("8-k.html", FIXTURE_8K), ("8-k_1.html", FIXTURE_8K_1)];

    for (name, content) in fixtures_8k {
        println!("\n=== Testing 8-K: {} ===", name);

        let result = from_ixbrl_str(content);
        assert!(result.is_ok(), "Failed to parse {}", name);

        let context = result.unwrap();
        let xbrl = &context.xbrl;

        // 8-K should have DocumentType fact
        let doc_type = xbrl.facts.iter().find(|f| f.local_name == "DocumentType");

        assert!(doc_type.is_some(), "{} should have DocumentType", name);

        // Check for date transformations (common in 8-K for event dates)
        let date_facts: Vec<_> = xbrl
            .facts
            .iter()
            .filter(|f| {
                f.format
                    .as_ref()
                    .map_or(false, |fmt| fmt.contains("date") && !fmt.contains("fixed"))
            })
            .collect();

        println!("  Date transformation facts: {}", date_facts.len());
        for fact in date_facts.iter().take(3) {
            println!("    {}: format={:?}", fact.local_name, fact.format);
        }
    }
}

// ============================================================================
// EDGE CASE & ERROR HANDLING TESTS
// ============================================================================

/// Tests that transformations handle invalid inputs gracefully.
#[test]
fn test_transformation_error_handling() {
    // Invalid ballot box character
    let result = bool_ballot_box("invalid");
    assert!(result.is_err(), "Should fail on invalid checkbox character");

    // Invalid number words
    let result = num_words_en("not a number word");
    assert!(
        result.is_err() || result.unwrap() == "0",
        "Should handle invalid number words gracefully"
    );

    // Invalid duration format
    let result = dur_words_en("invalid duration");
    assert!(
        (result.is_ok() && result.as_ref().unwrap() == "P0D") || result.is_err(),
        "Should handle invalid duration gracefully"
    );
}

/// Tests that transformations correctly trim whitespace from inputs.
/// Real SEC filings often have extra whitespace around values.
#[test]
fn test_transformation_whitespace_handling() {
    // Ballot box with surrounding whitespace
    assert_eq!(
        bool_ballot_box(" ☐ ").unwrap(),
        "false",
        "Should trim whitespace around checkbox"
    );

    // Number words with extra spaces
    assert_eq!(
        num_words_en("  one  ").unwrap(),
        "1",
        "Should trim whitespace from number words"
    );

    // Duration with leading/trailing spaces
    assert_eq!(
        dur_words_en("  5 years  ").unwrap(),
        "P5Y",
        "Should trim whitespace from duration"
    );
}

/// Tests that word-based transformations are case-insensitive.
/// SEC filings use various capitalization styles.
#[test]
fn test_transformation_case_insensitivity() {
    // Uppercase number words
    assert_eq!(num_words_en("ONE").unwrap(), "1", "Should handle uppercase");

    // Mixed case number words
    assert_eq!(
        num_words_en("Twelve").unwrap(),
        "12",
        "Should handle mixed case"
    );

    // Complex uppercase phrase
    assert_eq!(
        num_words_en("SEVENTY THOUSAND").unwrap(),
        "70000",
        "Should handle complex uppercase"
    );
}

// ============================================================================
// COMPREHENSIVE TRANSFORMATION REGISTRY TESTS
// ============================================================================

/// Tests all SEC-specific and standard transformations.
/// Covers the complete transformation registry including new additions.
#[test]
fn test_comprehensive_transformation_registry() {
    println!("\n=== Testing SEC-Specific Transformations (ixt-sec) ===");

    // Exchange name normalization with regex patterns
    assert_eq!(
        apply_transformation("New York Stock Exchange", "ixt-sec:exchnameen").unwrap(),
        "NYSE",
        "Full exchange name"
    );
    assert_eq!(
        apply_transformation("The NYSE", "ixt-sec:exchnameen").unwrap(),
        "NYSE",
        "Short form with 'The'"
    );
    assert_eq!(
        apply_transformation("The NASDAQ Stock Market, LLC", "ixt-sec:exchnameen").unwrap(),
        "NASDAQ",
        "NASDAQ with legal entity suffix"
    );
    assert_eq!(
        apply_transformation("Cboe BZX Exchange, Inc.", "ixt-sec:exchnameen").unwrap(),
        "CboeBZX",
        "Cboe exchange variant"
    );

    println!("\n=== Testing Date Transformations (ixt) ===");

    // Month-day-year format (American style)
    assert_eq!(
        apply_transformation("August 22, 2025", "ixt:datemonthdayyearen").unwrap(),
        "2025-08-22",
        "American date format"
    );
    assert_eq!(
        apply_transformation("November 29, 2021", "ixt:date-monthname-day-year-en").unwrap(),
        "2021-11-29",
        "Date with hyphenated transformation name"
    );

    println!("\n=== Testing State/Province Normalization (ixt-sec) ===");

    // US states
    assert_eq!(
        apply_transformation("Utah", "ixt-sec:stateprovnameen").unwrap(),
        "UT",
        "US state"
    );
    assert_eq!(
        apply_transformation("Delaware", "ixt-sec:stateprovnameen").unwrap(),
        "DE",
        "Most common incorporation state"
    );

    // Canadian provinces
    assert_eq!(
        apply_transformation("British Columbia", "ixt-sec:stateprovnameen").unwrap(),
        "BC",
        "Canadian province"
    );

    // US territories
    assert_eq!(
        apply_transformation("Puerto Rico", "ixt-sec:stateprovnameen").unwrap(),
        "PR",
        "US territory"
    );

    println!("\n=== Testing Boolean Transformations (ixt) ===");

    // Fixed boolean values (ignore input)
    assert_eq!(
        apply_transformation("anything", "ixt:booleanfalse").unwrap(),
        "false"
    );
    assert_eq!(
        apply_transformation("anything", "ixt:booleantrue").unwrap(),
        "true"
    );

    println!("\n=== Testing Numeric Transformation Variants (ixt) ===");

    // Spelling variant (numdotdecimal vs num-dot-decimal)
    assert_eq!(
        apply_transformation("1,234.56", "ixt:numdotdecimal").unwrap(),
        "1234.56",
        "No-hyphen variant"
    );

    println!("\n=== Testing Duration Transformations (ixt-sec) ===");

    // Day durations (with decimal support)
    assert_eq!(
        apply_transformation("30", "ixt-sec:durday").unwrap(),
        "P30D",
        "Integer days"
    );
    assert_eq!(
        apply_transformation("30.5", "ixt-sec:durday").unwrap(),
        "P30DT12H",
        "Decimal days (half day = 12 hours)"
    );

    // Month durations (with decimal support)
    assert_eq!(
        apply_transformation("12", "ixt-sec:durmonth").unwrap(),
        "P12M",
        "Integer months"
    );
    assert_eq!(
        apply_transformation("12.5", "ixt-sec:durmonth").unwrap(),
        "P12M15D",
        "Decimal months (half month ≈ 15 days)"
    );

    // Year durations
    assert_eq!(
        apply_transformation("1", "ixt-sec:duryear").unwrap(),
        "P1Y",
        "Single year"
    );

    // Week durations
    assert_eq!(
        apply_transformation("2", "ixt-sec:durweek").unwrap(),
        "P14D",
        "Two weeks = 14 days"
    );

    // Hour durations
    assert_eq!(
        apply_transformation("24", "ixt-sec:durhour").unwrap(),
        "PT24H",
        "24 hours"
    );

    println!("\n=== Testing Entity Filer Category (ixt-sec) ===");

    // Entity filer categories (pass through with normalization)
    assert_eq!(
        apply_transformation("Large Accelerated Filer", "ixt-sec:entityfilercategoryen").unwrap(),
        "Large Accelerated Filer"
    );
    assert_eq!(
        apply_transformation("Non-accelerated Filer", "ixt-sec:entityfilercategoryen").unwrap(),
        "Non-accelerated Filer"
    );
    assert_eq!(
        apply_transformation("Accelerated Filer", "ixt-sec:entityfilercategoryen").unwrap(),
        "Accelerated Filer"
    );

    println!("\n✅ All transformation registry tests passed!");
}
