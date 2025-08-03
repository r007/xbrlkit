use serde_json::to_string;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::time::{Duration, Instant};
use xbrl::taxonomies::{
    dei::{DeiInfo, extract_dei},
    us_gaap::{Financials, extract_financials},
};

// Test fixture path constants
const FORM_10Q_FIXTURE: &str = "../fixtures/filings/form_10q.xml";
const FORM_10Q_1_FIXTURE: &str = "../fixtures/filings/form_10q_1.xml";

/// Benchmarks the complete XBRL parsing workflow using the new serde-based approach.
#[test]
fn benchmark_complete_serde_workflow() {
    let content =
        read_to_string(FORM_10Q_1_FIXTURE).expect("Failed to read form_10q_1.xml fixture");

    let mut parse_times = Vec::new();
    let mut dei_times = Vec::new();
    let mut gaap_times = Vec::new();
    let mut total_times = Vec::new();

    // Run multiple iterations to get stable timing
    for _ in 0..20 {
        let total_start = Instant::now();

        // Step 1: Parse once
        let parse_start = Instant::now();
        let context = xbrl::from_str(&content).expect("XBRL parsing should succeed");
        parse_times.push(parse_start.elapsed());

        // Step 2: Extract multiple times from the same context
        let dei_start = Instant::now();
        let _dei_info = extract_dei(&context).expect("DEI extraction should succeed");
        dei_times.push(dei_start.elapsed());

        let gaap_start = Instant::now();
        let _financials =
            extract_financials(&context).expect("Financial extraction should succeed");
        gaap_times.push(gaap_start.elapsed());

        total_times.push(total_start.elapsed());
    }

    let avg_parse = parse_times.iter().sum::<Duration>() / parse_times.len() as u32;
    let avg_dei = dei_times.iter().sum::<Duration>() / dei_times.len() as u32;
    let avg_gaap = gaap_times.iter().sum::<Duration>() / gaap_times.len() as u32;
    let avg_total = total_times.iter().sum::<Duration>() / total_times.len() as u32;

    println!("Complete serde workflow benchmark results (new architecture):");
    println!("  Parsing/Indexing: {:?} (avg)", avg_parse);
    println!("  DEI extraction:   {:?} (avg)", avg_dei);
    println!("  US-GAAP extraction: {:?} (avg)", avg_gaap);
    println!("  Total workflow:   {:?} (avg)", avg_total);
    println!("  Document size:    {} bytes", content.len());

    // Performance ratio analysis
    let total_extraction_nanos = (avg_dei + avg_gaap).as_nanos();
    println!("\nPerformance breakdown:");
    println!(
        "  Parsing:   {:.1}%",
        (avg_parse.as_nanos() as f64 / avg_total.as_nanos() as f64) * 100.0
    );
    println!(
        "  Extraction: {:.1}%",
        (total_extraction_nanos as f64 / avg_total.as_nanos() as f64) * 100.0
    );

    // Performance assertions - should complete extraction quickly
    assert!(
        avg_total.as_millis() < 2000,
        "Total extraction should be under 2 seconds, got {:?}",
        avg_total
    );
}

/// Benchmarks the raw XBRL parsing and indexing (creation of XbrlDataContext).
#[test]
fn benchmark_raw_xbrl_parsing_and_indexing() {
    let content =
        read_to_string(FORM_10Q_1_FIXTURE).expect("Failed to read form_10q_1.xml fixture");

    let mut times = Vec::new();

    // Run multiple iterations to get stable timing
    for _ in 0..15 {
        let start = Instant::now();
        let _context = xbrl::from_str(&content).expect("Should parse XBRL successfully");
        times.push(start.elapsed());
    }

    let avg_time = times.iter().sum::<Duration>() / times.len() as u32;
    let min_time = times.iter().min().unwrap();
    let max_time = times.iter().max().unwrap();

    println!("Raw XBRL parsing & indexing benchmark results:");
    println!("  Average: {:?}", avg_time);
    println!("  Min:     {:?}", min_time);
    println!("  Max:     {:?}", max_time);
    println!("  Document size: {} bytes", content.len());

    // Performance assertion - raw parsing should be very fast
    assert!(
        avg_time.as_millis() < 500,
        "Raw XBRL parsing & indexing should be under 500ms, got {:?}",
        avg_time
    );
}

/// Benchmarks serde deserialization performance with different struct sizes.
#[test]
fn benchmark_serde_deserialization_scaling() {
    let content =
        read_to_string(FORM_10Q_1_FIXTURE).expect("Failed to read form_10q_1.xml fixture");
    let context = xbrl::from_str(&content).expect("Parse once");

    let taxonomies: Vec<(&str, Box<dyn Fn() -> xbrl::error::Result<()>>)> = vec![
        (
            "DEI (Document Info)",
            Box::new(|| extract_dei(&context).map(|_| ())),
        ),
        (
            "US-GAAP (Financial)",
            Box::new(|| extract_financials(&context).map(|_| ())),
        ),
    ];

    for (name, extract_fn) in taxonomies {
        let mut times = Vec::new();

        for _ in 0..25 {
            let start = Instant::now();
            let _result = extract_fn().expect("Extraction should succeed");
            times.push(start.elapsed());
        }

        let avg_time = times.iter().sum::<Duration>() / times.len() as u32;
        let min_time = times.iter().min().unwrap();

        println!("Serde deserialization benchmark - {}:", name);
        println!("  Average: {:?}", avg_time);
        println!("  Min:     {:?}", min_time);

        // Should be very fast (under 1 second even for complex structs)
        assert!(
            avg_time.as_millis() < 1000,
            "Serde deserialization should be under 1s for {}, got {:?}",
            name,
            avg_time
        );
    }
}

/// Benchmarks memory usage and extraction efficiency.
#[test]
fn benchmark_extraction_efficiency() {
    let content =
        read_to_string(FORM_10Q_1_FIXTURE).expect("Failed to read form_10q_1.xml fixture");

    // Parse raw XBRL to understand the data volume
    let context = xbrl::from_str(&content).unwrap();

    let context_count = context.xbrl.contexts.len();
    let unit_count = context.xbrl.units.len();
    let fact_count = context.xbrl.facts.len();

    println!("XBRL document characteristics:");
    println!("  Document size: {} bytes", content.len());
    println!("  Contexts: {}", context_count);
    println!("  Units: {}", unit_count);
    println!("  Facts: {}", fact_count);

    // Test extraction efficiency
    let start = Instant::now();
    let dei_info = extract_dei(&context).unwrap();
    let dei_time = start.elapsed();

    let start = Instant::now();
    let financials = extract_financials(&context).unwrap();
    let financials_time = start.elapsed();

    // Count extracted fields
    let dei_field_count = count_dei_fields(&dei_info);
    let financial_field_count = count_financial_fields(&financials);

    println!("\nExtraction efficiency:");
    println!(
        "  DEI: {} fields extracted in {:?} ({:.2} fields/ms)",
        dei_field_count,
        dei_time,
        dei_field_count as f64 / dei_time.as_millis().max(1) as f64
    );
    println!(
        "  Financials: {} fields extracted in {:?} ({:.2} fields/ms)",
        financial_field_count,
        financials_time,
        financial_field_count as f64 / financials_time.as_millis().max(1) as f64
    );

    let total_extracted = dei_field_count + financial_field_count;
    let total_time = dei_time + financials_time;

    println!(
        "  Total: {} fields from {} facts in {:?} ({:.1}% utilization)",
        total_extracted,
        fact_count,
        total_time,
        (total_extracted as f64 / fact_count as f64) * 100.0
    );

    // Efficiency assertions
    assert!(
        total_extracted > 10,
        "Should extract a reasonable number of fields"
    );
    assert!(
        total_time.as_millis() < 3000,
        "Total extraction should be under 3 seconds"
    );
}

/// Benchmarks performance across different document sizes.
#[test]
fn benchmark_document_size_scaling() {
    let files = [
        (FORM_10Q_FIXTURE, "Form 10-Q (earlier)"),
        (FORM_10Q_1_FIXTURE, "Form 10-Q (later)"),
    ];

    for (filename, description) in &files {
        let content = match read_to_string(filename) {
            Ok(content) => content,
            Err(_) => {
                println!("Skipping {} - file not found", filename);
                continue;
            }
        };

        let mut times = Vec::new();

        for _ in 0..10 {
            let start = Instant::now();
            let context = xbrl::from_str(&content).unwrap();
            let _dei = extract_dei(&context).unwrap();
            let _financials = extract_financials(&context).unwrap();
            times.push(start.elapsed());
        }

        let avg_time = times.iter().sum::<Duration>() / times.len() as u32;
        let doc_size_kb = content.len() / 1024;
        let throughput_mb_per_sec =
            (content.len() as f64 / 1024.0 / 1024.0) / avg_time.as_secs_f64();

        println!("Document scaling benchmark - {}:", description);
        println!("  Size: {} KB", doc_size_kb);
        println!("  Average extraction time: {:?}", avg_time);
        println!("  Throughput: {:.2} MB/s", throughput_mb_per_sec);
        println!(
            "  Time per KB: {:.2} μs/KB",
            avg_time.as_micros() as f64 / doc_size_kb as f64
        );
    }
}

/// Benchmarks JSON serialization performance of extracted data.
#[test]
fn benchmark_serialization_performance() {
    let content =
        read_to_string(FORM_10Q_1_FIXTURE).expect("Failed to read form_10q_1.xml fixture");
    let context = xbrl::from_str(&content).unwrap();

    // Extract data first
    let dei_info = extract_dei(&context).unwrap();
    let financials = extract_financials(&context).unwrap();

    // Benchmark JSON serialization
    let mut dei_serialize_times = Vec::new();
    let mut financials_serialize_times = Vec::new();

    for _ in 0..50 {
        // DEI serialization
        let start = Instant::now();
        let _json = to_string(&dei_info).unwrap();
        dei_serialize_times.push(start.elapsed());

        // Financials serialization
        let start = Instant::now();
        let _json = to_string(&financials).unwrap();
        financials_serialize_times.push(start.elapsed());
    }

    let avg_dei_serialize =
        dei_serialize_times.iter().sum::<Duration>() / dei_serialize_times.len() as u32;
    let avg_financials_serialize = financials_serialize_times.iter().sum::<Duration>()
        / financials_serialize_times.len() as u32;

    // Measure JSON sizes
    let dei_json = to_string(&dei_info).unwrap();
    let financials_json = to_string(&financials).unwrap();

    println!("JSON serialization benchmark:");
    println!(
        "  DEI:        {:?} (avg) for {} bytes",
        avg_dei_serialize,
        dei_json.len()
    );
    println!(
        "  Financials: {:?} (avg) for {} bytes",
        avg_financials_serialize,
        financials_json.len()
    );

    // Performance assertions - JSON serialization should be very fast
    assert!(
        avg_dei_serialize.as_millis() < 50,
        "DEI JSON serialization should be under 50ms"
    );
    assert!(
        avg_financials_serialize.as_millis() < 100,
        "Financials JSON serialization should be under 100ms"
    );
}

/// Benchmarks fact selection logic performance.
#[test]
fn benchmark_fact_selection_performance() {
    let content =
        read_to_string(FORM_10Q_1_FIXTURE).expect("Failed to read form_10q_1.xml fixture");
    let context = xbrl::from_str(&content).unwrap();

    // Count facts with multiple contexts (where selection logic matters)
    let mut multi_context_concepts = 0;
    let mut concept_counts = HashMap::new();

    for fact in &context.xbrl.facts {
        *concept_counts.entry(&fact.full_name).or_insert(0) += 1;
    }

    for count in concept_counts.values() {
        if *count > 1 {
            multi_context_concepts += 1;
        }
    }

    println!("Fact selection complexity analysis:");
    println!("  Total facts: {}", context.xbrl.facts.len());
    println!("  Unique concepts: {}", concept_counts.len());
    println!("  Multi-context concepts: {}", multi_context_concepts);
    println!("  Contexts: {}", context.xbrl.contexts.len());

    // Benchmark the extraction which includes fact selection
    let mut times = Vec::new();
    for _ in 0..20 {
        let start = Instant::now();
        let _financials = extract_financials(&context).unwrap();
        times.push(start.elapsed());
    }

    let avg_time = times.iter().sum::<Duration>() / times.len() as u32;

    println!("  Fact selection performance: {:?} avg", avg_time);
    println!(
        "  Time per concept: {:.2} μs/concept",
        avg_time.as_micros() as f64 / concept_counts.len() as f64
    );

    // The fact selection should not significantly impact performance
    assert!(
        avg_time.as_millis() < 1000,
        "Fact selection should be efficient"
    );
}

// Helper functions for counting extracted fields

fn count_dei_fields(dei: &DeiInfo) -> usize {
    let mut count = 0;

    // Count document fields
    if dei.document.document_type.is_some() {
        count += 1;
    }
    if dei.document.document_fiscal_period_focus.is_some() {
        count += 1;
    }
    if dei.document.amendment_flag.is_some() {
        count += 1;
    }

    // Count entity fields
    if dei.entity.entity_central_index_key.is_some() {
        count += 1;
    }
    if dei.entity.entity_registrant_name.is_some() {
        count += 1;
    }
    if dei.entity.trading_symbol.is_some() {
        count += 1;
    }
    if dei.entity.current_fiscal_year_end_date.is_some() {
        count += 1;
    }

    // Add more field checks as needed...
    count
}

fn count_financial_fields(financials: &Financials) -> usize {
    let mut count = 0;

    // Balance sheet
    if financials.balance_sheet.assets.is_some() {
        count += 1;
    }
    if financials.balance_sheet.liabilities.is_some() {
        count += 1;
    }
    if financials.balance_sheet.stockholders_equity.is_some() {
        count += 1;
    }
    if financials
        .balance_sheet
        .deferred_compensation_liability_noncurrent
        .is_some()
    {
        count += 1;
    }

    // Income statement
    if financials.income_statement.revenues.is_some() {
        count += 1;
    }
    if financials.income_statement.net_income_loss.is_some() {
        count += 1;
    }

    // Narratives
    if financials.narratives.nature_of_operations.is_some() {
        count += 1;
    }
    if financials.narratives.related_party_transactions.is_some() {
        count += 1;
    }
    if financials.narratives.stockholders_equity_note.is_some() {
        count += 1;
    }
    if financials.narratives.subsequent_events.is_some() {
        count += 1;
    }
    if financials
        .narratives
        .commitments_and_contingencies
        .is_some()
    {
        count += 1;
    }

    // Add more field checks as needed...
    count
}
