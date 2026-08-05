//! # XBRL Serde Deserializer
//!
//! This module provides a high-performance, custom serde deserializer for XBRL documents.
//! It bridges the gap between raw XBRL parsing and structured data extraction by implementing
//! intelligent fact selection and context-aware deserialization.
//!
//! ## Architecture Overview
//!
//! The architecture is decoupled into a data container and a deserializer:
//!
//! 1.  **`XbrlDataContext`**: A read-only "database" created once per document. It holds
//!     all parsed facts, contexts, and pre-built indexes for fast lookups.
//! 2.  **`XbrlDeserializer`**: A lightweight, short-lived state machine that implements
//!     `serde::Deserializer`. It holds a reference to the `XbrlDataContext` and processes
//!     a specific slice of facts (a "scope"), enabling nested struct deserialization.
//!
//! ```text
//!   Raw XBRL Document
//!           │
//!           ▼
//!   ┌─────────────────┐
//!   │  XBRL Parser    │
//!   │  (parser.rs)    │
//!   └─────────────────┘
//!           │
//!           ▼
//!   ┌─────────────────┐
//!   │ XbrlDataContext │ <- Holds all data & indexes. Created ONCE.
//!   └─────────────────┘
//!           │
//!           ├─►┌──────────────────┐
//!           │  │ XbrlDeserializer │ <- Deserializes a scope of facts
//!           │  └──────────────────┘      into a struct (e.g., DeiInfo)
//!           │
//!           └─►┌──────────────────┐
//!              │ XbrlDeserializer │ <- Deserializes another scope
//!              └──────────────────┘      into another struct (e.g., Financials)
//! ```

use crate::error::{Result, XbrlError};
use crate::structures::{Context, Fact, Period, Xbrl, XbrlValue};
use common::date_utils::is_iso_date;
use serde::de::{self, Deserializer, IntoDeserializer, MapAccess, Visitor};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::vec::IntoIter;

/// Holds the complete, indexed XBRL data for efficient lookup.
/// This struct acts as a read-only database during deserialization.
pub struct XbrlDataContext {
    /// Original parsed XBRL data containing all facts, contexts, and units.
    pub xbrl: Xbrl,

    /// Index mapping full concept names to fact positions.
    full_name_map: HashMap<String, Vec<usize>>,

    /// Index mapping local concept names to fact positions.
    local_name_map: HashMap<String, Vec<usize>>,

    /// Context cache mapping context IDs to resolved context objects.
    contexts: HashMap<String, Context>,

    /// The period this filing actually reports on, used to anchor fact selection.
    reporting_period: Option<ReportingPeriod>,
}

/// The span of time a filing reports on, derived from the SEC "Required Context".
///
/// Every SEC XBRL filing tags `dei:DocumentPeriodEndDate` against the context that
/// covers the reporting period, so that context tells us both the closing date of
/// the report and — for duration contexts — which span is the primary one. In a Q3
/// 10-Q, for instance, the required context runs from the start of the fiscal year
/// to the quarter end, which distinguishes the year-to-date column from the
/// three-month column that shares the same end date.
#[derive(Debug, Clone)]
struct ReportingPeriod {
    /// Closing date of the reporting period (e.g. `"2025-03-31"`).
    end: String,

    /// Opening date, present when the required context is a duration.
    start: Option<String>,
}

/// How well a context's period lines up with the filing's reporting period.
///
/// Ordering is meaningful: greater variants are better matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum PeriodMatch {
    /// The period carries no usable date.
    Unknown,

    /// The period extends past the close of the report. These are cover-page
    /// "as of" dates and subsequent-event disclosures — usable as a last resort,
    /// but never the filing's reported figure when a period-aligned fact exists.
    AfterReport,

    /// The period closes before the report does: prior-year comparatives, and the
    /// preceding fiscal year end that appears alongside every balance sheet.
    BeforeReport,

    /// The period closes on the reporting date but spans a different range than
    /// the required context (e.g. the three-month column of a Q3 filing).
    EndsOnReportDate,

    /// Exactly the filing's required context.
    RequiredContext,
}

impl PeriodMatch {
    /// Classifies a period against the filing's reporting period.
    ///
    /// When the reporting period is unknown (a document with no
    /// `dei:DocumentPeriodEndDate`), everything dated collapses into a single
    /// class so that selection falls back to plain recency.
    fn of(period: &Period, report: Option<&ReportingPeriod>) -> Self {
        let date = significant_date(period);
        if date.is_empty() {
            return PeriodMatch::Unknown;
        }

        let Some(report) = report else {
            return PeriodMatch::BeforeReport;
        };

        match date.cmp(report.end.as_str()) {
            Ordering::Greater => PeriodMatch::AfterReport,
            Ordering::Less => PeriodMatch::BeforeReport,
            Ordering::Equal => match (&period.start_date, &report.start) {
                // An instant closing on the reporting date is the balance sheet
                // date — exactly what the filing reports.
                (None, _) => PeriodMatch::RequiredContext,
                (Some(start), Some(report_start)) if start == report_start => {
                    PeriodMatch::RequiredContext
                }
                // Nothing to discriminate on, so treat it as the primary span.
                (Some(_), None) => PeriodMatch::RequiredContext,
                (Some(_), Some(_)) => PeriodMatch::EndsOnReportDate,
            },
        }
    }
}

/// Extracts the date that best represents a period: its end for durations, its
/// instant for points in time.
fn significant_date(period: &Period) -> &str {
    period
        .end_date
        .as_deref()
        .or(period.instant.as_deref())
        .unwrap_or("")
}

/// Counts the dimensional members that narrow a context.
///
/// Zero means consolidated, entity-wide data — the figure that belongs on a
/// financial statement line.
fn dimension_depth(context: &Context) -> (bool, usize) {
    let members = context
        .entity
        .segment
        .as_ref()
        .map_or(0, |s| s.explicit_members.len())
        + context
            .scenario
            .as_ref()
            .map_or(0, |s| s.explicit_members.len());
    let has_dims = context.entity.segment.is_some() || context.scenario.is_some();
    (has_dims, members)
}

impl XbrlDataContext {
    /// Creates a new data context from parsed Xbrl data.
    /// This constructor performs the expensive initialization work of building
    /// indexes and caching contexts.
    pub fn new(xbrl: Xbrl) -> Self {
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

        let reporting_period = Self::resolve_reporting_period(&xbrl, &contexts);

        XbrlDataContext {
            xbrl,
            full_name_map,
            local_name_map,
            contexts,
            reporting_period,
        }
    }

    /// Determines the period this filing reports on.
    ///
    /// `dei:DocumentPeriodEndDate` is tagged against the required context, so that
    /// fact gives us both the closing date (its value) and the span (its context).
    /// Returns `None` for documents that omit it, in which case fact selection
    /// degrades to preferring the most recent period.
    fn resolve_reporting_period(
        xbrl: &Xbrl,
        contexts: &HashMap<String, Context>,
    ) -> Option<ReportingPeriod> {
        let fact = xbrl.facts.iter().find(|f| {
            f.full_name == "dei:DocumentPeriodEndDate" || f.local_name == "DocumentPeriodEndDate"
        })?;

        let context = fact
            .context_ref
            .as_ref()
            .and_then(|id| contexts.get(id.as_str()));

        // Prefer the tagged value; fall back to the context's own dates when the
        // value is missing or not a normalized ISO date.
        let end = match &fact.value {
            XbrlValue::String(s) if is_iso_date(s.trim()) => Some(s.trim().to_string()),
            _ => None,
        }
        .or_else(|| {
            context
                .map(|c| significant_date(&c.period))
                .filter(|d| is_iso_date(d))
                .map(str::to_string)
        })?;

        Some(ReportingPeriod {
            end,
            start: context.and_then(|c| c.period.start_date.clone()),
        })
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
    fn find_best_fact(&self, rename_attr: &str) -> Option<&Fact> {
        // Determine which index to use based on concept name format
        let fact_indices = if rename_attr.contains(':') {
            // Full namespace format (e.g., "us-gaap:Assets")
            self.full_name_map.get(rename_attr)
        } else {
            // Local name format (e.g., "Assets")
            self.local_name_map.get(rename_attr)
        };

        let indices = fact_indices?;

        // Get all candidate facts for this concept
        let candidates: Vec<_> = indices.iter().map(|&i| &self.xbrl.facts[i]).collect();

        // Apply selection algorithm to find the best fact
        candidates.into_iter().max_by(|a, b| {
            let context_a = a.context_ref.as_ref().and_then(|id| self.contexts.get(id));
            let context_b = b.context_ref.as_ref().and_then(|id| self.contexts.get(id));
            Self::compare_contexts(context_a, context_b, self.reporting_period.as_ref())
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
    /// 2. **Period alignment**: Periods matching the filing's reporting period beat
    ///    comparatives and post-period dates (see [`PeriodMatch`])
    /// 3. **Proximity**: Within a class, the period nearest the reporting date wins
    /// 4. **Dimensional Simplicity**: Consolidated data preferred over segments
    ///
    /// Alignment outranks dimensionality deliberately: a segment-tagged figure for
    /// the period under report is still a figure about this period, whereas a
    /// consolidated figure from the prior year is not.
    ///
    /// # Arguments
    ///
    /// * `a` - First context to compare (optional)
    /// * `b` - Second context to compare (optional)
    /// * `report` - The filing's reporting period, when it could be determined
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
    fn compare_contexts(
        a: Option<&Context>,
        b: Option<&Context>,
        report: Option<&ReportingPeriod>,
    ) -> Ordering {
        match (a, b) {
            (Some(ctx_a), Some(ctx_b)) => {
                let match_a = PeriodMatch::of(&ctx_a.period, report);
                let match_b = PeriodMatch::of(&ctx_b.period, report);

                match_a
                    .cmp(&match_b)
                    .then_with(|| Self::compare_proximity(&ctx_a.period, &ctx_b.period, match_a))
                    // Prefer contexts without dimensional breakdowns (consolidated data)
                    .then_with(|| {
                        dimension_depth(ctx_a)
                            .cmp(&dimension_depth(ctx_b))
                            .reverse()
                    })
            }
            (Some(_), None) => Ordering::Greater, // Context beats no context
            (None, Some(_)) => Ordering::Less,    // No context loses to context
            (None, None) => Ordering::Equal,      // Both missing contexts are equal
        }
    }

    /// Breaks ties between two periods in the same [`PeriodMatch`] class by
    /// preferring the one closer to the reporting date.
    ///
    /// For periods that close after the report, "closer" means *earlier*: given a
    /// cover-page date and a subsequent-event date, the cover page is the nearer
    /// approximation of the reporting date. Everywhere else the later date wins,
    /// which also serves as the fallback ordering when the filing's reporting
    /// period is unknown.
    ///
    /// # Date Format
    ///
    /// Expects ISO 8601 date format (YYYY-MM-DD) which allows lexicographic comparison.
    fn compare_proximity(a: &Period, b: &Period, class: PeriodMatch) -> Ordering {
        let date_a = significant_date(a);
        let date_b = significant_date(b);

        match class {
            PeriodMatch::AfterReport => date_b.cmp(date_a),
            _ => date_a.cmp(date_b),
        }
    }
}

/// Deserializes a target struct from an `XbrlDataContext`.
/// This is the new high-level entry point for deserializing a taxonomy.
pub fn from_data<T>(context: &XbrlDataContext) -> Result<T>
where
    T: de::DeserializeOwned,
{
    let mut deserializer = XbrlDeserializer { context };
    T::deserialize(&mut deserializer)
}

/// Core XBRL deserializer that implements serde's `Deserializer` trait.
/// This struct is now lightweight and holds references to the data context
/// and the current scope of facts to be processed.
pub struct XbrlDeserializer<'a> {
    /// A reference to the global, read-only data context.
    context: &'a XbrlDataContext,
}

impl<'de, 'a, 'b> de::Deserializer<'de> for &'a mut XbrlDeserializer<'b> {
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

    /// Deserializes a struct by providing field-aware map access.
    /// This is the primary entry point for deserializing structs like `Financials`
    /// and nested structs like `BalanceSheet`.
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

    /// Deserializes a map. We treat it like a struct, which is correct for `#[serde(flatten)]`.
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_struct("", &[], visitor)
    }

    /// Deserializes an `Option<T>`. This is called for fields like `Option<f64>`.
    /// It checks if a fact exists. If not, it returns `None`. If it does, it proceeds
    /// to deserialize the inner `T`.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // Important: returning None here prevents serde from attempting to deserialize
        // primitives (e.g., f64) on the main deserializer, which would trigger the
        // 'deserialize_f64 called on main deserializer' error.
        //
        // Actual presence of a value for a field is handled by XbrlMapAccess::next_value_seed,
        // which supplies a ValueDeserializer when a fact exists. For fields with no facts,
        // None is correct behavior.
        visitor.visit_none()
    }

    // For primitive types, we create a `ValueDeserializer`. This will only be called
    // for fields inside a struct, after `XbrlMapAccess` has found a fact.
    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // This function should not be called directly on the main deserializer.
        // It indicates a logic error. The `XbrlMapAccess` should always create
        // a `ValueDeserializer` for primitive values.
        Err(XbrlError::DeserializationError(
            "deserialize_f64 called on main deserializer".to_string(),
        ))
    }

    // By removing `forward_to_deserialize_any!`, we force serde to use the
    // specific methods above. We only need to forward types that are not
    // structs, maps, or primitives we handle specially.
    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct enum identifier ignored_any
    }
}

/// Provides map-like access to XBRL facts for serde deserialization
///
/// This struct implements serde's `MapAccess` trait, allowing the deserializer
/// to present XBRL facts as key-value pairs where keys are concept names and
/// values are the selected fact values.
struct XbrlMapAccess<'a, 'b> {
    /// Reference to the main deserializer for fact lookup
    de: &'a mut XbrlDeserializer<'b>,

    /// Iterator over field names to process
    field_iterator: IntoIter<String>,

    /// Current field name, cached for rich error reporting
    current_field: Option<String>,

    /// Current fact to be deserialized (set by next_key_seed)
    fact: Option<&'b Fact>,
}

impl<'a, 'b> XbrlMapAccess<'a, 'b> {
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
    fn new(de: &'a mut XbrlDeserializer<'b>, fields: &'static [&'static str]) -> Self {
        let field_iterator: IntoIter<String> = if fields.is_empty() {
            // Flatten mode: build an owned Vec of keys.
            let mut keys: Vec<String> = de.context.full_name_map.keys().cloned().collect();
            keys.extend(de.context.local_name_map.keys().cloned());
            keys.sort();
            keys.dedup();

            keys.into_iter()
        } else {
            // Structured mode: create an owned Vec from the static slice.
            fields
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .into_iter()
        };

        XbrlMapAccess {
            de,
            field_iterator,
            current_field: None,
            fact: None,
        }
    }
}

impl<'de, 'a, 'b> MapAccess<'de> for XbrlMapAccess<'a, 'b> {
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
        for field in self.field_iterator.by_ref() {
            // **THE FIX**: An empty field name is invalid and causes infinite recursion
            // in flatten mode. We must explicitly skip it.
            if field.is_empty() {
                continue;
            }

            // Heuristic: fields with ':' are XBRL concepts (primitives).
            // Fields without ':' are nested structs.
            let is_primitive_field = field.contains(':');

            if is_primitive_field {
                // This is a primitive field like `us-gaap:Assets`.
                // We MUST find a fact for it.
                if let Some(fact) = self.de.context.find_best_fact(&field) {
                    self.fact = Some(fact);
                    self.current_field = Some(field.clone());

                    // Offer the field name as a key to the visitor
                    return seed.deserialize(field.into_deserializer()).map(Some);
                } else {
                    // No fact found for this primitive field. Skip it.
                    // Serde will use `#[serde(default)]` to populate it with `None`.
                    continue; // Move to the next field.
                }
            } else {
                // This is a nested struct field like `balance_sheet`.
                // We don't look for a fact. We offer the key and let `next_value_seed` recurse.
                self.fact = None; // Signal to `next_value_seed` that this is a struct.
                self.current_field = Some(field.clone());
                return seed.deserialize(field.into_deserializer()).map(Some);
            }
        }

        // No more fields left in the iterator.
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
        if let Some(fact) = self.fact.take() {
            // A fact was found by next_key_seed. This must be a primitive.
            // Deserialize it using the specialized ValueDeserializer.
            seed.deserialize(ValueDeserializer { fact })
        } else {
            // No fact was found. This must be a nested struct.
            // Delegate back to the main deserializer to start a new `deserialize_struct` cycle.
            seed.deserialize(&mut *self.de)
        }
    }
}

/// Deserializes individual XBRL fact values into appropriate Rust types.
///
/// This struct is the final stage of deserialization, responsible for converting
/// the raw string value from an `XbrlValue` into the specific type requested by
/// a struct field (e.g., `f64`, `bool`, `String`). It implements `serde`'s
/// type-directed deserialization methods.
struct ValueDeserializer<'a> {
    /// The complete XBRL fact to be deserialized
    fact: &'a Fact,
}

impl<'de, 'a> Deserializer<'de> for ValueDeserializer<'a> {
    type Error = XbrlError;

    /// Deserializes the value based on its runtime type. This is a fallback
    /// when the specific target type is not known. It uses the `decimals`
    /// attribute to make an intelligent guess between int, float, and string.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match &self.fact.value {
            XbrlValue::String(s) => {
                // 1. Check for boolean
                let lower = s.to_lowercase();
                if lower == "true" || lower == "yes" {
                    return visitor.visit_bool(true);
                }
                if lower == "false" || lower == "no" {
                    return visitor.visit_bool(false);
                }

                // 2. Check for numeric types using the 'decimals' attribute
                if let Some(decimals) = &self.fact.decimals {
                    let clean_s = s.replace(',', "");
                    // Treat "INF" decimals as integersб which is common for
                    // share counts and other whole numbers.
                    if decimals == "0" || decimals.eq_ignore_ascii_case("INF") {
                        if let Ok(i) = clean_s.parse::<i64>() {
                            return visitor.visit_i64(i);
                        }
                    } else if let Ok(f) = clean_s.parse::<f64>() {
                        // Any other 'decimals' value implies a float
                        return visitor.visit_f64(f);
                    }
                }

                // 3. Fallback to string
                visitor.visit_string(s.clone())
            }
            XbrlValue::Nil => visitor.visit_none(),
        }
    }

    /// Deserializes a struct field that expects a string.
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match &self.fact.value {
            XbrlValue::String(s) => visitor.visit_string(s.clone()),
            XbrlValue::Nil => visitor.visit_none(), // Should be handled by deserialize_option
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match &self.fact.value {
            XbrlValue::String(s) => s
                .replace(',', "")
                .parse::<i64>()
                .map_err(|_| XbrlError::ValueConversion {
                    field_name: self.fact.full_name.clone(),
                    value: s.clone(),
                    target_type: "i64".to_string(),
                })
                .and_then(|i| visitor.visit_i64(i)),
            XbrlValue::Nil => visitor.visit_none(),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match &self.fact.value {
            XbrlValue::String(s) => s
                .replace(',', "")
                .parse::<f64>()
                .map_err(|_| XbrlError::ValueConversion {
                    field_name: self.fact.full_name.clone(),
                    value: s.clone(),
                    target_type: "f64".to_string(),
                })
                .and_then(|f| visitor.visit_f64(f)),
            XbrlValue::Nil => visitor.visit_none(),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match &self.fact.value {
            XbrlValue::String(s) => {
                let lower = s.to_lowercase();

                // Handle standard text values
                if lower == "true" || lower == "yes" {
                    visitor.visit_bool(true)
                } else if lower == "false" || lower == "no" {
                    visitor.visit_bool(false)
                } else {
                    Err(XbrlError::ValueConversion {
                        field_name: self.fact.full_name.clone(),
                        value: s.clone(),
                        target_type: "bool".to_string(),
                    })
                }
            }
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
        match self.fact.value {
            XbrlValue::Nil => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    /// Handles newtype structs, which is how `serde` often represents `Option<T>`
    /// when a specific type is known. We delegate to the inner type's deserializer.
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    // Forward all other type-specific deserialization to deserialize_any.
    serde::forward_to_deserialize_any! {
        <V: Visitor<'de>>
        i8 i16 i32 i128 u8 u16 u32 u64 u128 f32 char
        bytes byte_buf unit unit_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structures::{Entity, ExplicitMember, Identifier, Segment};

    fn create_test_context(id: &str, end_date: &str) -> Context {
        Context {
            id: id.to_string(),
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
                end_date: Some(end_date.to_string()),
            },
            scenario: None,
        }
    }

    fn instant_context(id: &str, instant: &str) -> Context {
        Context {
            id: id.to_string(),
            entity: Entity {
                identifier: Identifier {
                    scheme: "cik".to_string(),
                    value: "123".to_string(),
                },
                segment: None,
            },
            period: Period {
                instant: Some(instant.to_string()),
                start_date: None,
                end_date: None,
            },
            scenario: None,
        }
    }

    fn duration_context(id: &str, start: &str, end: &str) -> Context {
        let mut context = create_test_context(id, end);
        context.period.start_date = Some(start.to_string());
        context
    }

    fn quarterly_report(start: &str, end: &str) -> ReportingPeriod {
        ReportingPeriod {
            end: end.to_string(),
            start: Some(start.to_string()),
        }
    }

    #[test]
    fn test_context_comparison_with_recent_periods() {
        let recent_context = create_test_context("recent", "2024-03-31");
        let older_context = create_test_context("older", "2023-03-31");

        let result =
            XbrlDataContext::compare_contexts(Some(&recent_context), Some(&older_context), None);
        assert_eq!(
            result,
            Ordering::Greater,
            "More recent context should be preferred"
        );
    }

    #[test]
    fn test_balance_sheet_date_beats_prior_year_end() {
        // The exact shape that made 10-Q total assets report the prior year end:
        // two undimensioned instants, one on the reporting date, one before it.
        let report = quarterly_report("2025-01-01", "2025-03-31");
        let reporting_date = instant_context("c5", "2025-03-31");
        let prior_year_end = instant_context("c6", "2024-12-31");

        assert_eq!(
            XbrlDataContext::compare_contexts(
                Some(&reporting_date),
                Some(&prior_year_end),
                Some(&report)
            ),
            Ordering::Greater,
            "Balance sheet date should beat the prior year end comparative"
        );
    }

    #[test]
    fn test_cover_page_date_loses_to_reporting_date() {
        // Cover-page "as of" instants are later than the reporting date but are
        // not what the financial statements report.
        let report = quarterly_report("2025-01-01", "2025-03-31");
        let reporting_date = instant_context("c5", "2025-03-31");
        let cover_page = instant_context("c4", "2025-05-12");

        assert_eq!(
            XbrlDataContext::compare_contexts(
                Some(&reporting_date),
                Some(&cover_page),
                Some(&report)
            ),
            Ordering::Greater,
            "A post-period cover date should not outrank the reporting date"
        );
    }

    #[test]
    fn test_required_context_beats_shorter_span_with_same_end() {
        // Q3 filing: the year-to-date column is the required context, the
        // three-month column shares its end date.
        let report = quarterly_report("2024-01-01", "2024-09-30");
        let year_to_date = duration_context("ytd", "2024-01-01", "2024-09-30");
        let three_months = duration_context("q3", "2024-07-01", "2024-09-30");

        assert_eq!(
            XbrlDataContext::compare_contexts(
                Some(&year_to_date),
                Some(&three_months),
                Some(&report)
            ),
            Ordering::Greater,
            "The filing's required context should win over a same-end span"
        );
    }

    #[test]
    fn test_consolidated_beats_dimensional_within_same_period() {
        let report = quarterly_report("2025-01-01", "2025-03-31");
        let consolidated = instant_context("c5", "2025-03-31");
        let mut segmented = instant_context("c7", "2025-03-31");
        segmented.entity.segment = Some(Segment {
            explicit_members: vec![ExplicitMember {
                dimension: "us-gaap:StatementClassOfStockAxis".to_string(),
                value: "ClassAMember".to_string(),
            }],
        });

        assert_eq!(
            XbrlDataContext::compare_contexts(Some(&consolidated), Some(&segmented), Some(&report)),
            Ordering::Greater,
            "Consolidated data should win over a segment breakdown"
        );
    }

    #[test]
    fn test_period_match_classification() {
        let report = quarterly_report("2025-01-01", "2025-03-31");

        assert_eq!(
            PeriodMatch::of(&instant_context("a", "2025-03-31").period, Some(&report)),
            PeriodMatch::RequiredContext
        );
        assert_eq!(
            PeriodMatch::of(&instant_context("b", "2024-12-31").period, Some(&report)),
            PeriodMatch::BeforeReport
        );
        assert_eq!(
            PeriodMatch::of(&instant_context("c", "2025-05-12").period, Some(&report)),
            PeriodMatch::AfterReport
        );
        assert_eq!(
            PeriodMatch::of(
                &duration_context("d", "2025-02-01", "2025-03-31").period,
                Some(&report)
            ),
            PeriodMatch::EndsOnReportDate
        );
    }
}
