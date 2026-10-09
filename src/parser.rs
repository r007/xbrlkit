//! # High-Performance XBRL and iXBRL Parser
//!
//! Provides efficient, single-pass parsing of both traditional XBRL XML documents and modern
//! iXBRL (Inline XBRL) HTML documents using event-driven processing. This module handles
//! the low-level parsing and converts raw documents into structured data representations.
//!
//! ## Parsing Strategies
//!
//! This module supports two distinct XBRL formats:
//!
//! ### 1. Traditional XBRL (XML)
//!
//! Pure XML documents with clear structure:
//! ```xml
//! <xbrl xmlns="http://www.xbrl.org/2003/instance">
//!   <context id="c1">...</context>
//!   <unit id="usd">...</unit>
//!   <us-gaap:Assets contextRef="c1" unitRef="usd">1000000</us-gaap:Assets>
//! </xbrl>
//! ```
//!
//! ### 2. iXBRL (Inline XBRL in HTML) - **Primary Format**
//!
//! HTML documents with embedded XBRL data tags:
//! ```html
//! <html xmlns:ix="http://www.xbrl.org/2013/inlineXBRL">
//!   <ix:header>
//!     <xbrli:context id="c1">...</xbrli:context>
//!     <xbrli:unit id="usd">...</xbrli:unit>
//!   </ix:header>
//!   <body>
//!     Total assets: <ix:nonfraction name="us-gaap:Assets"
//!                    contextref="c1" unitref="usd" format="ixt:num-dot-decimal">1,000,000</ix:nonfraction>
//!   </body>
//! </html>
//! ```
//!
//! ## Transformation Layer Integration
//!
//! The parser automatically captures `format` attributes from iXBRL tags and applies
//! transformations before storing fact values. This ensures that deserialization
//! receives normalized values:
//!
//! - `format="ixt:num-dot-decimal"` → removes comma separators
//! - `format="ixt-sec:boolballotbox"` → converts checkboxes to booleans
//! - `format="ixt-sec:numwordsen"` → converts English words to numbers
//!
//! See the `transformations` module for complete transformation documentation.
//!
//! ## Architecture
//!
//! ```text
//!   iXBRL HTML Document (Primary)      Traditional XBRL XML (Fallback)
//!            │                                     │
//!            ▼                                     ▼
//!   ┌─────────────────────┐           ┌─────────────────────┐
//!   │ parse_ixbrl  │           │ parse_xml   │
//!   │  (HTML-aware)       │           │   (XML-only)        │
//!   └─────────────────────┘           └─────────────────────┘
//!            │                                     │
//!            └──────────────┬──────────────────────┘
//!                           ▼
//!                  ┌─────────────────┐
//!                  │  Instance Structure │  <- Unified representation
//!                  │ (contexts, units│
//!                  │     facts)      │
//!                  └─────────────────┘
//!                           │
//!                           ▼
//!                  ┌─────────────────┐
//!                  │ Document │  <- Queryable interface
//!                  │  (serde_xbrl)   │
//!                  └─────────────────┘
//! ```
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use xbrlkit::parser::{parse_ixbrl, parse_xml};
//!
//! // Primary approach: Try iXBRL first
//! let content = std::fs::read_to_string("filing.html").unwrap();
//! let xbrl = parse_ixbrl(&content)
//!     .or_else(|_| {
//!         // Fallback: Try traditional XBRL XML
//!         parse_xml(&content)
//!     })
//!     .expect("Failed to parse either iXBRL or XBRL");
//!
//! println!("Parsed {} facts", xbrl.facts.len());
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

/// Checks if a tag name represents an XBRL root element
///
/// This function handles various namespace prefixes that might be used
/// for the XBRL root element, making the parser more flexible.
///
/// # Arguments
///
/// * `tag_name` - The XML tag name to check
///
/// # Returns
///
/// `true` if the tag name represents an XBRL root element
fn is_xbrl_root_element(tag_name: &str) -> bool {
    // Accept various namespace prefixes for the XBRL root element
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

/// Parses an XBRL document using a single-pass, event-driven approach
///
/// This function serves as the primary entry point for XBRL parsing. It processes
/// the entire document in a single pass, extracting contexts, units, and facts
/// while maintaining minimal memory overhead.
///
/// # Arguments
///
/// * `xml_content` - The raw XBRL document content as a string
///
/// # Returns
///
/// * `Result<Instance>` - Parsed XBRL structure or parsing error
///
/// # Errors
///
/// Returns `XbrlError` for various parsing failures:
/// - `ParsingError`: Malformed XML structure
/// - `DeserializationError`: Missing required XBRL root element
/// - `IoError`: Issues reading the document content
///
/// # Example
///
/// ```rust
/// use xbrlkit::parser::parse_xml;
///
/// let content = r#"
/// <?xml version="1.0" encoding="utf-8"?>
/// <xbrl xmlns="http://www.xbrl.org/2003/instance">
///     <context id="c0">
///         <entity>
///             <identifier scheme="http://www.sec.gov/CIK">0001234567</identifier>
///         </entity>
///         <period>
///             <instant>2021-03-31</instant>
///         </period>
///     </context>
/// </xbrl>
/// "#;
///
/// let xbrl_data = parse_xml(content).unwrap();
/// println!("Extracted {} facts from {} contexts",
///          xbrl_data.facts.len(),
///          xbrl_data.contexts.len());
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

/// Parses an iXBRL (Inline XBRL) HTML document using a resilient, single-pass approach
///
/// This function serves as the primary entry point for parsing iXBRL filings, which are
/// SEC HTML documents with embedded XBRL data. It handles real-world SEC filings that
/// often contain malformed HTML and uses robust error recovery strategies.
///
/// # iXBRL Structure
///
/// iXBRL documents contain:
/// - `<ix:header>` with contexts (`<xbrli:context>`) and units (`<xbrli:unit>`)
/// - Inline facts using tags like `<ix:nonfraction>`, `<ix:nonnumeric>`, etc.
/// - Facts embedded directly in the HTML presentation layer
///
/// # Arguments
///
/// * `html_content` - The raw HTML content containing inline XBRL data
///
/// # Returns
///
/// * `Result<Instance>` - Parsed XBRL structure with facts, contexts, and units
///
/// # Errors
///
/// Returns `XbrlError` for critical parsing failures. However, the parser is designed
/// to be resilient and will:
/// - Skip malformed HTML tags (using `check_end_names = false`)
/// - Handle unknown HTML entities gracefully (falls back to raw text)
/// - Continue processing even if individual elements fail to parse
///
/// # Example
///
/// ```rust,no_run
/// use xbrlkit::parser::parse_ixbrl;
///
/// let html = std::fs::read_to_string("10-q.html").unwrap();
/// let xbrl_data = parse_ixbrl(&html).unwrap();
/// println!("Extracted {} facts from {} contexts",
///          xbrl_data.facts.len(),
///          xbrl_data.contexts.len());
/// ```
///
/// # Notes
///
/// - This parser prioritizes robustness over strict validation
/// - Designed specifically for SEC EDGAR iXBRL filings
/// - Handles both uppercase and lowercase tag variations (e.g., `contextref` and `contextRef`)
/// - Applies `scale` and `sign` to numeric values, so a loss printed as
///   `(1,234)` in thousands is the value `-1234000`
/// - A fact's value is the text of everything inside it, whatever HTML wraps
///   it, and continues through any `ix:continuation` it points at
/// - Facts nested inside another fact (every figure in a note that is tagged
///   as a text block) are extracted alongside it
pub fn parse_ixbrl(html_content: &str) -> Result<Instance> {
    let mut reader = Reader::from_str(html_content);
    reader.config_mut().trim_text(true);
    reader.config_mut().expand_empty_elements = true;
    // Allow malformed HTML - important for handling real-world SEC filings
    reader.config_mut().check_end_names = false;
    // Skip unknown HTML entities like &nbsp; instead of erroring
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
/// `<ix:nonNumeric ...><b>NEWCOURT ACQUISITION CORP</b></ix:nonNumeric>`, and a
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
///     <p>... at an exercise price of $<ix:nonFraction name="us-gaap:ClassOfWarrant...">11.50</ix:nonFraction> ...</p>
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

/// A generic, unified event processing loop for both XBRL and iXBRL
///
/// This function iterates through XML events and dispatches them to the appropriate
/// handlers. It takes a `should_break` closure to define the termination condition,
/// allowing it to be used for both full-document (iXBRL) and sub-tree (XML) parsing.
///
/// # Architecture
///
/// This function achieves perfect architectural symmetry between XBRL and iXBRL parsing:
///
/// - **XBRL (XML)**: Processes events until the closing `</xbrl>` tag
/// - **iXBRL (HTML)**: Processes events until `Event::Eof` (end of file)
///
/// Both formats share the same event dispatching logic:
/// 1. Check for metadata tags (context, unit) or link tags
/// 2. Check for iXBRL fact tags (ix:nonfraction, ix:fraction, ix:nonnumeric)
/// 3. Fall back to traditional XBRL fact handling for other start tags
/// 4. Handle empty (self-closing) tags as nil facts
///
/// # Arguments
///
/// * `reader` - Mutable reference to the XML reader
/// * `xbrl` - Mutable reference to the XBRL structure being built
/// * `should_break` - A closure that takes an `Event` and returns `true` to stop processing
///
/// # Returns
///
/// * `Result<()>` - Success or parsing error
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

/// Handles XML start tags with content `<tag>...</tag>`
///
/// This function is now simplified. It no longer needs to check for metadata,
/// as that is handled by the caller (`process_events`). It only processes facts.
fn handle_start_event(
    reader: &mut Reader<&[u8]>,
    xbrl: &mut Instance,
    e: BytesStart,
    tag_name: String,
) -> Result<()> {
    // Use the unified attribute parser
    let (mut fact, attrs) = parse_fact_attributes_common(&e, false);
    let is_explicitly_nil = attrs.is_nil;
    // For XML, the concept name IS the tag name
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

/// Handles XML empty (self-closing) tags `<tag .../>`
///
/// This function processes XML elements that have no content and are self-closing.
/// In XBRL, empty elements typically represent nil values (e.g., missing data points).
///
/// # Processing Strategy
///
/// - **Link Elements**: Skipped entirely (not needed for financial data)
/// - **RawFact Elements**: Treated as nil values with only attribute data preserved
///
/// # Arguments
///
/// * `xbrl` - Mutable reference to the XBRL structure
/// * `e` - The self-closing tag event containing attributes
/// * `tag_name` - The tag name as a string
///
/// # Example
///
/// ```xml
/// <us-gaap:PreferredStockValue contextRef="c1" unitRef="usd" xsi:nil="true"/>
/// ```
///
/// This would be parsed as a fact with `XbrlValue::Nil`.
fn handle_empty_event(xbrl: &mut Instance, e: BytesStart, tag_name: String) {
    // Skip link elements
    if tag_name.starts_with("link:") {
        return;
    }

    // Use the unified attribute parser
    let (mut fact, _) = parse_fact_attributes_common(&e, false);
    // For XML, the concept name IS the tag name
    fact.local_name = local_part(&tag_name).to_string();
    fact.full_name = tag_name;
    fact.value = XbrlValue::Nil; // Empty elements are considered nil

    xbrl.facts.push(fact);
}

/// Converts a string value to a basic XbrlValue.
///
/// This function no longer performs aggressive type inference. It simply
/// trims the string and wraps it in the XbrlValue::String variant. Empty or
/// whitespace-only strings are considered Nil.
///
/// # Arguments
///
/// * `value_str` - The raw string value from XML content
///
/// # Returns
///
/// * `XbrlValue` - A simple, untyped value representation.
fn parse_typed_value(value_str: &str) -> XbrlValue {
    let trimmed_val = value_str.trim();
    if trimmed_val.is_empty() {
        XbrlValue::Nil
    } else {
        XbrlValue::String(trimmed_val.to_string())
    }
}

/// Reconstructs the complete XML string for a complex element
///
/// This function is used for context and unit elements that require
/// full XML structure preservation for proper serde deserialization.
/// It captures the complete element including all nested children.
///
/// # Processing Strategy
///
/// 1. Write the opening tag with all attributes
/// 2. Copy all child content maintaining proper nesting
/// 3. Write the closing tag
/// 4. Return complete XML string for serde processing
///
/// # Arguments
///
/// * `reader` - Mutable reference to the XML reader
/// * `start_event` - The opening tag event
/// * `tag_name` - Name of the tag being reconstructed
///
/// # Returns
///
/// * `Result<String>` - Complete XML string or parsing error
///
/// # Errors
///
/// Returns `XbrlError::DeserializationError` if the element is not properly closed
/// or if XML reconstruction fails.
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

/// Unified handler for metadata (context, unit) and link elements
///
/// This function centralizes the logic for parsing complex elements that are
/// common to both traditional XBRL and iXBRL formats. It handles case-insensitive
/// tag matching and uses serde deserialization for structured elements.
///
/// # Element Types Handled
///
/// ## 1. Context Elements (`*:context`)
/// Defines the circumstances under which a fact applies:
/// - Entity identification (CIK, ticker)
/// - Time period (instant or duration)
/// - Dimensional segments
///
/// ## 2. Unit Elements (`*:unit`)
/// Defines measurement units for numeric facts:
/// - Simple measures (e.g., "iso4217:USD", "xbrli:shares")
/// - Divide units (e.g., "USD/shares" for per-share calculations)
///
/// ## 3. Link Elements (`link:*`)
/// XBRL linkbase references that are skipped:
/// - Presentation links (visual hierarchy)
/// - Calculation links (formulas)
/// - Definition links (relationships)
///
/// # Case Sensitivity
///
/// The function uses case-insensitive matching to handle variations:
/// - `xbrli:context` and `xbrli:Context` both match
/// - `xbrli:unit` and `XBRLI:UNIT` both match
///
/// # Arguments
///
/// * `reader` - Mutable reference to the XML reader for content extraction
/// * `xbrl` - Mutable reference to the XBRL structure for storing parsed elements
/// * `e` - The start tag event containing attributes
/// * `tag_name` - The tag name as a string (with namespace prefix)
///
/// # Returns
///
/// * `Result<bool>` - Returns:
///   - `Ok(true)` if the element was handled (context, unit, or link)
///   - `Ok(false)` if the element was not recognized, indicating the caller should process it
///
/// # Error Handling
///
/// If context or unit deserialization fails, the element is silently skipped
/// rather than causing the entire parse to fail. This provides resilience
/// against malformed metadata that doesn't affect fact extraction.
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

/// Unified, case-insensitive attribute parser for both XBRL and iXBRL facts
///
/// This function centralizes attribute extraction logic, handling case variations
/// and format-specific differences between traditional XBRL and inline XBRL (iXBRL).
/// It implements the DRY principle by eliminating duplicate attribute parsing code.
///
/// # Attribute Handling
///
/// ## Common Attributes (both formats)
/// - **contextRef/contextref**: Reference to a context ID (case-insensitive)
/// - **unitRef/unitref**: Reference to a unit ID (case-insensitive)
/// - **decimals**: Precision indicator for numeric values
/// - **id**: Unique identifier for this fact instance
/// - **xsi:nil/nil**: Indicates an explicit nil value
///
/// ## iXBRL-Specific Attributes (`is_ixbrl = true`)
/// - **name**: Full concept name (e.g., "us-gaap:Cash")
/// - **scale**: Power of 10 multiplier for numeric values (e.g., scale="-3" for thousands)
///
/// ## Traditional XBRL Notes
/// In traditional XBRL, the concept name comes from the tag name itself, not from
/// a "name" attribute, so it must be set by the caller.
///
/// # Case Insensitivity
///
/// The parser converts all attribute keys to lowercase before matching, allowing it
/// to handle SEC filings that use inconsistent casing:
/// - `contextRef`, `contextref`, `ContextRef` all match
/// - `unitRef`, `unitref`, `UnitRef` all match
///
/// # Arguments
///
/// * `e` - The start tag event containing the attributes to parse
/// * `is_ixbrl` - Boolean flag indicating iXBRL format (enables format-specific processing)
///
/// # Returns
///
/// Returns the partially populated [`RawFact`] and the [`FactAttrs`] that shape
/// its value: nil, and for iXBRL `scale`, `sign`, `continuedAt` and `escape`.
///
/// # Example Usage
///
/// ```rust,ignore
/// // For iXBRL facts
/// let (fact, attrs) = parse_fact_attributes_common(&start_tag, true);
/// // fact.full_name is set from "name" attribute
///
/// // For traditional XBRL facts
/// let (mut fact, attrs) = parse_fact_attributes_common(&start_tag, false);
/// fact.full_name = tag_name; // Caller must set the concept name from tag
/// ```
///
/// # Nil Handling
///
/// When `xsi:nil="true"` or `nil="true"` is encountered:
/// - `fact.value` is immediately set to `XbrlValue::Nil`
/// - `FactAttrs::is_nil` is returned as `true`
/// - Callers should skip content extraction for nil facts
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
            "xsi:nil" | "nil" => {
                if value_str.to_lowercase() == "true" {
                    fact.value = XbrlValue::Nil;
                    attrs.is_nil = true;
                }
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
