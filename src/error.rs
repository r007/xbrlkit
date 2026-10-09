//! Error types.

use std::error::Error as StdError;
use thiserror::Error;

/// What can go wrong reading a document, or a struct from one.
///
/// The parser is lenient by design: real filings carry malformed HTML,
/// unknown entities and unclosed tags, and it steps over all of those. An
/// error here means the document could not be read at all.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum XbrlError {
    /// The markup is broken at a point the parser cannot step over.
    #[error("malformed document: {0}")]
    Malformed(#[source] Box<dyn StdError + Send + Sync + 'static>),

    /// The document is well-formed, but it is not XBRL: an XML document with
    /// no `<xbrl>` root, say.
    #[error("not an XBRL document: {0}")]
    NotXbrl(String),

    /// A fact's value does not convert to the type of the field bound to it:
    /// a filer tagging `N/A` under a numeric concept.
    ///
    /// [`Document::extract`](crate::Document::extract) fails on the first of
    /// these; [`Document::extract_lenient`](crate::Document::extract_lenient)
    /// leaves the field empty and returns them alongside the struct.
    #[error("field `{field}` ({concept}): cannot read {value:?} as {target_type}")]
    ValueConversion {
        /// The struct field being filled.
        field: String,
        /// The concept the value was tagged with, e.g. `us-gaap:Assets`.
        concept: String,
        /// The value as the filing gives it, cut to 80 characters.
        value: String,
        /// The Rust type of the field.
        target_type: &'static str,
    },
}

impl XbrlError {
    /// Wraps an error from the XML reader, keeping it as the source without
    /// making the reader's types part of this crate's API.
    pub(crate) fn malformed(source: impl Into<Box<dyn StdError + Send + Sync + 'static>>) -> Self {
        XbrlError::Malformed(source.into())
    }
}

/// `Result<T, XbrlError>`.
pub type Result<T> = std::result::Result<T, XbrlError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_conversion_error_names_the_field_the_concept_and_the_value() {
        let err = XbrlError::ValueConversion {
            field: "assets".to_string(),
            concept: "us-gaap:Assets".to_string(),
            value: "N/A".to_string(),
            target_type: "f64",
        };
        assert_eq!(
            err.to_string(),
            "field `assets` (us-gaap:Assets): cannot read \"N/A\" as f64"
        );
    }

    #[test]
    fn a_malformed_document_keeps_its_cause() {
        let err = XbrlError::malformed("unexpected end of input");
        assert!(err.source().is_some());
        assert!(err.to_string().contains("unexpected end of input"));
    }
}
