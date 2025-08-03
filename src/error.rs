//! # XBRL Parser Error Types
//!
//! Comprehensive error handling for XBRL parsing operations, providing detailed
//! error information for debugging and monitoring in production environments.

use serde::de;
use std::fmt::Display;
use thiserror::Error;

/// Comprehensive error types for XBRL parsing and processing operations
///
/// This enum covers all possible error conditions that can occur during XBRL
/// document processing, from low-level XML parsing to high-level semantic validation.
/// Each variant provides specific context to aid in debugging and error recovery.
#[derive(Error, Debug)]
pub enum XbrlError {
    /// XML parsing errors from the underlying quick-xml parser
    ///
    /// These errors occur during the initial XML document parsing phase,
    /// typically due to malformed XML structure, encoding issues, or
    /// unexpected XML constructs.
    #[error("XBRL parsing failed: {0}")]
    ParsingError(#[from] quick_xml::Error),

    /// XML attribute parsing errors
    ///
    /// Occurs when XML attributes cannot be properly decoded or parsed,
    /// often due to encoding issues or malformed attribute values.
    #[error("Attribute parsing failed: {0}")]
    AttributeError(#[from] quick_xml::events::attributes::AttrError),

    /// I/O errors during document processing
    ///
    /// These errors occur when reading XBRL documents from files, network
    /// streams, or other I/O sources.
    #[error("IO error during parsing: {0}")]
    IoError(#[from] std::io::Error),

    /// Type conversion errors for XBRL fact values
    ///
    /// Occurs when attempting to convert XBRL fact values to specific
    /// Rust types (e.g., parsing "123.45" as f64 or "true" as bool).
    #[error("For field '{field_name}': could not parse value '{value}' as {target_type}")]
    ValueConversion {
        /// The name of the struct field being deserialized.
        field_name: String,
        /// The raw value that failed conversion
        value: String,
        /// The target type we attempted to convert to
        target_type: String,
    },

    /// Missing required XBRL facts
    ///
    /// Indicates that a required financial fact or concept was not found
    /// in any valid context within the XBRL document.
    #[error("Required fact '{0}' not found in any context")]
    MissingFact(String),

    /// Context reference resolution errors
    ///
    /// Occurs when an XBRL fact references a context that doesn't exist
    /// in the document's context definitions.
    #[error("Context with id '{0}' not found")]
    ContextNotFound(String),

    /// Unit reference resolution errors
    ///
    /// Occurs when an XBRL fact references a unit definition that doesn't
    /// exist in the document's unit definitions.
    #[error("Unit with id '{0}' not found")]
    UnitNotFound(String),

    /// Serde deserialization errors
    ///
    /// Custom errors that occur during the serde-based deserialization
    /// process when mapping XBRL concepts to Rust structs.
    #[error("Custom deserialization error: {0}")]
    DeserializationError(String),

    /// Generic unsupported operation errors
    ///
    /// Used for XBRL features or constructs that are not yet supported
    /// by the parser implementation.
    #[error("Unsupported XBRL feature: {0}")]
    Unsupported(String),
}

/// Result type alias for XBRL operations
///
/// Provides a convenient shorthand for `Result<T, XbrlError>` used
/// throughout the XBRL parsing codebase.
pub type Result<T> = std::result::Result<T, XbrlError>;

/// Implementation of serde's Error trait for custom deserialization
///
/// This allows XbrlError to be used as a serde deserialization error,
/// enabling seamless integration with the serde-based XBRL concept mapping.
impl de::Error for XbrlError {
    fn custom<T: Display>(msg: T) -> Self {
        XbrlError::DeserializationError(msg.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = XbrlError::ValueConversion {
            field_name: "amount".to_string(),
            value: "invalid_number".to_string(),
            target_type: "f64".to_string(),
        };

        assert!(err.to_string().contains("invalid_number"));
        assert!(err.to_string().contains("f64"));
    }

    #[test]
    fn test_serde_error_trait() {
        let _err: XbrlError = de::Error::custom("test error");
        // Should compile without issues
    }
}
