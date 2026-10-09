//! How long it takes to read a filing, and to read structs from it.
//!
//! ```text
//! cargo bench
//! cargo bench -- path/to/filing.htm another.xml
//! ```
//!
//! With no arguments it reads the filings in `tests/fixtures`. Each step is
//! timed over many runs and the median is reported; there is no statistics
//! beyond that, by design — this is here to show the order of magnitude and
//! to catch a regression, and has no dependencies to do it.

use std::hint::black_box;
use std::time::{Duration, Instant};
use xbrlkit::taxonomies::{dei::DeiInfo, us_gaap::Financials};
use xbrlkit::{Document, parser};

const FIXTURES: [&str; 5] = [
    "tests/fixtures/aapl-10k-2025.htm",
    "tests/fixtures/aapl-10k-2025_htm.xml",
    "tests/fixtures/tsla-10q-2026q2.htm",
    "tests/fixtures/tsla-10q-2026q2_htm.xml",
    "tests/fixtures/spac/10-q_3.html",
];

/// The median time of `f`, run until at least `budget` has been spent.
fn median<T>(budget: Duration, mut f: impl FnMut() -> T) -> Duration {
    // Warm the caches and the allocator before measuring.
    for _ in 0..3 {
        black_box(f());
    }
    let mut samples = Vec::new();
    let started = Instant::now();
    while started.elapsed() < budget || samples.len() < 10 {
        let run = Instant::now();
        black_box(f());
        samples.push(run.elapsed());
    }
    samples.sort();
    samples[samples.len() / 2]
}

fn main() {
    // `cargo bench` passes `--bench`; anything else is a path.
    let args: Vec<String> = std::env::args()
        .skip(1)
        .filter(|arg| !arg.starts_with("--"))
        .collect();
    let paths: Vec<&str> = match args.is_empty() {
        true => FIXTURES.to_vec(),
        false => args.iter().map(String::as_str).collect(),
    };
    let budget = Duration::from_millis(1500);

    println!(
        "{:<42} {:>8} {:>7} {:>10} {:>9} {:>10} {:>11}",
        "document", "size", "facts", "parse", "MB/s", "index", "statements"
    );
    for path in paths {
        let content = match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("{path}: {e}");
                continue;
            }
        };
        let is_xml = parser::is_xml_instance(&content);
        let parse = || match is_xml {
            true => parser::parse_xml(black_box(&content)),
            false => parser::parse_ixbrl(black_box(&content)),
        };
        let instance = match parse() {
            Ok(instance) => instance,
            Err(e) => {
                eprintln!("{path}: {e}");
                continue;
            }
        };
        let facts = instance.facts.len();

        // Reading the text into contexts, units and facts.
        let parse_time = median(budget, parse);
        // Building the indexes a struct is read through.
        let index_time = median(budget, || Document::new(instance.clone()))
            .saturating_sub(median(budget, || instance.clone()));
        // Reading the cover page and all three statements, every period.
        let doc = Document::new(instance.clone());
        let extract_time = median(budget, || {
            (
                doc.extract_lenient::<DeiInfo>(),
                doc.extract_lenient::<Financials>(),
            )
        });

        let megabytes = content.len() as f64 / 1e6;
        let name = path.rsplit('/').next().unwrap_or(path);
        println!(
            "{name:<42} {megabytes:>6.2}MB {facts:>7} {:>10.2?} {:>9.0} {:>10.2?} {:>11.2?}",
            parse_time,
            megabytes / parse_time.as_secs_f64(),
            index_time,
            extract_time,
        );
    }
}
