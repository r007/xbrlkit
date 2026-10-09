//! # Inline fact values
//!
//! What the value of an inline fact is: its sign, its scale, the text HTML
//! wraps, the facts nested inside it, and the text it continues into.

use xbrlkit::instance::{Instance, XbrlValue};
use xbrlkit::parser::parse_ixbrl;

/// Wraps a body in the smallest document the parser accepts.
fn document(body: &str) -> String {
    format!(
        r#"<html xmlns:ix="http://www.xbrl.org/2013/inlineXBRL">
<head><ix:header><ix:resources>
  <xbrli:context id="c0"><xbrli:entity><xbrli:identifier scheme="http://www.sec.gov/CIK">0001</xbrli:identifier></xbrli:entity>
    <xbrli:period><xbrli:instant>2025-09-30</xbrli:instant></xbrli:period></xbrli:context>
  <xbrli:unit id="usd"><xbrli:measure>iso4217:USD</xbrli:measure></xbrli:unit>
</ix:resources></ix:header></head>
<body>{body}</body></html>"#
    )
}

fn parse(body: &str) -> Instance {
    parse_ixbrl(&document(body)).expect("document should parse")
}

/// The value of the only fact tagged `name`.
fn value(xbrl: &Instance, name: &str) -> Option<String> {
    let mut found = xbrl.facts.iter().filter(|f| f.full_name == name);
    let fact = found.next().unwrap_or_else(|| panic!("no fact {name}"));
    assert!(found.next().is_none(), "more than one fact {name}");
    match &fact.value {
        XbrlValue::String(s) => Some(s.clone()),
        XbrlValue::Nil => None,
    }
}

#[test]
fn sign_attribute_makes_the_value_negative() {
    // A loss is printed "(16,068,717)" with the parentheses outside the tag;
    // `sign` is the only place the minus lives.
    let xbrl = parse(
        r#"(<ix:nonFraction name="us-gaap:StockholdersEquity" contextRef="c0" unitRef="usd"
              decimals="0" format="ixt:numdotdecimal" sign="-" scale="0">16,068,717</ix:nonFraction>)
           <ix:nonFraction name="us-gaap:Assets" contextRef="c0" unitRef="usd"
              decimals="0" format="ixt:numdotdecimal">12,680,715</ix:nonFraction>"#,
    );
    assert_eq!(
        value(&xbrl, "us-gaap:StockholdersEquity").as_deref(),
        Some("-16068717")
    );
    assert_eq!(value(&xbrl, "us-gaap:Assets").as_deref(), Some("12680715"));
}

#[test]
fn sign_is_applied_after_scale_and_leaves_zero_alone() {
    let xbrl = parse(
        r#"<ix:nonFraction name="us-gaap:NetIncomeLoss" contextRef="c0" unitRef="usd"
              format="ixt:numdotdecimal" sign="-" scale="3">1,234</ix:nonFraction>
           <ix:nonFraction name="us-gaap:EarningsPerShareBasic" contextRef="c0" unitRef="usd"
              format="ixt:numdotdecimal" sign="-" scale="0">0.00</ix:nonFraction>"#,
    );
    assert_eq!(
        value(&xbrl, "us-gaap:NetIncomeLoss").as_deref(),
        Some("-1234000")
    );
    assert_eq!(
        value(&xbrl, "us-gaap:EarningsPerShareBasic").as_deref(),
        Some("0")
    );
}

#[test]
fn negative_scale_moves_the_decimal_point_exactly() {
    // Through f64 this was 0.055000000000000004.
    let xbrl = parse(
        r#"<ix:nonFraction name="us-gaap:DebtInstrumentInterestRateStatedPercentage" contextRef="c0"
              unitRef="usd" format="ixt:numdotdecimal" scale="-2">5.5</ix:nonFraction>
           <ix:nonFraction name="us-gaap:Liabilities" contextRef="c0" unitRef="usd"
              format="ixt:numdotdecimal" scale="6">1.5</ix:nonFraction>"#,
    );
    assert_eq!(
        value(&xbrl, "us-gaap:DebtInstrumentInterestRateStatedPercentage").as_deref(),
        Some("0.055")
    );
    assert_eq!(
        value(&xbrl, "us-gaap:Liabilities").as_deref(),
        Some("1500000")
    );
}

#[test]
fn text_wrapped_in_formatting_is_the_value() {
    let xbrl = parse(
        r#"<ix:nonNumeric name="dei:EntityRegistrantName" contextRef="c0"><b style="font-weight:bold;">NEWCOURT
              ACQUISITION&nbsp;CORP</b></ix:nonNumeric>
           <ix:nonNumeric name="dei:DocumentPeriodEndDate" contextRef="c0"
              format="ixt:datemonthdayyearen"><span>September 30</span>, <span>2025</span></ix:nonNumeric>
           <ix:nonFraction name="us-gaap:Cash" contextRef="c0" unitRef="usd"
              format="ixt:numdotdecimal"><span>88,174</span></ix:nonFraction>"#,
    );
    assert_eq!(
        value(&xbrl, "dei:EntityRegistrantName").as_deref(),
        Some("NEWCOURT ACQUISITION CORP")
    );
    assert_eq!(
        value(&xbrl, "dei:DocumentPeriodEndDate").as_deref(),
        Some("2025-09-30")
    );
    assert_eq!(value(&xbrl, "us-gaap:Cash").as_deref(), Some("88174"));
}

#[test]
fn facts_inside_a_text_block_are_extracted() {
    // Every figure in the notes sits inside the text block that tags its note.
    let xbrl = parse(
        r#"<ix:nonNumeric name="us-gaap:StockholdersEquityNoteDisclosureTextBlock" contextRef="c0" escape="true">
             <p><b>Note 7 — Warrants</b></p>
             <p>Each warrant is exercisable at $<ix:nonFraction
                 name="us-gaap:ClassOfWarrantOrRightExercisePriceOfWarrantsOrRights1" contextRef="c0"
                 unitRef="usd" decimals="2">11.50</ix:nonFraction> per share.</p>
             <table><tr><td>Trust</td><td><ix:nonFraction name="us-gaap:AssetsHeldInTrustNoncurrent"
                 contextRef="c0" unitRef="usd" format="ixt:numdotdecimal">12,518,199</ix:nonFraction></td></tr></table>
           </ix:nonNumeric>"#,
    );
    assert_eq!(
        value(
            &xbrl,
            "us-gaap:ClassOfWarrantOrRightExercisePriceOfWarrantsOrRights1"
        )
        .as_deref(),
        Some("11.50")
    );
    assert_eq!(
        value(&xbrl, "us-gaap:AssetsHeldInTrustNoncurrent").as_deref(),
        Some("12518199")
    );

    // The text block itself reads as lines of text, figures included as shown.
    assert_eq!(
        value(&xbrl, "us-gaap:StockholdersEquityNoteDisclosureTextBlock").as_deref(),
        Some(
            "Note 7 — Warrants\nEach warrant is exercisable at $11.50 per share.\nTrust\n12,518,199"
        )
    );

    // A nested fact is registered before the fact that contains it.
    let names: Vec<&str> = xbrl.facts.iter().map(|f| f.local_name.as_str()).collect();
    assert_eq!(
        names.last().copied(),
        Some("StockholdersEquityNoteDisclosureTextBlock")
    );
}

#[test]
fn excluded_text_is_left_out_but_its_facts_are_kept() {
    let xbrl = parse(
        r#"<ix:nonNumeric name="us-gaap:SubsequentEventsTextBlock" contextRef="c0" escape="true">
             <p>The Company evaluated subsequent events</p>
             <ix:exclude><p>Page 12 of <ix:nonFraction name="us-gaap:NumberOfOperatingSegments"
                 contextRef="c0" unitRef="usd">1</ix:nonFraction></p></ix:exclude>
             <p>through the date of issuance.</p>
           </ix:nonNumeric>"#,
    );
    assert_eq!(
        value(&xbrl, "us-gaap:SubsequentEventsTextBlock").as_deref(),
        Some("The Company evaluated subsequent events\nthrough the date of issuance.")
    );
    assert_eq!(
        value(&xbrl, "us-gaap:NumberOfOperatingSegments").as_deref(),
        Some("1")
    );
}

#[test]
fn a_value_runs_on_through_its_continuations() {
    // The chain is followed in `continuedAt` order, wherever the pieces sit.
    let xbrl = parse(
        r#"<ix:continuation id="part3"><p>and ends here.</p></ix:continuation>
           <ix:nonNumeric name="us-gaap:NatureOfOperations" contextRef="c0" continuedAt="part2"
              escape="true"><p>The note starts here,</p></ix:nonNumeric>
           <p>Unrelated page footer</p>
           <ix:continuation id="part2" continuedAt="part3"><p>carries a figure of <ix:nonFraction
              name="us-gaap:Cash" contextRef="c0" unitRef="usd">250</ix:nonFraction></p></ix:continuation>"#,
    );
    assert_eq!(
        value(&xbrl, "us-gaap:NatureOfOperations").as_deref(),
        Some("The note starts here,\ncarries a figure of 250\nand ends here.")
    );
    assert_eq!(value(&xbrl, "us-gaap:Cash").as_deref(), Some("250"));
}

#[test]
fn nil_fact_stays_nil_and_does_not_swallow_what_follows() {
    let xbrl = parse(
        r#"<ix:nonFraction name="us-gaap:OtherLiabilitiesCurrent" contextRef="c0" unitRef="usd"
              xsi:nil="true"></ix:nonFraction>
           <ix:nonFraction name="us-gaap:Liabilities" contextRef="c0" unitRef="usd">42</ix:nonFraction>"#,
    );
    assert_eq!(value(&xbrl, "us-gaap:OtherLiabilitiesCurrent"), None);
    assert_eq!(value(&xbrl, "us-gaap:Liabilities").as_deref(), Some("42"));
}

#[test]
fn word_processor_tags_are_not_facts() {
    let xbrl = parse(
        r#"<p><o:p>stray</o:p></p>
           <ix:nonFraction name="us-gaap:Assets" contextRef="c0" unitRef="usd">7</ix:nonFraction>"#,
    );
    assert_eq!(xbrl.facts.len(), 1);
    assert_eq!(value(&xbrl, "us-gaap:Assets").as_deref(), Some("7"));
}
