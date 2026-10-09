//! Reads the same XML instances with xbrlkit, crabrl and xbrl-rs, and
//! reports how many facts each returns and the median time it takes.
//!
//! ```text
//! cd benches/compare
//! cargo run --release                      # the instances in tests/fixtures
//! cargo run --release -- a.xml b.xml       # your own
//! ```
//!
//! Only XML instances are compared: neither of the other two reads inline
//! XBRL. Each parser is called through its documented entry point with its
//! default configuration, on bytes already in memory.

use std::hint::black_box;
use std::io::Cursor;
use std::time::{Duration, Instant};

const FIXTURES: [&str; 3] = [
    "../../tests/fixtures/aapl-10k-2025_htm.xml",
    "../../tests/fixtures/tsla-10q-2026q2_htm.xml",
    "../../tests/fixtures/spac/form_10q_4.xml",
];

/// The median time of `f` over two seconds of runs.
fn median<T>(mut f: impl FnMut() -> T) -> Duration {
    for _ in 0..3 {
        black_box(f());
    }
    let mut samples = Vec::new();
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(2) || samples.len() < 10 {
        let run = Instant::now();
        black_box(f());
        samples.push(run.elapsed());
    }
    samples.sort();
    samples[samples.len() / 2]
}

/// The fact count, or the start of the error that stood in for one.
fn outcome<E: std::fmt::Display>(result: Result<usize, E>) -> String {
    match result {
        Ok(facts) => facts.to_string(),
        Err(e) => format!("error: {}", e.to_string().chars().take(44).collect::<String>()),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let paths: Vec<&str> = match args.is_empty() {
        true => FIXTURES.to_vec(),
        false => args.iter().map(String::as_str).collect(),
    };

    for path in paths {
        let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
        let bytes = text.as_bytes();
        // The number of elements that name a context: what there is to find.
        let tagged = text.matches("contextRef=\"").count();
        println!(
            "\n{} ({:.2} MB, {tagged} facts)",
            path.rsplit('/').next().unwrap_or(path),
            text.len() as f64 / 1e6
        );

        let time = median(|| xbrlkit::parser::parse_xml(black_box(&text)));
        let facts = xbrlkit::parser::parse_xml(&text).map(|instance| instance.facts.len());
        println!("  xbrlkit  {time:>10.2?}  {}", outcome(facts));

        let parser = crabrl::Parser::new();
        let time = median(|| parser.parse_bytes(black_box(bytes)));
        let facts = parser.parse_bytes(bytes).map(|document| document.facts.len());
        println!("  crabrl   {time:>10.2?}  {}", outcome(facts));

        let read = |bytes| xbrl_rs::InstanceDocument::from_reader(Cursor::new(bytes));
        let time = median(|| read(black_box(bytes)));
        let facts = read(bytes).map(|document| document.facts().len());
        println!("  xbrl-rs  {time:>10.2?}  {}", outcome(facts));
    }
}
