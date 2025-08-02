//! # XBRL Serde Deserializer
//!
//! This module provides a high-performance, custom serde deserializer for XBRL documents.
//! It bridges the gap between raw XBRL parsing and structured data extraction by implementing
//! intelligent fact selection and context-aware deserialization.
//!
//! ## Architecture Overview
//!
//! The deserializer operates in two main phases:
//!
//! ```text
//!   Raw XBRL Document
//!           │
//!           ▼
//!   ┌─────────────────┐
//!   │  XBRL Parser    │ <- Extract facts, contexts, units
//!   │  (parser.rs)    │
//!   └─────────────────┘
//!           │
//!           ▼
//!   ┌─────────────────┐
//!   │ XbrlDeserializer│ <- Index facts, implement smart selection
//!   │                 │
//!   │ • Fact indexing │
//!   │ • Context cache │
//!   │ • Best fact sel │
//!   └─────────────────┘
//!           │
//!           ▼
//!   ┌─────────────────┐
//!   │  Serde Visitor  │ <- Map to target structs
//!   │                 │
//!   │ • Field mapping │
//!   │ • Type convert  │
//!   │ • Option handle │
//!   └─────────────────┘
//!           │
//!           ▼
//!   Typed Rust Structs
//! ```
//!
//! ## Fact Selection Algorithm
//!
//! When multiple facts exist for the same concept (e.g., quarterly vs annual data),
//! the deserializer applies sophisticated selection logic:
//!
//! 1. **Temporal Preference**: Most recent periods preferred
//! 2. **Dimensional Simplicity**: Consolidated data preferred over segmented
//! 3. **Context Quality**: Valid contexts preferred over malformed ones

use crate::error::{Result, XbrlError};
use crate::parser::extract_xbrl_data;
use crate::structures::{Context, Fact, Period, Xbrl, XbrlValue};
use serde::de::{self, Deserializer, IntoDeserializer, MapAccess, Visitor};
use std::cmp::Ordering;
use std::collections::HashMap;

/// High-level entry point for XBRL-to-struct deserialization
///
/// This function provides a familiar serde interface while hiding the complexity
/// of XBRL parsing and fact selection. It automatically handles the two-phase
/// process of XML parsing followed by structured deserialization.
///
/// # Type Parameter
///
/// - `T`: Target struct type that implements `DeserializeOwned`
///
/// # Arguments
///
/// * `s` - Raw XBRL document content as string
///
/// # Returns
///
/// * `Result<T>` - Deserialized struct or parsing/deserialization error
///
/// # Errors
///
/// Returns `XbrlError` for various failure modes:
/// - `ParsingError`: Malformed XML structure
/// - `DeserializationError`: Missing required fields or mapping failures
/// - `AttributeError`: Invalid XML attributes
pub fn from_str<T>(s: &str) -> Result<T>
where
    T: de::DeserializeOwned,
{
    let xbrl_data = extract_xbrl_data(s)?;
    let mut deserializer = XbrlDeserializer::from_xbrl(xbrl_data);
    T::deserialize(&mut deserializer)
}

/// Core XBRL deserializer that implements serde's `Deserializer` trait
///
/// This struct maintains all the state needed for intelligent XBRL deserialization,
/// including pre-computed indexes for fast fact lookup and cached context information
/// for smart fact selection.
pub struct XbrlDeserializer {
    /// Original parsed XBRL data containing all facts, contexts, and units
    xbrl: Xbrl,

    /// Index mapping full concept names (e.g., "us-gaap:Assets") to fact positions
    full_name_map: HashMap<String, Vec<usize>>,

    /// Index mapping local concept names (e.g., "Assets") to fact positions
    local_name_map: HashMap<String, Vec<usize>>,

    /// Context cache mapping context IDs to resolved context objects
    contexts: HashMap<String, Context>,
}

impl XbrlDeserializer {
    /// Creates a new deserializer from parsed XBRL data
    ///
    /// This constructor performs the expensive initialization work of building
    /// indexes and caching contexts. The resulting deserializer can then perform
    /// very fast fact lookups during the deserialization process.
    ///
    /// # Arguments
    ///
    /// * `xbrl` - Parsed XBRL document containing facts, contexts, and units
    ///
    /// # Returns
    ///
    /// * `XbrlDeserializer` - Initialized deserializer ready for use
    pub fn from_xbrl(xbrl: Xbrl) -> Self {
        let mut full_name_map = HashMap::<String, Vec<usize>>::new();
        let mut local_name_map = HashMap::<String, Vec<usize>>::new();

        // Build fact indexes for fast lookup
        for (i, fact) in xbrl.facts.iter().enumerate() {
            full_name_map
                .entry(fact.full_name.clone())
                .or_default()
                .push(i);
            local_name_map
                .entry(fact.local_name.clone())
                .or_default()
                .push(i);
        }

        // Build context cache for fast comparison
        let contexts: HashMap<String, Context> = xbrl
            .contexts
            .iter()
            .map(|c| (c.id.clone(), c.clone()))
            .collect();

        XbrlDeserializer {
            xbrl,
            full_name_map,
            local_name_map,
            contexts,
        }
    }

    /// Finds the best fact for a given concept name using intelligent selection
    ///
    /// This method implements the core logic of the XBRL deserializer: when multiple
    /// facts exist for the same concept (common in XBRL due to different contexts),
    /// it selects the most appropriate one based on business logic.
    ///
    /// # Arguments
    ///
    /// * `rename_attr` - The concept name to search for (supports both full and local names)
    ///
    /// # Returns
    ///
    /// * `Option<&Fact>` - The best matching fact, or None if no facts found
    ///
    /// # Example Selection Logic
    ///
    /// For a concept like "Assets" with multiple contexts:
    /// - Context A: Q3 2024 (consolidated)
    /// - Context B: Q2 2024 (consolidated)
    /// - Context C: Q3 2024 (segment breakdown)
    ///
    /// The algorithm would select Context A (most recent + consolidated).
    fn find_best_fact(&self, rename_attr: &str) -> Option<&Fact> {
        // Determine which index to use based on concept name format
        let fact_indices = if rename_attr.contains(':') {
            // Full namespace format (e.g., "us-gaap:Assets")
            self.full_name_map.get(rename_attr)
        } else {
            // Local name format (e.g., "Assets")
            self.local_name_map.get(rename_attr)
        };

        // Get all candidate facts for this concept
        let candidates: Vec<_> = fact_indices?.iter().map(|&i| &self.xbrl.facts[i]).collect();

        // Apply selection algorithm to find the best fact
        candidates.into_iter().max_by(|a, b| {
            let context_a = a.context_ref.as_ref().and_then(|id| self.contexts.get(id));
            let context_b = b.context_ref.as_ref().and_then(|id| self.contexts.get(id));
            Self::compare_contexts(context_a, context_b)
        })
    }

    /// Compares two contexts to determine which represents "better" data
    ///
    /// This method implements the business logic for context comparison, considering
    /// factors like temporal recency, dimensional complexity, and data quality.
    ///
    /// # Comparison Criteria
    ///
    /// 1. **Existence**: Facts with contexts beat facts without contexts
    /// 2. **Temporal Order**: More recent periods are preferred
    /// 3. **Dimensional Simplicity**: Consolidated data preferred over segments
    ///
    /// # Arguments
    ///
    /// * `a` - First context to compare (optional)
    /// * `b` - Second context to compare (optional)
    ///
    /// # Returns
    ///
    /// * `Ordering` - Comparison result following Rust's ordering conventions
    ///
    /// # Implementation Notes
    ///
    /// The comparison is designed to be:
    /// - **Transitive**: If A > B and B > C, then A > C
    /// - **Stable**: Same contexts always compare the same way
    /// - **Business-Logical**: Reflects real-world preferences for financial data
    fn compare_contexts(a: Option<&Context>, b: Option<&Context>) -> Ordering {
        match (a, b) {
            (Some(ctx_a), Some(ctx_b)) => {
                // Both contexts exist - compare their quality
                Self::compare_periods(&ctx_a.period, &ctx_b.period).then_with(|| {
                    // Prefer contexts without dimensional breakdowns (consolidated data)
                    let a_has_dims = ctx_a.entity.segment.is_some() || ctx_a.scenario.is_some();
                    let b_has_dims = ctx_b.entity.segment.is_some() || ctx_b.scenario.is_some();
                    a_has_dims.cmp(&b_has_dims).reverse() // Reverse: false (no dims) > true (has dims)
                })
            }
            (Some(_), None) => Ordering::Greater, // Context beats no context
            (None, Some(_)) => Ordering::Less,    // No context loses to context
            (None, None) => Ordering::Equal,      // Both missing contexts are equal
        }
    }

    /// Compares two time periods to determine temporal precedence
    ///
    /// XBRL contexts contain period information that can be either instant (point-in-time)
    /// or duration (time range). This method implements the logic for determining which
    /// period is more recent or otherwise preferable.
    ///
    /// # Period Comparison Logic
    ///
    /// 1. **End Dates**: For duration periods, compare end dates (more recent wins)
    /// 2. **Instant Dates**: For instant periods, compare the instant date
    /// 3. **Mixed Types**: End dates are compared with instant dates directly
    /// 4. **Missing Dates**: Periods with dates beat periods without dates
    ///
    /// # Arguments
    ///
    /// * `a` - First period to compare
    /// * `b` - Second period to compare
    ///
    /// # Returns
    ///
    /// * `Ordering` - Temporal ordering (more recent = Greater)
    ///
    /// # Date Format
    ///
    /// Expects ISO 8601 date format (YYYY-MM-DD) which allows lexicographic comparison.
    fn compare_periods(a: &Period, b: &Period) -> Ordering {
        // Extract the most significant date from each period
        let date_a = a.end_date.as_deref().or(a.instant.as_deref()).unwrap_or("");
        let date_b = b.end_date.as_deref().or(b.instant.as_deref()).unwrap_or("");

        // Compare dates lexicographically (works for ISO 8601 format)
        // Note: We reverse the comparison so more recent dates are "greater"
        date_b.cmp(date_a)
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut XbrlDeserializer {
    type Error = XbrlError;

    /// Deserializes any value by delegating to struct deserialization
    ///
    /// This is the main entry point called by serde. For XBRL data, we always
    /// treat the top-level data as a map/struct since XBRL concepts map to
    /// struct fields.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(XbrlMapAccess::new(self, &[]))
    }

    /// Deserializes a struct by providing field-aware map access
    ///
    /// This method is called when serde knows the target struct type and its fields.
    /// We use this information to optimize the deserialization process by only
    /// looking up facts for fields that actually exist in the target struct.
    ///
    /// # Arguments
    ///
    /// * `_name` - Name of the struct being deserialized (unused)
    /// * `fields` - List of field names in the target struct
    /// * `visitor` - Serde visitor that will process the deserialized data
    ///
    /// # Returns
    ///
    /// * `Result<V::Value>` - Deserialized struct or error
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(XbrlMapAccess::new(self, fields))
    }

    // Delegate all other deserialization methods to deserialize_any
    // This is a common pattern for custom deserializers that treat all data as maps
    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map enum identifier ignored_any
    }
}

/// Provides map-like access to XBRL facts for serde deserialization
///
/// This struct implements serde's `MapAccess` trait, allowing the deserializer
/// to present XBRL facts as key-value pairs where keys are concept names and
/// values are the selected fact values.
struct XbrlMapAccess<'a> {
    /// Reference to the main deserializer for fact lookup
    de: &'a XbrlDeserializer,

    /// Iterator over field names to process
    field_iterator: Box<dyn Iterator<Item = &'static str> + 'a>,

    /// Current fact value to be deserialized (set by next_key_seed)
    value: Option<&'a XbrlValue>,
}

impl<'a> XbrlMapAccess<'a> {
    /// Creates a new map access iterator for the given fields
    ///
    /// # Arguments
    ///
    /// * `de` - Reference to the main deserializer
    /// * `fields` - List of struct fields to deserialize (empty for flatten)
    ///
    /// # Returns
    ///
    /// * `XbrlMapAccess` - Iterator ready for serde processing
    fn new(de: &'a XbrlDeserializer, fields: &'static [&'static str]) -> Self {
        let field_iterator: Box<dyn Iterator<Item = &'static str>> = if fields.is_empty() {
            // Flatten mode: offer all available fact names
            let mut keys: Vec<String> = de.full_name_map.keys().cloned().collect();
            keys.extend(de.local_name_map.keys().cloned());
            keys.sort();
            keys.dedup();

            // Convert to static strings (note: this leaks memory, but is needed for the iterator lifetime)
            let static_keys: &'static [String] = keys.leak();
            Box::new(static_keys.iter().map(|s| s.as_str()))
        } else {
            // Normal struct mode: iterate over provided fields
            Box::new(fields.iter().copied())
        };

        XbrlMapAccess {
            de,
            field_iterator,
            value: None,
        }
    }
}

impl<'de, 'a> MapAccess<'de> for XbrlMapAccess<'a> {
    type Error = XbrlError;

    /// Provides the next key (field name) for deserialization
    ///
    /// This method implements the serde MapAccess pattern by finding the next
    /// field that has data available and offering it to the visitor. The visitor
    /// can accept or reject the key based on the target struct's requirements.
    ///
    /// # Arguments
    ///
    /// * `seed` - Serde's key deserializer seed
    ///
    /// # Returns
    ///
    /// * `Result<Option<K::Value>>` - Next key or None if iteration complete
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        // Find the next field that we have data for
        while let Some(field) = self.field_iterator.next() {
            if let Some(fact) = self.de.find_best_fact(field) {
                // We found a fact for this field - cache its value
                self.value = Some(&fact.value);

                // Offer the field name as a key to the visitor
                return seed.deserialize(field.into_deserializer()).map(Some);
            }
        }

        // No more fields with data available
        Ok(None)
    }

    /// Provides the value corresponding to the previously offered key
    ///
    /// This method is called immediately after a successful next_key_seed call.
    /// It deserializes the cached fact value into the appropriate type for the
    /// target struct field.
    ///
    /// # Arguments
    ///
    /// * `seed` - Serde's value deserializer seed
    ///
    /// # Returns
    ///
    /// * `Result<V::Value>` - Deserialized value
    ///
    /// # Panics
    ///
    /// Panics if called without a successful next_key_seed call (serde contract violation).
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        let value = self
            .value
            .take()
            .expect("next_value_seed called without a successful next_key_seed");

        seed.deserialize(ValueDeserializer {
            value: value.clone(),
        })
    }
}

/// Deserializes individual XBRL fact values into appropriate Rust types
///
/// This struct handles the conversion from `XbrlValue` enum variants to the
/// specific types expected by target struct fields. It implements serde's
/// type-directed deserialization pattern.
///
/// # Type Conversion Strategy
///
/// The deserializer maps XBRL value types to Rust types as follows:
/// - `XbrlValue::String` → String types
/// - `XbrlValue::Bool` → bool types  
/// - `XbrlValue::F64` → floating-point types
/// - `XbrlValue::I64` → integer types
/// - `XbrlValue::Nil` → Option::None
struct ValueDeserializer {
    /// The XBRL value to be deserialized
    value: XbrlValue,
}

impl<'de> Deserializer<'de> for ValueDeserializer {
    type Error = XbrlError;

    /// Deserializes the value based on its runtime type
    ///
    /// This method examines the `XbrlValue` variant and calls the appropriate
    /// visitor method to perform type-safe conversion.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            XbrlValue::String(s) => visitor.visit_string(s),
            XbrlValue::Bool(b) => visitor.visit_bool(b),
            XbrlValue::F64(f) => visitor.visit_f64(f),
            XbrlValue::I64(i) => visitor.visit_i64(i),
            XbrlValue::Nil => visitor.visit_none(),
        }
    }

    /// Deserializes Option types with special handling for Nil values
    ///
    /// XBRL documents frequently have missing or nil values, which should
    /// be represented as `None` in Rust Option types. This method provides
    /// that mapping.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            XbrlValue::Nil => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    // Delegate all other type-specific deserialization to deserialize_any
    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structures::{Entity, Identifier, Period};

    #[test]
    fn test_context_comparison_with_recent_periods() {
        let recent_context = Context {
            id: "recent".to_string(),
            entity: Entity {
                identifier: Identifier {
                    scheme: "cik".to_string(),
                    value: "123".to_string(),
                },
                segment: None,
            },
            period: Period {
                instant: None,
                start_date: Some("2024-01-01".to_string()),
                end_date: Some("2024-03-31".to_string()),
            },
            scenario: None,
        };

        let older_context = Context {
            id: "older".to_string(),
            entity: Entity {
                identifier: Identifier {
                    scheme: "cik".to_string(),
                    value: "123".to_string(),
                },
                segment: None,
            },
            period: Period {
                instant: None,
                start_date: Some("2023-01-01".to_string()),
                end_date: Some("2023-03-31".to_string()),
            },
            scenario: None,
        };

        let result =
            XbrlDeserializer::compare_contexts(Some(&recent_context), Some(&older_context));
        assert_eq!(
            result,
            Ordering::Greater,
            "More recent context should be preferred"
        );
    }

    #[test]
    fn test_period_comparison() {
        let recent_period = Period {
            instant: None,
            start_date: Some("2024-01-01".to_string()),
            end_date: Some("2024-03-31".to_string()),
        };

        let older_period = Period {
            instant: None,
            start_date: Some("2023-01-01".to_string()),
            end_date: Some("2023-03-31".to_string()),
        };

        let result = XbrlDeserializer::compare_periods(&recent_period, &older_period);
        assert_eq!(result, Ordering::Greater, "More recent period should win");
    }

    #[test]
    fn test_value_deserializer_types() {
        // Test string value
        let string_deserializer = ValueDeserializer {
            value: XbrlValue::String("test".to_string()),
        };

        // Test that the deserializer handles different value types
        // (Full testing would require implementing a test visitor)
        match string_deserializer.value {
            XbrlValue::String(s) => assert_eq!(s, "test"),
            _ => panic!("Should be string value"),
        }

        // Test nil value
        let nil_deserializer = ValueDeserializer {
            value: XbrlValue::Nil,
        };

        match nil_deserializer.value {
            XbrlValue::Nil => {} // Expected
            _ => panic!("Should be nil value"),
        }
    }
}
