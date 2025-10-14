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
//!   │ extract_ixbrl_data  │           │ extract_xbrl_data   │
//!   │  (HTML-aware)       │           │   (XML-only)        │
//!   └─────────────────────┘           └─────────────────────┘
//!            │                                     │
//!            └──────────────┬──────────────────────┘
//!                           ▼
//!                  ┌─────────────────┐
//!                  │  Xbrl Structure │  <- Unified representation
//!                  │ (contexts, units│
//!                  │     facts)      │
//!                  └─────────────────┘
//!                           │
//!                           ▼
//!                  ┌─────────────────┐
//!                  │ XbrlDataContext │  <- Queryable interface
//!                  │  (serde_xbrl)   │
//!                  └─────────────────┘
//! ```
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use xbrl::parser::{extract_ixbrl_data, extract_xbrl_data};
//!
//! // Primary approach: Try iXBRL first
//! let content = std::fs::read_to_string("filing.html").unwrap();
//! let xbrl = extract_ixbrl_data(&content)
//!     .or_else(|_| {
//!         // Fallback: Try traditional XBRL XML
//!         extract_xbrl_data(&content)
//!     })
//!     .expect("Failed to parse either iXBRL or XBRL");
//!
//! println!("Parsed {} facts", xbrl.facts.len());
//! ```

use super::error::{Result, XbrlError};
use super::structures::{Fact, Xbrl, XbrlValue};
use super::transformations;
use quick_xml::{
    Reader, Writer,
    de::from_str,
    events::{BytesStart, Event},
};
use std::borrow::Cow;

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
/// * `Result<Xbrl>` - Parsed XBRL structure or parsing error
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
/// use xbrl::parser::extract_xbrl_data;
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
/// let xbrl_data = extract_xbrl_data(content).unwrap();
/// println!("Extracted {} facts from {} contexts",
///          xbrl_data.facts.len(),
///          xbrl_data.contexts.len());
/// ```
pub fn extract_xbrl_data(xml_content: &str) -> Result<Xbrl> {
    let mut reader = Reader::from_str(xml_content);

    // Configure parser for optimal performance
    reader.config_mut().trim_text(true);
    reader.config_mut().expand_empty_elements = false;

    let mut buf = Vec::new();
    let mut xbrl = Xbrl::default();

    // Find the root XBRL element
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if is_xbrl_root_element(&tag_name) {
                    // Use the generic event processor, breaking when we find the closing </xbrl> tag.
                    process_events(&mut reader, &mut xbrl, |event| {
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
                return Err(XbrlError::DeserializationError(
                    "Could not find root XBRL tag (<xbrl> or <xbrli:xbrl>) in document".to_string(),
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
/// * `Result<Xbrl>` - Parsed XBRL structure with facts, contexts, and units
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
/// use xbrl::parser::extract_ixbrl_data;
///
/// let html = std::fs::read_to_string("10-q.html").unwrap();
/// let xbrl_data = extract_ixbrl_data(&html).unwrap();
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
/// - Automatically applies scale attributes to numeric values
/// - Removes formatting (commas) from numeric strings
pub fn extract_ixbrl_data(html_content: &str) -> Result<Xbrl> {
    let mut reader = Reader::from_str(html_content);
    reader.config_mut().trim_text(true);
    reader.config_mut().expand_empty_elements = true;
    // Allow malformed HTML - important for handling real-world SEC filings
    reader.config_mut().check_end_names = false;
    // Skip unknown HTML entities like &nbsp; instead of erroring
    reader.config_mut().allow_unmatched_ends = true;

    let mut xbrl = Xbrl::default();

    // Use the generic event processor. For iXBRL, we process until the end of the file.
    process_events(&mut reader, &mut xbrl, |event| matches!(event, Event::Eof))?;

    Ok(xbrl)
}

/// Parses an individual iXBRL fact element
///
/// Extracts XBRL fact data from inline tags such as `<ix:nonfraction>`, `<ix:nonnumeric>`,
/// and `<ix:fraction>`. These tags contain both attributes (context reference, unit reference, etc.)
/// and text content (the actual value).
///
/// # Arguments
///
/// * `e` - The start tag event containing attributes
/// * `reader` - Mutable reference to the XML reader for extracting text content
///
/// # Returns
///
/// * `Result<(Fact, bool)>` - Tuple of (parsed Fact, is_nil flag)
///
/// # Attribute Handling
///
/// - `name`: The full concept name (e.g., "us-gaap:Cash")
/// - `contextref`/`contextRef`: Reference to a context definition
/// - `unitref`/`unitRef`: Reference to a unit definition
/// - `decimals`: Decimal precision indicator
/// - `scale`: Power of 10 to multiply the value by
/// - `nil`/`xsi:nil`: Indicates an explicit nil value
/// - `id`: Unique identifier for this fact instance
///
/// # Value Processing
///
/// 1. Extracts text content from the tag
/// 2. Handles HTML entities (falls back to raw text if unescape fails)
/// 3. Removes commas from numeric values (e.g., "4,921" -> "4921")
/// 4. Applies scale transformation (multiplies by 10^scale)
/// 5. Extracts local name from full name (e.g., "us-gaap:Cash" -> "Cash")
///
/// # Example iXBRL Fact
///
/// ```xml
/// <ix:nonfraction name="us-gaap:Cash" contextref="cref_71270353"
///                 unitref="uref_826444437" scale="0" decimals="0"
///                 format="ixt:num-dot-decimal" id="ixv-3389">
///     4,921
/// </ix:nonfraction>
/// ```
///
/// This would be parsed into a Fact with:
/// - `full_name`: "us-gaap:Cash"
/// - `local_name`: "Cash"
/// - `context_ref`: Some("cref_71270353")
/// - `unit_ref`: Some("uref_826444437")
/// - `value`: XbrlValue::String("4921") (after comma removal)
fn parse_ix_fact(e: &BytesStart, reader: &mut Reader<&[u8]>) -> Result<(Fact, bool)> {
    // Use the unified attribute parser
    let (mut fact, is_nil, scale) = parse_fact_attributes_common(e, true);

    // Extract local name from full name (set by the common parser from 'name' attribute)
    if let Some(idx) = fact.full_name.find(':') {
        fact.local_name = fact.full_name[idx + 1..].to_string();
    } else {
        fact.local_name = fact.full_name.clone();
    }

    // Get value from text node if not nil
    if !is_nil {
        let mut value_buf = Vec::new();
        match reader.read_event_into(&mut value_buf)? {
            Event::Text(text) => {
                let raw_text_value = text
                    .unescape()
                    .unwrap_or_else(|_| {
                        Cow::Owned(String::from_utf8_lossy(text.as_ref()).into_owned())
                    })
                    .into_owned();

                // --- TRANSFORMATION LOGIC ---
                // Apply format-specific transformations (e.g., num-dot-decimal, boolballotbox)
                let transformed_value = if let Some(format) = &fact.format {
                    transformations::apply_transformation(&raw_text_value, format)
                        .unwrap_or_else(|_| raw_text_value.clone()) // On error, fall back to raw value
                } else {
                    raw_text_value
                };

                // Apply scale attribute if present (e.g., scale="6" means multiply by 10^6)
                let final_value = if let Some(s) = scale {
                    if let Ok(num) = transformed_value.parse::<f64>() {
                        let scaled_num = num * 10f64.powi(s);
                        scaled_num.to_string()
                    } else {
                        transformed_value
                    }
                } else {
                    transformed_value
                };

                fact.value = XbrlValue::String(final_value);
            }
            _ => {
                fact.value = XbrlValue::Nil;
            }
        }
    }

    // Note: We do NOT consume to the end tag here. We let the main event loop handle that.
    // This allows nested facts (if they exist) to be processed by the main loop.
    // The main loop will skip the End event for this tag naturally.

    Ok((fact, is_nil))
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
fn process_events<F>(reader: &mut Reader<&[u8]>, xbrl: &mut Xbrl, mut should_break: F) -> Result<()>
where
    F: FnMut(&Event) -> bool,
{
    let mut buf = Vec::new();
    loop {
        buf.clear();
        let event = reader.read_event_into(&mut buf)?;

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

                // If not metadata, check if it's an iXBRL fact tag.
                if matches!(
                    lowercase_tag.as_str(),
                    "ix:nonfraction" | "ix:fraction" | "ix:nonnumeric"
                ) {
                    let (fact, _) = parse_ix_fact(&e, reader)?;
                    xbrl.facts.push(fact);
                    continue;
                }

                // Check if this looks like an XBRL fact (has namespace prefix, not an HTML tag)
                // For traditional XML, any tag with a namespace prefix could be a fact
                let lc_tag = lowercase_tag.as_str();
                if tag_name.contains(':')
                    && !lc_tag.starts_with("ix:")
                    && !lc_tag.starts_with("link:")
                    && !lc_tag.starts_with("html")
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
    Ok(())
}

/// Handles XML start tags with content `<tag>...</tag>`
///
/// This function is now simplified. It no longer needs to check for metadata,
/// as that is handled by the caller (`process_events`). It only processes facts.
fn handle_start_event(
    reader: &mut Reader<&[u8]>,
    xbrl: &mut Xbrl,
    e: BytesStart,
    tag_name: String,
) -> Result<()> {
    // Use the unified attribute parser
    let (mut fact, is_explicitly_nil, _) = parse_fact_attributes_common(&e, false);
    // For XML, the concept name IS the tag name
    fact.full_name = tag_name;

    if is_explicitly_nil {
        // Fact is explicitly marked as nil - skip content and mark as nil
        fact.value = XbrlValue::Nil;
        reader.read_to_end_into(e.name(), &mut Vec::new())?;
    } else {
        // Extract text content and infer type
        let mut text_buf = Vec::new();
        match reader.read_event_into(&mut text_buf)? {
            Event::Text(text) => {
                let value_str = text.unescape()?.into_owned();
                fact.value = parse_typed_value(&value_str);
                // Consume the closing tag
                reader.read_to_end_into(e.name(), &mut Vec::new())?;
            }
            Event::End(end_tag) if end_tag.name() == e.name() => {
                // Empty element (no text content) is considered Nil
                fact.value = XbrlValue::Nil;
            }
            _ => {
                // Complex content - skip for now
                reader.read_to_end_into(e.name(), &mut Vec::new())?;
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
/// - **Fact Elements**: Treated as nil values with only attribute data preserved
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
fn handle_empty_event(xbrl: &mut Xbrl, e: BytesStart, tag_name: String) {
    // Skip link elements
    if tag_name.starts_with("link:") {
        return;
    }

    // Use the unified attribute parser
    let (mut fact, _, _) = parse_fact_attributes_common(&e, false);
    // For XML, the concept name IS the tag name
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
    writer.write_event(Event::Start(start_event.clone()))?;

    let mut depth = 0;
    let mut buf = Vec::new();

    loop {
        buf.clear();
        let event = reader.read_event_into(&mut buf)?;

        match &event {
            Event::End(e) if e.name().as_ref() == tag_name.as_bytes() && depth == 0 => {
                // Found matching closing tag at root level
                writer.write_event(event)?;
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
                return Err(XbrlError::DeserializationError(format!(
                    "Unexpected end of document while parsing element <{}>",
                    tag_name
                )));
            }
            _ => {
                // Copy all other events as-is
            }
        }

        writer.write_event(event)?;
    }

    // Convert the written bytes back to a string
    let xml_bytes = writer.into_inner();
    String::from_utf8(xml_bytes)
        .map_err(|e| XbrlError::DeserializationError(format!("UTF-8 conversion failed: {}", e)))
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
    xbrl: &mut Xbrl,
    e: &BytesStart,
    tag_name: &str,
) -> Result<bool> {
    let lowercase_tag = tag_name.to_lowercase();

    if lowercase_tag.ends_with(":context") {
        let element_xml = reconstruct_element(reader, e, tag_name)?;
        if let Ok(context) = from_str(&element_xml) {
            xbrl.contexts.push(context);
        }
        Ok(true)
    } else if lowercase_tag.ends_with(":unit") {
        let element_xml = reconstruct_element(reader, e, tag_name)?;
        if let Ok(unit) = from_str(&element_xml) {
            xbrl.units.push(unit);
        }
        Ok(true)
    } else if lowercase_tag.starts_with("link:") {
        // Skip XBRL linkbase references - we don't need presentation/calculation links
        reader.read_to_end_into(e.name(), &mut Vec::new())?;
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
/// Returns a tuple containing:
/// 1. **`Fact`** - Partially populated fact with attributes extracted
/// 2. **`bool`** - `is_nil` flag indicating if the fact is explicitly marked nil
/// 3. **`Option<i32>`** - Optional scale value (iXBRL only) for numeric transformation
///
/// # Example Usage
///
/// ```rust,ignore
/// // For iXBRL facts
/// let (fact, is_nil, scale) = parse_fact_attributes_common(&start_tag, true);
/// // fact.full_name is set from "name" attribute
/// // scale is Some(value) if scale attribute exists
///
/// // For traditional XBRL facts
/// let (mut fact, is_nil, _scale) = parse_fact_attributes_common(&start_tag, false);
/// fact.full_name = tag_name; // Caller must set the concept name from tag
/// ```
///
/// # Nil Handling
///
/// When `xsi:nil="true"` or `nil="true"` is encountered:
/// - `fact.value` is immediately set to `XbrlValue::Nil`
/// - The `is_nil` flag is returned as `true`
/// - Callers should skip content extraction for nil facts
fn parse_fact_attributes_common(e: &BytesStart, is_ixbrl: bool) -> (Fact, bool, Option<i32>) {
    let mut fact = Fact::default();
    let mut is_nil = false;
    let mut scale = None;

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
            "format" if is_ixbrl => fact.format = Some(value_str.into_owned()), // NEW: Capture format

            // Format-specific attributes
            "name" if is_ixbrl => fact.full_name = value_str.into_owned(),
            "scale" if is_ixbrl => {
                if let Ok(s) = value_str.parse::<i32>() {
                    scale = Some(s);
                }
            }

            // Unified nil handling
            "xsi:nil" | "nil" => {
                if value_str.to_lowercase() == "true" {
                    fact.value = XbrlValue::Nil;
                    is_nil = true;
                }
            }
            _ => {
                // Ignore other attributes like arcrole, format, title, etc.
            }
        }
    }

    (fact, is_nil, scale)
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

        let result = extract_xbrl_data(malformed_content);
        assert!(result.is_err(), "Should fail to parse malformed XML");

        match result {
            Err(XbrlError::ParsingError(_)) => {
                // Expected error type for XML parsing issues
            }
            _ => panic!("Should return ParsingError for malformed XML"),
        }
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

        let result = extract_xbrl_data(content_with_namespace);
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

        let result = extract_xbrl_data(content_without_root);
        assert!(result.is_err());

        match result {
            Err(XbrlError::DeserializationError(msg)) => {
                assert!(msg.contains("Could not find root XBRL tag"));
            }
            _ => panic!("Should return DeserializationError for missing root"),
        }
    }
}
