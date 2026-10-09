//! Walks the parsed document itself, without binding it to a struct.
//!
//! ```text
//! cargo run --example facts -- path/to/filing.htm [concept]
//! ```
//!
//! With a concept (`us-gaap:Assets`, or just `Assets`) it lists every fact
//! tagged with it. Without one it summarises the filing. With no argument at
//! all it reads the Apple 10-K in `tests/fixtures`.

use std::collections::BTreeMap;
use xbrlkit::{Document, Fact};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .unwrap_or_else(|| "tests/fixtures/aapl-10k-2025.htm".to_string());
    let concept = args.next();

    let doc = Document::parse(&std::fs::read_to_string(&path)?)?;

    match concept {
        Some(concept) => list(&doc, &concept),
        None => summarise(&doc),
    }
    Ok(())
}

/// Every fact for one concept: each period, each dimension member.
fn list(doc: &Document, concept: &str) {
    // The same call a `Vec<Fact<String>>` field makes. `String` accepts any
    // value; ask for `f64` to drop what is not a number.
    let facts: Vec<Fact<String>> = doc.get(&[concept]);
    if facts.is_empty() {
        println!("the filing does not tag {concept}");
    }
    for fact in facts {
        let period = match &fact.period_start {
            Some(start) => format!("{start} to {}", fact.period_end.as_deref().unwrap_or("?")),
            None => fact.period_end.clone().unwrap_or_default(),
        };
        let members: Vec<&str> = fact.dimensions.iter().map(|d| d.member.as_str()).collect();
        let value: String = fact.value.chars().take(60).collect();
        println!(
            "{period:<24} {value:>20} {:<12} {}",
            fact.unit.as_deref().unwrap_or(""),
            members.join(", ")
        );
    }
}

/// What the filing holds, counted from the raw facts.
fn summarise(doc: &Document) {
    println!(
        "{} facts, {} contexts, {} units",
        doc.facts().len(),
        doc.contexts().len(),
        doc.units().len()
    );
    if let Some(period) = doc.reporting_period() {
        println!(
            "reporting period: {} to {}",
            period.start.as_deref().unwrap_or("(instant)"),
            period.end
        );
    }

    // Facts by taxonomy prefix: us-gaap, dei, srt, and the filer's own.
    let mut by_prefix: BTreeMap<&str, usize> = BTreeMap::new();
    for fact in doc.facts() {
        let prefix = fact
            .full_name
            .split_once(':')
            .map_or("", |(prefix, _)| prefix);
        *by_prefix.entry(prefix).or_default() += 1;
    }
    println!("\nfacts by taxonomy:");
    for (prefix, count) in &by_prefix {
        println!("  {prefix:<10} {count}");
    }

    let consolidated = doc
        .contexts()
        .iter()
        .filter(|c| c.is_consolidated())
        .count();
    println!(
        "\ncontexts: {consolidated} for the entity as a whole, {} narrowed by a dimension",
        doc.contexts().len() - consolidated
    );

    let nil = doc.facts().iter().filter(|f| f.value.is_nil()).count();
    println!("facts reported with no value: {nil}");
}
