//! Holds the inline parser to the SEC's own reading of the same filings.
//!
//! EDGAR extracts a traditional instance (`*_htm.xml`) from every inline
//! filing it accepts. That file is what the SEC says the inline document
//! means: every `format`, `scale` and `sign` already applied. Reading the
//! inline document here and comparing it, fact by fact, with that instance
//! checks the whole inline path against an independent implementation.

use std::collections::HashMap;
use std::fs::read_to_string;
use xbrlkit::parser::{parse_ixbrl, parse_xml};
use xbrlkit::{Instance, RawFact};

/// An inline filing and the instance EDGAR extracted from it.
fn filing(name: &str) -> (Instance, Instance) {
    let read = |file: String| {
        read_to_string(format!("tests/fixtures/{file}")).unwrap_or_else(|e| panic!("{file}: {e}"))
    };
    let inline = parse_ixbrl(&read(format!("{name}.htm"))).expect("the inline document parses");
    let extracted = parse_xml(&read(format!("{name}_htm.xml"))).expect("the instance parses");
    (inline, extracted)
}

fn number(fact: &RawFact) -> Option<f64> {
    fact.value.as_str()?.parse().ok()
}

fn assert_agrees_with_the_sec(name: &str) {
    let (inline, extracted) = filing(name);

    assert_eq!(inline.facts.len(), extracted.facts.len(), "{name}: facts");
    assert_eq!(
        inline.contexts.len(),
        extracted.contexts.len(),
        "{name}: contexts"
    );
    assert_eq!(inline.units.len(), extracted.units.len(), "{name}: units");

    // EDGAR carries each inline fact's id over to the instance.
    let by_id: HashMap<&str, &RawFact> = extracted
        .facts
        .iter()
        .filter_map(|fact| Some((fact.id.as_deref()?, fact)))
        .collect();

    let (mut numeric, mut formatted) = (0, 0);
    for fact in &inline.facts {
        let id = fact.id.as_deref().expect("every inline fact has an id");
        let theirs = by_id
            .get(id)
            .unwrap_or_else(|| panic!("{name}: {} [{id}] is not in the instance", fact.full_name));

        assert_eq!(fact.full_name, theirs.full_name, "{name} [{id}]: concept");
        assert_eq!(
            fact.context_ref, theirs.context_ref,
            "{name} [{id}]: context"
        );
        assert_eq!(fact.unit_ref, theirs.unit_ref, "{name} [{id}]: unit");

        if fact.unit_ref.is_some() {
            // The same number, however each side writes it: `1234000` or `1.234E6`.
            numeric += 1;
            assert_eq!(
                number(fact),
                number(theirs),
                "{name}: {} [{id}] is {:?} here and {:?} in the instance",
                fact.full_name,
                fact.value,
                theirs.value
            );
        } else if fact.format.is_some() {
            // Dates, durations, check boxes, exchange names: the same text.
            formatted += 1;
            assert_eq!(
                fact.value, theirs.value,
                "{name}: {} [{id}] under {:?}",
                fact.full_name, fact.format
            );
        }
        // What is left is prose, which the instance keeps as escaped markup
        // and this crate reads as text.
    }

    assert!(
        numeric > 900,
        "{name}: only {numeric} numeric facts compared"
    );
    assert!(
        formatted > 20,
        "{name}: only {formatted} formatted facts compared"
    );
}

#[test]
fn an_annual_report_reads_as_the_sec_reads_it() {
    assert_agrees_with_the_sec("aapl-10k-2025");
}

#[test]
fn a_quarterly_report_reads_as_the_sec_reads_it() {
    assert_agrees_with_the_sec("tsla-10q-2026q2");
}

#[test]
fn contexts_carry_the_same_periods_and_members() {
    let (inline, extracted) = filing("tsla-10q-2026q2");
    let theirs: HashMap<&str, _> = extracted
        .contexts
        .iter()
        .map(|c| (c.id.as_str(), c))
        .collect();

    let mut typed = 0;
    for ours in &inline.contexts {
        let theirs = theirs[ours.id.as_str()];
        assert_eq!(ours.period.instant, theirs.period.instant);
        assert_eq!(ours.period.start_date, theirs.period.start_date);
        assert_eq!(ours.period.end_date, theirs.period.end_date);

        let members = |context: &xbrlkit::instance::Context| -> Vec<(String, String)> {
            let explicit = context.explicit_members().map(|m| (&m.dimension, &m.value));
            let typed = context.typed_members().map(|m| (&m.dimension, &m.value));
            let mut members: Vec<_> = explicit
                .chain(typed)
                .map(|(axis, member)| (axis.trim().to_string(), member.trim().to_string()))
                .collect();
            // The order of members within a context carries no meaning, and
            // EDGAR does not keep the document's.
            members.sort();
            members
        };
        assert_eq!(members(ours), members(theirs), "context {}", ours.id);
        typed += ours.typed_members().count();
    }
    // The filing uses typed dimensions, and they are read rather than dropped.
    assert!(typed > 0);
}
