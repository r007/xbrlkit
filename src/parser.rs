//! # The parser
//!
//! Reads a filing into an [`Instance`]: every context, unit and fact it tags,
//! in document order. One pass over the text, no DOM.
//!
//! An SEC filing carries its tagged data in one of two forms, and there is an
//! entry point for each:
//!
//! **Inline XBRL** — [`parse_ixbrl`]. The facts are tagged in place in the
//! HTML a reader sees. Every 10-K, 10-Q and 8-K since 2019–2021 is filed this
//! way.
//!
//! ```html
//! <html xmlns:ix="http://www.xbrl.org/2013/inlineXBRL">
//!   <ix:header>
//!     <xbrli:context id="c1">...</xbrli:context>
//!     <xbrli:unit id="usd">...</xbrli:unit>
//!   </ix:header>
//!   <body>
//!     Total assets: <ix:nonFraction name="us-gaap:Assets" contextRef="c1" unitRef="usd"
//!                    scale="6" format="ixt:num-dot-decimal">359,241</ix:nonFraction>
//!   </body>
//! </html>
//! ```
//!
//! **A traditional instance** — [`parse_xml`]. A separate XML document where
//! each fact is an element named for its concept. Older filings attach one as
//! an exhibit, and EDGAR extracts one (`*_htm.xml`) from every inline filing.
//!
//! ```xml
//! <xbrl xmlns="http://www.xbrl.org/2003/instance">
//!   <context id="c1">...</context>
//!   <unit id="usd">...</unit>
//!   <us-gaap:Assets contextRef="c1" unitRef="usd" decimals="-6">359241000000</us-gaap:Assets>
//! </xbrl>
//! ```
//!
//! [`is_xml_instance`] tells the two apart, and
//! [`Document::parse`](crate::Document::parse) uses it to pick.
//!
//! ## What an inline value is
//!
//! The text between an inline fact's tags is what the filing *shows*; the
//! value is what that text *means*. The parser closes the gap:
//!
//! - the value is the text of everything inside the tag, whatever HTML wraps
//!   it, with `ix:exclude` left out and any `ix:continuation` appended
//! - the `format` [transformation](crate::transformations) is applied:
//!   `1,234` becomes `1234`, `September 30, 2025` becomes `2025-09-30`
//! - `scale` moves the decimal point, exactly: `1,234` at scale 3 is `1234000`
//! - `sign="-"` makes it negative, which is the only place the sign of a
//!   figure printed as `(1,234)` lives
//! - facts nested inside another fact — every figure in a note that is itself
//!   tagged as a text block — are facts too
//!
//! ## Leniency
//!
//! Filings are produced by many tools and checked by none of them for
//! well-formed HTML. Mismatched and unclosed tags, unknown entities and
//! inconsistent attribute case (`contextRef`, `contextref`) are all read
//! through. A context or unit that cannot be understood is skipped rather
//! than failing the document.
//!
//! ```no_run
//! use xbrlkit::parser::{is_xml_instance, parse_ixbrl, parse_xml};
//!
//! let content = std::fs::read_to_string("filing.htm")?;
//! let instance = match is_xml_instance(&content) {
//!     true => parse_xml(&content)?,
//!     false => parse_ixbrl(&content)?,
//! };
//! println!("{} facts in {} contexts", instance.facts.len(), instance.contexts.len());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::error::{Result, XbrlError};
use crate::instance::{Instance, RawFact, XbrlValue};
use crate::transformations;
use quick_xml::{
    Reader, Writer,
    de::from_str,
    events::{BytesStart, Event},
};
use std::borrow::Cow;
use std::collections::HashMap;

/// Whether a tag is the root of an instance: `xbrl`, under any prefix.
fn is_xbrl_root_element(tag_name: &str) -> bool {
    tag_name == "xbrl" || tag_name.ends_with(":xbrl")
}

/// Whether a document is a traditional XBRL instance rather than inline
/// XBRL, judged by its root element: `<xbrl>` under any prefix, against the
/// `<html>` of an inline document.
///
/// Only the start of the document is read — the XML declaration, comments
/// and the doctype are stepped over to reach the first element.
pub fn is_xml_instance(content: &str) -> bool {
    let mut rest = content;
    loop {
        let Some(open) = rest.find('<') else {
            return false;
        };
        rest = &rest[open + 1..];
        if let Some(comment) = rest.strip_prefix("!--") {
            rest = comment.find("-->").map_or("", |end| &comment[end + 3..]);
        } else if !rest.starts_with(['?', '!']) {
            break;
        }
    }
    let name_end = rest
        .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
        .unwrap_or(rest.len());
    local_part(&rest[..name_end]).eq_ignore_ascii_case("xbrl")
}

/// Returns the local part of a (possibly) namespace-qualified tag name.
///
/// `us-gaap:Assets` → `Assets`, `context` → `context`.
fn local_part(tag_name: &str) -> &str {
    match tag_name.rfind(':') {
        Some(idx) => &tag_name[idx + 1..],
        None => tag_name,
    }
}

/// Checks whether an element carries an `id` attribute (case-insensitive).
///
/// Used to distinguish real XBRL `<context>`/`<unit>` elements — which are
/// required to have an `id` — from unrelated markup that happens to share the
/// name when scanning an iXBRL HTML document.
fn has_id_attribute(e: &BytesStart) -> bool {
    e.attributes()
        .flatten()
        .any(|attr| attr.key.as_ref().eq_ignore_ascii_case(b"id"))
}

/// Parses a traditional XBRL instance document.
///
/// Fails with [`XbrlError::NotXbrl`] when the document has no `<xbrl>` root,
/// and with [`XbrlError::Malformed`] when its XML cannot be read.
///
/// ```
/// use xbrlkit::parser::parse_xml;
///
/// let instance = parse_xml(r#"<?xml version="1.0" encoding="utf-8"?>
///     <xbrl xmlns="http://www.xbrl.org/2003/instance" xmlns:dei="http://xbrl.sec.gov/dei/2025">
///       <context id="c0">
///         <entity><identifier scheme="http://www.sec.gov/CIK">0001234567</identifier></entity>
///         <period><instant>2025-03-31</instant></period>
///       </context>
///       <dei:EntityRegistrantName contextRef="c0">Example Corp</dei:EntityRegistrantName>
///     </xbrl>"#)?;
///
/// assert_eq!(instance.contexts.len(), 1);
/// assert_eq!(instance.facts[0].full_name, "dei:EntityRegistrantName");
/// assert_eq!(instance.facts[0].value.as_str(), Some("Example Corp"));
/// # Ok::<(), xbrlkit::XbrlError>(())
/// ```
pub fn parse_xml(xml_content: &str) -> Result<Instance> {
    let mut reader = Reader::from_str(xml_content);

    // Configure parser for optimal performance
    reader.config_mut().trim_text(true);
    reader.config_mut().expand_empty_elements = false;

    let mut buf = Vec::new();
    let mut xbrl = Instance::default();

    // Find the root XBRL element
    loop {
        match reader
            .read_event_into(&mut buf)
            .map_err(XbrlError::malformed)?
        {
            Event::Start(e) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if is_xbrl_root_element(&tag_name) {
                    // Use the generic event processor, breaking when we find the closing </xbrl> tag.
                    process_events(&mut reader, &mut xbrl, false, |event| {
                        if let Event::End(end_event) = event {
                            let end_tag_name =
                                String::from_utf8_lossy(end_event.name().as_ref()).to_string();
                            return is_xbrl_root_element(&end_tag_name);
                        }
                        false
                    })?;
                    return Ok(xbrl); // Parsing finished
                }
            }
            Event::Eof => {
                return Err(XbrlError::NotXbrl(
                    "no <xbrl> root element in the document".to_string(),
                ));
            }
            _ => {
                // Continue searching for root element
            }
        }
        buf.clear();
    }
}

/// Parses an inline XBRL document: a filing's HTML with the facts tagged in
/// place.
///
/// See the [module documentation](self) for what becomes of an inline fact's
/// text. Fails only when the markup cannot be read at all; a document that
/// tags nothing parses to an empty [`Instance`].
///
/// ```
/// use xbrlkit::parser::parse_ixbrl;
///
/// let instance = parse_ixbrl(r#"<html xmlns:ix="http://www.xbrl.org/2013/inlineXBRL"><body>
///     Net loss of $(<ix:nonFraction name="us-gaap:NetIncomeLoss" contextRef="c1" unitRef="usd"
///         scale="3" sign="-" format="ixt:num-dot-decimal">1,234</ix:nonFraction>) thousand
///     </body></html>"#)?;
///
/// assert_eq!(instance.facts[0].value.as_str(), Some("-1234000"));
/// # Ok::<(), xbrlkit::XbrlError>(())
/// ```
pub fn parse_ixbrl(html_content: &str) -> Result<Instance> {
    let mut reader = Reader::from_str(html_content);
    reader.config_mut().trim_text(true);
    reader.config_mut().expand_empty_elements = true;
    // Filings are HTML, not XML: tags close out of order, or never.
    reader.config_mut().check_end_names = false;
    reader.config_mut().allow_unmatched_ends = true;

    let mut xbrl = Instance::default();

    // Use the generic event processor. For iXBRL, we process until the end of the file.
    process_events(&mut reader, &mut xbrl, true, |event| {
        matches!(event, Event::Eof)
    })?;

    Ok(xbrl)
}

/// Attributes of a fact element that shape its value without being part of it.
#[derive(Debug, Default)]
struct FactAttrs {
    /// `xsi:nil="true"`: the fact is reported, and has no value.
    is_nil: bool,

    /// iXBRL `scale`: the displayed number is multiplied by ten to this power.
    scale: Option<i32>,

    /// iXBRL `sign="-"`: the displayed number is the absolute value of a
    /// negative one. Filings print losses as `(1,234)` with the parentheses
    /// outside the tag, so this attribute is the only place the sign lives.
    negative: bool,

    /// iXBRL `continuedAt`: the value runs on in the `ix:continuation` with this id.
    continued_at: Option<String>,

    /// iXBRL `escape="true"`: the value is a block of markup (a text block),
    /// whose line structure is worth keeping.
    escape: bool,
}

/// What a pass over an iXBRL document has to remember until it ends.
///
/// An inline fact's value cannot be settled where its tag closes: it may run on
/// in `ix:continuation` elements that appear later in the document, sometimes
/// pages later. Facts are therefore registered as they are met, in document
/// order, and their values are settled once every continuation has been read.
#[derive(Default)]
struct InlineState {
    /// One entry per inline fact: its index in `Instance::facts`, the attributes
    /// that shape its value, and the text collected between its tags.
    pending: Vec<(usize, FactAttrs, String)>,

    /// The text of each `ix:continuation` by id, and the id it continues at.
    continuations: HashMap<String, (String, Option<String>)>,
}

/// The tags that carry an inline fact.
fn is_inline_fact_tag(lowercase_tag: &str) -> bool {
    matches!(
        lowercase_tag,
        "ix:nonfraction" | "ix:fraction" | "ix:nonnumeric"
    )
}

/// HTML elements that start a new line of text. Their boundaries become line
/// breaks in the collected text so that words on either side do not fuse.
fn is_block_tag(lowercase_tag: &str) -> bool {
    matches!(
        lowercase_tag,
        "p" | "div"
            | "br"
            | "tr"
            | "td"
            | "th"
            | "li"
            | "ul"
            | "ol"
            | "table"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "hr"
            | "blockquote"
    )
}

/// Resolves the named entities an SEC filing's HTML uses beyond XML's five.
///
/// quick-xml only knows the predefined XML entities, and an unresolved one
/// fails the whole text node — which used to leave the literal `&nbsp;` inside
/// values. Numeric references (`&#160;`) are handled by quick-xml itself.
fn resolve_entity(entity: &str) -> Option<&'static str> {
    if let Some(predefined) = quick_xml::escape::resolve_predefined_entity(entity) {
        return Some(predefined);
    }
    Some(match entity {
        "nbsp" | "ensp" | "emsp" | "thinsp" => " ",
        "ndash" => "–",
        "mdash" => "—",
        "lsquo" => "‘",
        "rsquo" => "’",
        "sbquo" => "‚",
        "ldquo" => "“",
        "rdquo" => "”",
        "bdquo" => "„",
        "laquo" => "«",
        "raquo" => "»",
        "hellip" => "…",
        "bull" => "•",
        "middot" => "·",
        "sect" => "§",
        "para" => "¶",
        "copy" => "©",
        "reg" => "®",
        "trade" => "™",
        "deg" => "°",
        "plusmn" => "±",
        "times" => "×",
        "divide" => "÷",
        "cent" => "¢",
        "pound" => "£",
        "euro" => "€",
        "yen" => "¥",
        "frac12" => "½",
        "frac14" => "¼",
        "frac34" => "¾",
        "dagger" => "†",
        "Dagger" => "‡",
        "shy" | "zwnj" | "zwj" => "",
        _ => return None,
    })
}

/// Decodes a text node, falling back to the raw bytes when it holds an entity
/// nobody recognises.
fn decode_text(text: &quick_xml::events::BytesText) -> String {
    text.unescape_with(resolve_entity)
        .unwrap_or_else(|_| Cow::Owned(String::from_utf8_lossy(text.as_ref()).into_owned()))
        .into_owned()
}

/// Reads one attribute by name, ignoring case.
fn attribute(e: &BytesStart, lowercase_name: &str) -> Option<String> {
    e.attributes().flatten().find_map(|attr| {
        attr.key
            .as_ref()
            .eq_ignore_ascii_case(lowercase_name.as_bytes())
            .then(|| String::from_utf8_lossy(&attr.value).into_owned())
    })
}

/// Reads an inline fact (`ix:nonFraction`, `ix:nonNumeric`, `ix:fraction`) and
/// everything inside it, registering the fact — and any fact nested in it — on
/// `xbrl`. Returns the fact's index in `inline.pending`.
///
/// ## What counts as the value
///
/// The value of an inline fact is the text of *all* its descendants, whatever
/// HTML wraps them. A registrant's name is tagged as
/// `<ix:nonNumeric ...><b>EXAMPLE HOLDINGS INC</b></ix:nonNumeric>`, and a
/// text block wraps whole pages of `<p>` and `<table>`. Only `ix:exclude` is
/// left out, which is how a filer keeps page headers out of a text block.
///
/// ## Nested facts
///
/// Facts nest two ways, and both are common:
///
/// ```xml
/// <!-- a value built from another value -->
/// <ix:nonNumeric format="ixt:date-monthname-day-year-en" name="dei:DocumentPeriodEndDate">
///     September 30, <ix:nonNumeric name="dei:DocumentFiscalYearFocus">2025</ix:nonNumeric>
/// </ix:nonNumeric>
///
/// <!-- a note, tagged as a text block, holding the figures it discusses -->
/// <ix:nonNumeric name="us-gaap:StockholdersEquityNoteDisclosureTextBlock" escape="true">
///     <p>... par value of $<ix:nonFraction name="us-gaap:CommonStockParOrStatedValuePerShare">0.0001</ix:nonFraction> ...</p>
/// </ix:nonNumeric>
/// ```
///
/// Every figure in the notes to the financial statements is of the second
/// kind. A nested fact is registered before the fact that contains it, and its
/// text is part of the outer fact's.
fn read_inline_fact(
    e: &BytesStart,
    lowercase_tag: &str,
    reader: &mut Reader<&[u8]>,
    xbrl: &mut Instance,
    inline: &mut InlineState,
) -> Result<usize> {
    let (mut fact, attrs) = parse_fact_attributes_common(e, true);
    fact.local_name = local_part(&fact.full_name).to_string();

    // The content is consumed even for a nil fact, so that the reader ends up
    // past the closing tag either way.
    let text = read_inline_content(reader, lowercase_tag, xbrl, inline)?;

    inline.pending.push((xbrl.facts.len(), attrs, text));
    xbrl.facts.push(fact);
    Ok(inline.pending.len() - 1)
}

/// Collects the text inside an inline element, consuming events up to and
/// including its end tag. Facts met on the way are registered on `xbrl`.
fn read_inline_content(
    reader: &mut Reader<&[u8]>,
    lowercase_end_tag: &str,
    xbrl: &mut Instance,
    inline: &mut InlineState,
) -> Result<String> {
    let mut text = String::new();
    let mut buf = Vec::new();
    // Elements of the same name opened inside this one. Inline facts recurse,
    // so this only counts HTML that happens to repeat the tag being closed.
    let mut same_name_depth = 0usize;

    loop {
        buf.clear();
        match reader
            .read_event_into(&mut buf)
            .map_err(XbrlError::malformed)?
        {
            Event::Text(chunk) => text.push_str(&decode_text(&chunk)),
            Event::CData(chunk) => text.push_str(&String::from_utf8_lossy(chunk.as_ref())),
            Event::Start(child) => {
                let tag = String::from_utf8_lossy(child.name().as_ref()).to_lowercase();

                if is_inline_fact_tag(&tag) {
                    let nested = read_inline_fact(&child, &tag, reader, xbrl, inline)?;
                    text.push_str(&inline.pending[nested].2);
                } else if tag == "ix:exclude" {
                    // Its text is not part of the value; a fact inside it is still a fact.
                    read_inline_content(reader, &tag, xbrl, inline)?;
                } else if tag == "ix:continuation" {
                    read_continuation(&child, reader, xbrl, inline)?;
                } else {
                    if tag == lowercase_end_tag {
                        same_name_depth += 1;
                    }
                    if is_block_tag(&tag) {
                        text.push('\n');
                    }
                }
            }
            Event::End(end) => {
                let tag = String::from_utf8_lossy(end.name().as_ref()).to_lowercase();
                if tag == lowercase_end_tag {
                    if same_name_depth == 0 {
                        break;
                    }
                    same_name_depth -= 1;
                } else if is_block_tag(&tag) {
                    text.push('\n');
                }
                // Any other end tag is HTML closing, or malformed HTML: carry on.
            }
            Event::Eof => break,
            _ => { /* comments, processing instructions */ }
        }
    }

    Ok(text)
}

/// Reads an `ix:continuation` and files its text under its id.
fn read_continuation(
    e: &BytesStart,
    reader: &mut Reader<&[u8]>,
    xbrl: &mut Instance,
    inline: &mut InlineState,
) -> Result<()> {
    let id = attribute(e, "id");
    let continued_at = attribute(e, "continuedat");
    let text = read_inline_content(reader, "ix:continuation", xbrl, inline)?;
    if let Some(id) = id {
        inline.continuations.insert(id, (text, continued_at));
    }
    Ok(())
}

/// Settles the value of every inline fact once the whole document has been read.
fn settle_inline_facts(xbrl: &mut Instance, inline: InlineState) {
    // A filing chains a handful of continuations; the limit is only there so a
    // malformed cycle cannot spin.
    const MAX_CONTINUATIONS: usize = 4096;

    let InlineState {
        pending,
        continuations,
    } = inline;

    for (index, attrs, mut text) in pending {
        if attrs.is_nil {
            continue;
        }

        let mut next = attrs.continued_at.as_deref();
        for _ in 0..MAX_CONTINUATIONS {
            let Some((more, then)) = next.and_then(|id| continuations.get(id)) else {
                break;
            };
            text.push('\n');
            text.push_str(more);
            next = then.as_deref();
        }

        let fact = &mut xbrl.facts[index];
        fact.value = inline_value(&text, fact.format.as_deref(), &attrs);
    }
}

/// Turns the text collected for an inline fact into its value: whitespace
/// normalised, the `format` transformation applied, then `scale`, then `sign`.
///
/// Text that comes to nothing is `Nil` rather than an empty string, so that a
/// tag with no content does not surface downstream as "could not parse '' as f64".
fn inline_value(raw: &str, format: Option<&str>, attrs: &FactAttrs) -> XbrlValue {
    let text = if attrs.escape {
        normalize_lines(raw)
    } else {
        collapse_whitespace(raw)
    };

    let transformed = match format {
        // A transformation reads one line of text: "September 30, 2025".
        Some(format) => {
            let single_line = collapse_whitespace(&text);
            transformations::apply_transformation(&single_line, format).unwrap_or(text)
        }
        None => text,
    };

    let scaled = match attrs.scale {
        Some(scale) => shift_decimal(&transformed, scale).unwrap_or(transformed),
        None => transformed,
    };

    let value = if attrs.negative {
        negate(scaled)
    } else {
        scaled
    };

    if value.trim().is_empty() {
        XbrlValue::Nil
    } else {
        XbrlValue::String(value)
    }
}

/// Joins text into one line: every run of whitespace becomes a single space.
fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Keeps a block of text as lines: each line collapsed, empty lines dropped.
fn normalize_lines(text: &str) -> String {
    text.lines()
        .map(collapse_whitespace)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Multiplies a decimal number by ten to the power `scale`, by moving its
/// decimal point. Returns `None` when the text is not a plain decimal number.
///
/// Done on the digits rather than through `f64` so that `5.5` at scale `-2`
/// is `0.055` and not `0.055000000000000004`.
fn shift_decimal(value: &str, scale: i32) -> Option<String> {
    let value = value.trim();
    let (negative, unsigned) = match value.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, value),
    };
    let (int, frac) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    if int.is_empty() && frac.is_empty() {
        return None;
    }
    if !int.bytes().chain(frac.bytes()).all(|b| b.is_ascii_digit()) {
        return None;
    }

    let mut digits = format!("{int}{frac}");
    let mut point = int.len() as i64 + i64::from(scale);
    if point <= 0 {
        digits.insert_str(0, &"0".repeat((1 - point) as usize));
        point = 1;
    }
    let point = point as usize;
    if point > digits.len() {
        digits.push_str(&"0".repeat(point - digits.len()));
    }

    let (int, frac) = digits.split_at(point);
    let int = int.trim_start_matches('0');
    let int = if int.is_empty() { "0" } else { int };
    let frac = frac.trim_end_matches('0');

    let mut shifted = String::with_capacity(digits.len() + 2);
    if negative && (int != "0" || !frac.is_empty()) {
        shifted.push('-');
    }
    shifted.push_str(int);
    if !frac.is_empty() {
        shifted.push('.');
        shifted.push_str(frac);
    }
    Some(shifted)
}

/// Applies iXBRL `sign="-"` to a value. Zero stays unsigned.
fn negate(value: String) -> String {
    if let Some(positive) = value.strip_prefix('-') {
        return positive.to_string();
    }
    let is_zero = value.parse::<f64>().is_ok_and(|number| number == 0.0);
    if is_zero { value } else { format!("-{value}") }
}

/// The event loop both entry points share.
///
/// Dispatches each start tag in turn: a context or unit, an inline fact or
/// continuation, or — for anything else with a namespace prefix — a fact of a
/// traditional instance. `should_break` ends the loop: at `</xbrl>` for an
/// instance, at the end of the file for an inline document. In an
/// `inline_document`, a prefixed tag is only a fact if it names its context.
fn process_events<F>(
    reader: &mut Reader<&[u8]>,
    xbrl: &mut Instance,
    inline_document: bool,
    mut should_break: F,
) -> Result<()>
where
    F: FnMut(&Event) -> bool,
{
    let mut buf = Vec::new();
    let mut inline = InlineState::default();
    loop {
        buf.clear();
        let event = reader
            .read_event_into(&mut buf)
            .map_err(XbrlError::malformed)?;

        if should_break(&event) {
            break;
        }

        match event {
            Event::Start(e) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let lowercase_tag = tag_name.to_lowercase();

                // First, try to handle it as a metadata tag (context, unit) or a link tag.
                if try_handle_metadata_or_link(reader, xbrl, &e, &tag_name)? {
                    continue;
                }

                // If not metadata, check if it's an iXBRL fact tag, or the
                // continuation of one. Both consume up to and including their
                // closing tag. Whitespace inside them is part of the value —
                // it is what separates "September 30," from "2025" — so the
                // reader stops trimming text for as long as one is open.
                let is_continuation = lowercase_tag == "ix:continuation";
                if is_continuation || is_inline_fact_tag(&lowercase_tag) {
                    reader.config_mut().trim_text(false);
                    let read = if is_continuation {
                        read_continuation(&e, reader, xbrl, &mut inline)
                    } else {
                        read_inline_fact(&e, &lowercase_tag, reader, xbrl, &mut inline).map(|_| ())
                    };
                    reader.config_mut().trim_text(true);
                    read?;
                    continue;
                }

                // Check if this looks like an XBRL fact (has namespace prefix, not an HTML tag)
                // For traditional XML, any tag with a namespace prefix could be a fact.
                // An HTML document carries its facts in `ix:` tags, so there a
                // prefixed tag is only a fact when it says which context it
                // belongs to — `<o:p>` and friends are word-processor residue.
                let lc_tag = lowercase_tag.as_str();
                if tag_name.contains(':')
                    && !lc_tag.starts_with("ix:")
                    && !lc_tag.starts_with("link:")
                    && !lc_tag.starts_with("html")
                    && (!inline_document || attribute(&e, "contextref").is_some())
                {
                    handle_start_event(reader, xbrl, e, tag_name)?;
                }
                // Otherwise, ignore HTML container tags and other non-XBRL tags
            }
            Event::Empty(e) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                handle_empty_event(xbrl, e, tag_name);
            }
            Event::Eof => break,
            _ => { /* Ignore other events */ }
        }
    }
    settle_inline_facts(xbrl, inline);
    Ok(())
}

/// Reads a fact of a traditional instance: an element named for its concept.
fn handle_start_event(
    reader: &mut Reader<&[u8]>,
    xbrl: &mut Instance,
    e: BytesStart,
    tag_name: String,
) -> Result<()> {
    let (mut fact, attrs) = parse_fact_attributes_common(&e, false);
    let is_explicitly_nil = attrs.is_nil;
    // In an instance the concept is the tag name.
    fact.local_name = local_part(&tag_name).to_string();
    fact.full_name = tag_name;

    if is_explicitly_nil {
        // RawFact is explicitly marked as nil - skip content and mark as nil
        fact.value = XbrlValue::Nil;
        reader
            .read_to_end_into(e.name(), &mut Vec::new())
            .map_err(XbrlError::malformed)?;
    } else {
        // Extract text content and infer type
        let mut text_buf = Vec::new();
        match reader
            .read_event_into(&mut text_buf)
            .map_err(XbrlError::malformed)?
        {
            Event::Text(text) => {
                let value_str = text.unescape().map_err(XbrlError::malformed)?.into_owned();
                fact.value = parse_typed_value(&value_str);
                // Consume the closing tag
                reader
                    .read_to_end_into(e.name(), &mut Vec::new())
                    .map_err(XbrlError::malformed)?;
            }
            Event::End(end_tag) if end_tag.name() == e.name() => {
                // Empty element (no text content) is considered Nil
                fact.value = XbrlValue::Nil;
            }
            _ => {
                // Complex content - skip for now
                reader
                    .read_to_end_into(e.name(), &mut Vec::new())
                    .map_err(XbrlError::malformed)?;
            }
        }
    }

    xbrl.facts.push(fact);

    Ok(())
}

/// Reads a self-closing element as a fact with no value:
/// `<us-gaap:PreferredStockValue contextRef="c1" unitRef="usd" xsi:nil="true"/>`.
fn handle_empty_event(xbrl: &mut Instance, e: BytesStart, tag_name: String) {
    // Skip link elements
    if tag_name.starts_with("link:") {
        return;
    }

    let (mut fact, _) = parse_fact_attributes_common(&e, false);
    fact.local_name = local_part(&tag_name).to_string();
    fact.full_name = tag_name;
    fact.value = XbrlValue::Nil; // Empty elements are considered nil

    xbrl.facts.push(fact);
}

/// The text of an instance fact as a value: trimmed, and nil when empty.
fn parse_typed_value(value_str: &str) -> XbrlValue {
    let trimmed_val = value_str.trim();
    if trimmed_val.is_empty() {
        XbrlValue::Nil
    } else {
        XbrlValue::String(trimmed_val.to_string())
    }
}

/// Copies an element and everything in it back out as a string, for serde to
/// read a context or a unit from. Consumes through the element's end tag.
fn reconstruct_element(
    reader: &mut Reader<&[u8]>,
    start_event: &BytesStart,
    tag_name: &str,
) -> Result<String> {
    let mut writer = Writer::new(Vec::new());

    // Write the opening tag
    writer
        .write_event(Event::Start(start_event.clone()))
        .map_err(XbrlError::malformed)?;

    let mut depth = 0;
    let mut buf = Vec::new();

    loop {
        buf.clear();
        let event = reader
            .read_event_into(&mut buf)
            .map_err(XbrlError::malformed)?;

        match &event {
            Event::End(e) if e.name().as_ref() == tag_name.as_bytes() && depth == 0 => {
                // Found matching closing tag at root level
                writer.write_event(event).map_err(XbrlError::malformed)?;
                break;
            }
            Event::Start(_) => {
                // Entering nested element
                depth += 1;
            }
            Event::End(_) => {
                // Exiting nested element
                depth -= 1;
            }
            Event::Eof => {
                return Err(XbrlError::malformed(format!(
                    "the document ends inside <{tag_name}>"
                )));
            }
            _ => {
                // Copy all other events as-is
            }
        }

        writer.write_event(event).map_err(XbrlError::malformed)?;
    }

    // Convert the written bytes back to a string
    let xml_bytes = writer.into_inner();
    String::from_utf8(xml_bytes).map_err(XbrlError::malformed)
}

/// Reads a context or a unit, or skips a `link:` element (a schema or
/// linkbase reference, which this crate does not follow). Returns whether the
/// tag was one of these.
///
/// A context or unit serde cannot make sense of is dropped, not fatal.
fn try_handle_metadata_or_link(
    reader: &mut Reader<&[u8]>,
    xbrl: &mut Instance,
    e: &BytesStart,
    tag_name: &str,
) -> Result<bool> {
    let lowercase_tag = tag_name.to_lowercase();
    // XBRL instance documents declare the instance namespace as the *default*
    // namespace (`xmlns="http://www.xbrl.org/2003/instance"`), so contexts and
    // units arrive as bare `<context>`/`<unit>` tags. Matching only on the
    // prefixed spelling silently dropped every context in those documents, which
    // in turn disabled all period-aware fact selection. Compare on the local part
    // instead, and require the mandatory `id` attribute so the unprefixed match
    // cannot swallow unrelated markup while scanning iXBRL HTML.
    let local_tag = local_part(&lowercase_tag);

    if local_tag == "context" && has_id_attribute(e) {
        let element_xml = reconstruct_element(reader, e, tag_name)?;
        if let Ok(context) = from_str(&element_xml) {
            xbrl.contexts.push(context);
        }
        Ok(true)
    } else if local_tag == "unit" && has_id_attribute(e) {
        let element_xml = reconstruct_element(reader, e, tag_name)?;
        if let Ok(unit) = from_str(&element_xml) {
            xbrl.units.push(unit);
        }
        Ok(true)
    } else if lowercase_tag.starts_with("link:") {
        // Skip XBRL linkbase references - we don't need presentation/calculation links
        reader
            .read_to_end_into(e.name(), &mut Vec::new())
            .map_err(XbrlError::malformed)?;
        Ok(true)
    } else {
        // This was not a metadata or link tag
        Ok(false)
    }
}

/// Reads the attributes of a fact element, ignoring their case: filings
/// write `contextRef` and `contextref` both.
///
/// Returns the fact, without its value, and the [`FactAttrs`] that will
/// shape that value. For an inline fact (`is_ixbrl`) the concept is the
/// `name` attribute; for an instance fact it is the tag, set by the caller.
fn parse_fact_attributes_common(e: &BytesStart, is_ixbrl: bool) -> (RawFact, FactAttrs) {
    let mut fact = RawFact::default();
    let mut attrs = FactAttrs::default();

    for attr in e.attributes().flatten() {
        // Convert attribute key to lowercase for case-insensitive matching
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_lowercase();
        let value_str = String::from_utf8_lossy(&attr.value);

        match key.as_str() {
            // Common attributes
            "contextref" => fact.context_ref = Some(value_str.into_owned()),
            "unitref" => fact.unit_ref = Some(value_str.into_owned()),
            "decimals" => fact.decimals = Some(value_str.into_owned()),
            "id" => fact.id = Some(value_str.into_owned()),
            "format" if is_ixbrl => fact.format = Some(value_str.into_owned()),

            // Format-specific attributes
            "name" if is_ixbrl => fact.full_name = value_str.into_owned(),
            "scale" if is_ixbrl => attrs.scale = value_str.trim().parse::<i32>().ok(),
            "sign" if is_ixbrl => attrs.negative = value_str.trim() == "-",
            "continuedat" if is_ixbrl => attrs.continued_at = Some(value_str.into_owned()),
            "escape" if is_ixbrl => attrs.escape = value_str.trim().eq_ignore_ascii_case("true"),

            // Unified nil handling
            "xsi:nil" | "nil" if value_str.eq_ignore_ascii_case("true") => {
                fact.value = XbrlValue::Nil;
                attrs.is_nil = true;
            }
            _ => {
                // Ignore other attributes like arcrole, title, etc.
            }
        }
    }

    (fact, attrs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_xbrl_root_element() {
        // Test various namespace prefixes for XBRL root element
        assert!(is_xbrl_root_element("xbrl"));
        assert!(is_xbrl_root_element("xbrli:xbrl"));
        assert!(is_xbrl_root_element("xbrl:xbrl"));
        assert!(is_xbrl_root_element("ns:xbrl"));

        // Test non-XBRL elements
        assert!(!is_xbrl_root_element("context"));
        assert!(!is_xbrl_root_element("unit"));
        assert!(!is_xbrl_root_element("xbrl:context"));
    }

    #[test]
    fn the_root_element_tells_an_instance_from_an_inline_document() {
        assert!(is_xml_instance(
            "<xbrl xmlns=\"http://www.xbrl.org/2003/instance\">"
        ));
        assert!(is_xml_instance(
            "<?xml version=\"1.0\"?>\n<!-- <html> in a comment -->\n<xbrli:xbrl>"
        ));
        assert!(!is_xml_instance(
            "<?xml version=\"1.0\"?><!DOCTYPE html><html xmlns:ix=\"http://www.xbrl.org/2013/inlineXBRL\">"
        ));
        assert!(!is_xml_instance("<HTML><body>"));
        assert!(!is_xml_instance("no markup at all"));
        assert!(!is_xml_instance(""));
    }

    #[test]
    fn test_parse_typed_value() {
        // Should preserve string content and trim whitespace
        assert_eq!(
            parse_typed_value("1500000"),
            XbrlValue::String("1500000".to_string())
        );
        assert_eq!(
            parse_typed_value("true"),
            XbrlValue::String("true".to_string())
        );
        assert_eq!(
            parse_typed_value("  See Note 1  "),
            XbrlValue::String("See Note 1".to_string())
        );

        // Should treat empty or whitespace-only strings as Nil
        assert_eq!(parse_typed_value(""), XbrlValue::Nil);
        assert_eq!(parse_typed_value("   "), XbrlValue::Nil);
    }

    #[test]
    fn test_error_handling_malformed_xml() {
        let malformed_content = r#"
        <?xml version="1.0" encoding="utf-8"?>
        <xbrl xmlns="http://www.xbrl.org/2003/instance">
            <context id="c0">
                <!-- Missing closing tag -->
            <context>
        </xbrl>
        "#;

        let result = parse_xml(malformed_content);
        assert!(result.is_err(), "Should fail to parse malformed XML");

        assert!(matches!(result, Err(XbrlError::Malformed(_))));
    }

    #[test]
    fn test_namespace_prefix_root_element() {
        let content_with_namespace = r#"
        <?xml version="1.0" encoding="utf-8"?>
        <xbrli:xbrl xmlns:xbrli="http://www.xbrl.org/2003/instance">
            <xbrli:context id="c0">
                <xbrli:entity>
                    <xbrli:identifier scheme="http://www.sec.gov/CIK">0001234567</xbrli:identifier>
                </xbrli:entity>
                <xbrli:period>
                    <xbrli:instant>2021-03-31</xbrli:instant>
                </xbrli:period>
            </xbrli:context>
        </xbrli:xbrl>
        "#;

        let result = parse_xml(content_with_namespace);
        assert!(result.is_ok(), "Should parse XBRL with namespace prefix");

        let xbrl = result.unwrap();
        assert_eq!(xbrl.contexts.len(), 1);
        assert_eq!(xbrl.contexts[0].id, "c0");
    }

    #[test]
    fn test_missing_root_element() {
        let content_without_root = r#"
        <?xml version="1.0" encoding="utf-8"?>
        <document>
            <context id="c0"/>
        </document>
        "#;

        let result = parse_xml(content_without_root);
        assert!(result.is_err());

        assert!(matches!(result, Err(XbrlError::NotXbrl(_))));
    }
}
