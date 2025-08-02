//! # High-Performance XBRL XML Parser
//!
//! Provides efficient, single-pass parsing of XBRL documents using event-driven XML processing.
//! This module handles the low-level XML parsing and converts raw XBRL content into structured
//! data representations suitable for further processing.
//!
//! ## Parsing Strategy
//!
//! The parser uses a streaming, event-driven approach that provides several advantages:
//!
//! ```text
//!   Raw XBRL Document (10-50MB)
//!            │
//!            ▼
//!   ┌─────────────────────┐
//!   │   Event-Driven      │  <- Single pass through document
//!   │   XML Reader        │
//!   └─────────────────────┘
//!            │
//!            ▼
//!   ┌─────────────────────┐
//!   │   Context & Unit    │  <- Extract structure definitions
//!   │   Definitions       │
//!   └─────────────────────┘
//!            │
//!            ▼
//!   ┌─────────────────────┐
//!   │   Fact Extraction   │  <- Extract and type data values
//!   │   & Type Inference  │
//!   └─────────────────────┘
//!            │
//!            ▼
//!   Structured Xbrl Object
//! ```

use super::error::{Result, XbrlError};
use super::structures::{Fact, Xbrl, XbrlValue};
use quick_xml::{Reader, Writer, de::from_str, events::BytesStart, events::Event};

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
/// use std::fs;
///
/// let content = fs::read_to_string("sample-10q.xml")?;
/// let xbrl_data = extract_xbrl_data(&content)?;
///
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

    // Find and process the root <xbrl> element
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) if e.name().as_ref() == b"xbrl" => {
                process_xbrl_children(&mut reader, &mut xbrl)?;
                break;
            }
            Event::Eof => {
                return Err(XbrlError::DeserializationError(
                    "Could not find root <xbrl> tag in document".to_string(),
                ));
            }
            _ => {
                // Continue searching for root element
            }
        }
        buf.clear();
    }

    Ok(xbrl)
}

/// Processes all child elements of the root `<xbrl>` tag
///
/// This function iterates through the children of the XBRL root element,
/// routing each element to the appropriate handler based on its type.
/// The processing maintains document order while building the structured
/// representation.
///
/// # Processing Logic
///
/// - **Contexts**: Parsed and stored for later fact resolution
/// - **Units**: Parsed and stored for numeric value interpretation  
/// - **Facts**: Extracted with type inference and context association
/// - **Links**: Ignored (presentation/calculation linkbases not needed)
/// - **Other**: Treated as potential facts with full processing
///
/// # Arguments
///
/// * `reader` - Mutable reference to the XML reader
/// * `xbrl` - Mutable reference to the XBRL structure being built
///
/// # Returns
///
/// * `Result<()>` - Success or parsing error
fn process_xbrl_children(reader: &mut Reader<&[u8]>, xbrl: &mut Xbrl) -> Result<()> {
    let mut buf = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                handle_start_event(reader, xbrl, e, tag_name)?;
            }
            Event::Empty(e) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                handle_empty_event(xbrl, e, tag_name);
            }
            Event::End(e) if e.name().as_ref() == b"xbrl" => {
                // End of root element - parsing complete
                break;
            }
            Event::Eof => {
                // Unexpected end of document
                break;
            }
            _ => {
                // Ignore text, comments, etc. at root level
            }
        }
    }

    Ok(())
}

/// Handles XML start tags with content `<tag>...</tag>`
///
/// This function processes XML elements that have content between opening
/// and closing tags. It routes different element types to appropriate
/// processing logic based on the tag name.
///
/// # Processing Strategy
///
/// - **Context/Unit Elements**: Use full XML reconstruction for serde parsing
/// - **Link Elements**: Skip entirely (not needed for financial data)
/// - **Fact Elements**: Extract attributes and text content with type inference
///
/// # Arguments
///
/// * `reader` - Mutable reference to the XML reader
/// * `xbrl` - Mutable reference to the XBRL structure
/// * `e` - The start tag event
/// * `tag_name` - The tag name as a string
///
/// # Returns
///
/// * `Result<()>` - Success or parsing error
fn handle_start_event(
    reader: &mut Reader<&[u8]>,
    xbrl: &mut Xbrl,
    e: BytesStart,
    tag_name: String,
) -> Result<()> {
    match tag_name.as_str() {
        // Handle context definitions - these are complex structures that need full XML parsing
        "context" => {
            let element_xml = reconstruct_element(reader, &e, &tag_name)?;
            if let Ok(context) = from_str(&element_xml) {
                xbrl.contexts.push(context);
            }
            // Note: We silently ignore contexts that fail to parse rather than
            // failing the entire document, as some contexts may be malformed
            // but the document may still contain usable data
        }

        // Handle unit definitions - similar to contexts
        "unit" => {
            let element_xml = reconstruct_element(reader, &e, &tag_name)?;
            if let Ok(unit) = from_str(&element_xml) {
                xbrl.units.push(unit);
            }
        }

        // Skip XBRL linkbase references - we don't need presentation/calculation links
        _ if tag_name.starts_with("link:") => {
            reader.read_to_end_into(e.name(), &mut Vec::new())?;
        }

        // Handle all other elements as potential facts
        _ => {
            let (mut fact, is_explicitly_nil) = parse_fact_attributes(&e);
            fact.full_name = tag_name;
            fact.local_name = fact.full_name.split(':').last().unwrap_or("").to_string();

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
                        fact.value = parse_typed_value(&value_str, fact.decimals.as_deref());
                        // Consume the closing tag
                        reader.read_to_end_into(e.name(), &mut Vec::new())?;
                    }
                    Event::End(end_tag) if end_tag.name() == e.name() => {
                        // Empty element (no text content)
                        fact.value = XbrlValue::String("".to_string());
                    }
                    _ => {
                        // Complex content - skip for now
                        reader.read_to_end_into(e.name(), &mut Vec::new())?;
                    }
                }
            }

            xbrl.facts.push(fact);
        }
    }

    Ok(())
}

/// Handles self-closing XML tags `<tag/>`
///
/// Self-closing tags in XBRL typically represent facts with no text content,
/// often used for flags or when the presence of the element itself conveys
/// information.
///
/// # Arguments
///
/// * `xbrl` - Mutable reference to the XBRL structure
/// * `e` - The empty tag event
/// * `tag_name` - The tag name as a string
fn handle_empty_event(xbrl: &mut Xbrl, e: BytesStart, tag_name: String) {
    // Skip link elements
    if tag_name.starts_with("link:") {
        return;
    }

    // Treat as a fact with no value
    let (mut fact, _) = parse_fact_attributes(&e);
    fact.full_name = tag_name;
    fact.local_name = fact.full_name.split(':').last().unwrap_or("").to_string();
    fact.value = XbrlValue::Nil; // Empty elements are considered nil

    xbrl.facts.push(fact);
}

/// Extracts and parses attributes from a fact element
///
/// XBRL facts contain several important attributes that provide metadata
/// about how to interpret the fact value. This function extracts these
/// attributes and determines if the fact is explicitly marked as nil.
///
/// # Key Attributes
///
/// - **contextRef**: Links to temporal/dimensional context
/// - **unitRef**: Links to unit of measure definition  
/// - **decimals**: Indicates precision for numeric values
/// - **id**: Unique identifier for cross-references
/// - **xsi:nil**: Explicit nil value indicator
///
/// # Arguments
///
/// * `e` - The XML start tag containing attributes
///
/// # Returns
///
/// * `(Fact, bool)` - Tuple of parsed fact and nil flag
fn parse_fact_attributes(e: &BytesStart) -> (Fact, bool) {
    let mut fact = Fact::default();
    let mut is_explicitly_nil = false;

    // Parse all attributes
    for attr_result in e.attributes() {
        if let Ok(attr) = attr_result {
            match attr.key.as_ref() {
                b"contextRef" => {
                    fact.context_ref = Some(String::from_utf8_lossy(&attr.value).into_owned());
                }
                b"unitRef" => {
                    fact.unit_ref = Some(String::from_utf8_lossy(&attr.value).into_owned());
                }
                b"decimals" => {
                    fact.decimals = Some(String::from_utf8_lossy(&attr.value).into_owned());
                }
                b"id" => {
                    fact.id = Some(String::from_utf8_lossy(&attr.value).into_owned());
                }
                b"xsi:nil" if String::from_utf8_lossy(&attr.value) == "true" => {
                    fact.value = XbrlValue::Nil;
                    is_explicitly_nil = true;
                }
                _ => {
                    // Ignore other attributes (precision, scale, etc.)
                }
            }
        }
    }

    (fact, is_explicitly_nil)
}

/// Converts a string value to the appropriate typed XbrlValue
///
/// This function implements intelligent type inference for XBRL fact values,
/// using context clues from the content and attributes to determine the most
/// appropriate Rust type representation.
///
/// # Type Inference Logic
///
/// 1. **Numeric with decimals attribute**: Parse as f64 or i64 based on decimals value
/// 2. **Boolean patterns**: "true"/"false", "yes"/"no" become Bool variants
/// 3. **Numeric patterns**: Integer or decimal strings become numeric types
/// 4. **Default**: All other content becomes String variant
///
/// # Arguments
///
/// * `value_str` - The raw string value from XML content
/// * `decimals` - Optional decimals attribute value for numeric interpretation
///
/// # Returns
///
/// * `XbrlValue` - Appropriately typed value representation
///
/// # Example
///
/// ```rust
/// use xbrl::parser::parse_typed_value;
///
/// let monetary = parse_typed_value("1500000", Some("0"));      // XbrlValue::F64(1500000.0)
/// let shares = parse_typed_value("1000000", Some("INF"));      // XbrlValue::I64(1000000)
/// let flag = parse_typed_value("true", None);                  // XbrlValue::Bool(true)
/// let text = parse_typed_value("See Note 1", None);            // XbrlValue::String("See Note 1")
/// ```
fn parse_typed_value(value_str: &str, decimals: Option<&str>) -> XbrlValue {
    let trimmed_val = value_str.trim();

    // Handle numeric values based on decimals attribute
    if let Some(d) = decimals {
        if d.eq_ignore_ascii_case("inf") {
            // INF decimals indicates integer values (like share counts)
            if let Ok(i) = trimmed_val.parse::<i64>() {
                return XbrlValue::I64(i);
            }
        } else {
            // Specific decimal precision indicates floating-point monetary values
            if let Ok(f) = trimmed_val.parse::<f64>() {
                return XbrlValue::F64(f);
            }
        }
    }

    // Handle boolean patterns (case-insensitive)
    let lower_val = trimmed_val.to_lowercase();
    if lower_val == "true" || lower_val == "yes" {
        return XbrlValue::Bool(true);
    }
    if lower_val == "false" || lower_val == "no" {
        return XbrlValue::Bool(false);
    }

    // Try parsing as number even without decimals attribute
    if let Ok(f) = trimmed_val.parse::<f64>() {
        return XbrlValue::F64(f);
    }
    if let Ok(i) = trimmed_val.parse::<i64>() {
        return XbrlValue::I64(i);
    }

    // Default to string representation
    XbrlValue::String(trimmed_val.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_typed_value_monetary() {
        let result = parse_typed_value("1500000", Some("0"));
        assert_eq!(result, XbrlValue::F64(1500000.0));
    }

    #[test]
    fn test_parse_typed_value_shares() {
        let result = parse_typed_value("1000000", Some("INF"));
        assert_eq!(result, XbrlValue::I64(1000000));
    }

    #[test]
    fn test_parse_typed_value_boolean() {
        assert_eq!(parse_typed_value("true", None), XbrlValue::Bool(true));
        assert_eq!(parse_typed_value("TRUE", None), XbrlValue::Bool(true));
        assert_eq!(parse_typed_value("false", None), XbrlValue::Bool(false));
        assert_eq!(parse_typed_value("yes", None), XbrlValue::Bool(true));
        assert_eq!(parse_typed_value("no", None), XbrlValue::Bool(false));
    }

    #[test]
    fn test_parse_typed_value_string() {
        let result = parse_typed_value("See Note 1", None);
        assert_eq!(result, XbrlValue::String("See Note 1".to_string()));
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
                assert!(msg.contains("Could not find root <xbrl> tag"));
            }
            _ => panic!("Should return DeserializationError for missing root"),
        }
    }
}
