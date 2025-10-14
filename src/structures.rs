//! # Core XBRL Document Structures
//!
//! Provides the foundational data structures for representing parsed XBRL documents
//! in memory. These structures correspond directly to XBRL XML elements and serve
//! as the intermediate representation between raw XML and typed financial data.
//!
//! ## Structure Hierarchy
//!
//! ```text
//!                            ┌─────────────────┐
//!                            │      Xbrl       │ <- Root document
//!                            │   (top-level)   │
//!                            └─────────────────┘
//!                                     │
//!                    ┌────────────────┼────────────────┐
//!                    ▼                ▼                ▼
//!            ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
//!            │  Context    │  │    Unit     │  │    Fact     │
//!            │ (temporal & │  │ (measure)   │  │ (values)    │
//!            │dimensional) │  └─────────────┘  └─────────────┘
//!            └─────────────┘
//!                   │
//!        ┌──────────┼──────────┐
//!        ▼          ▼          ▼
//!   ┌─────────┐ ┌─────────┐ ┌─────────┐
//!   │ Entity  │ │ Period  │ │Scenario │
//!   │         │ │         │ │         │
//!   └─────────┘ └─────────┘ └─────────┘
//! ```

use serde::Deserialize;

/// Represents the entire XBRL document, corresponding to the root `<xbrl>` tag
///
/// This is the top-level container for all XBRL data, containing the three
/// primary components: contexts (temporal and dimensional information),
/// units (measurement definitions), and facts (actual data values).
///
/// # Example XBRL Structure
///
/// ```xml
/// <xbrl xmlns="http://www.xbrl.org/2003/instance">
///   <context id="c1">...</context>
///   <unit id="usd">...</unit>
///   <us-gaap:Assets contextRef="c1" unitRef="usd">1000000</us-gaap:Assets>
/// </xbrl>
/// ```
#[derive(Deserialize, Debug, Default, Clone)]
pub struct Xbrl {
    /// Context definitions that provide temporal and dimensional information for facts
    pub contexts: Vec<Context>,

    /// Unit definitions that specify measurement scales for numeric facts
    pub units: Vec<Unit>,

    /// The actual data facts reported in the XBRL document
    pub facts: Vec<Fact>,
}

/// A generic struct to capture metadata reference elements
///
/// XBRL documents often contain various `*Ref` elements (like `linkbaseRef`,
/// `schemaRef`, etc.) that provide metadata. This struct serves as a placeholder
/// to consume these elements during deserialization without causing errors,
/// while not requiring us to process their content.
#[derive(Deserialize, Debug, Clone, Default)]
pub struct MetadataRef {
    // Empty struct is sufficient to consume the tag and its attributes
    // This prevents deserialization errors while ignoring metadata we don't need
}

/// Represents an XBRL context defining the circumstances of a fact
///
/// Contexts provide essential metadata about when, for what entity, and under
/// what conditions a fact applies. They are referenced by facts through the
/// `contextRef` attribute and are crucial for proper fact interpretation.
///
/// # Context Selection Logic
///
/// When multiple facts exist for the same concept, contexts are used to
/// select the most appropriate value based on:
/// - **Period**: Most recent reporting period preferred
/// - **Entity**: Consolidated entity preferred over segments
/// - **Scenario**: Actual values preferred over budgets/forecasts
#[derive(Deserialize, Debug, Clone)]
pub struct Context {
    /// Unique identifier for this context, referenced by facts
    #[serde(rename = "@id")]
    pub id: String,

    /// Entity information (company, segments)
    pub entity: Entity,

    /// Time period information (instant, duration)
    pub period: Period,

    /// Optional scenario information (actual vs budget, etc.)
    pub scenario: Option<Scenario>,
}

/// Represents the reporting entity and optional dimensional segments
///
/// The entity identifies which company or organization the facts relate to,
/// and can include dimensional breakdowns (segments) for detailed reporting.
#[derive(Deserialize, Debug, Clone)]
pub struct Entity {
    /// The entity identifier (typically CIK for SEC filings)
    pub identifier: Identifier,

    /// Optional dimensional segments for detailed breakdowns
    pub segment: Option<Segment>,
}

/// Entity identifier with scheme information
///
/// Provides the actual entity identifier along with the scheme that
/// defines the identifier format (e.g., CIK scheme for SEC filings).
#[derive(Deserialize, Debug, Clone)]
pub struct Identifier {
    /// The identification scheme (e.g., "http://www.sec.gov/CIK")
    #[serde(rename = "@scheme")]
    pub scheme: String,

    /// The actual identifier value (e.g., "0001234567")
    #[serde(rename = "$text")]
    pub value: String,
}

/// Represents the time period for a context
///
/// XBRL supports two types of periods:
/// - **Instant**: A specific point in time (e.g., balance sheet date)
/// - **Duration**: A span of time (e.g., quarter or year for income statement)
///
/// # Period Comparison
///
/// For fact selection, periods are compared with preference given to:
/// 1. Most recent end dates
/// 2. Most recent instant dates
/// 3. Longer durations (annual over quarterly)
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Period {
    /// Point-in-time date for instant periods (YYYY-MM-DD format)
    #[serde(rename = "instant")]
    pub instant: Option<String>,

    /// Start date for duration periods (YYYY-MM-DD format)
    #[serde(rename = "startDate")]
    pub start_date: Option<String>,

    /// End date for duration periods (YYYY-MM-DD format)
    #[serde(rename = "endDate")]
    pub end_date: Option<String>,
}

/// Represents dimensional information in a segment
///
/// Segments provide additional dimensional breakdowns beyond the basic
/// entity-period combination, such as business segments, geographical
/// regions, or product lines.
#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Segment {
    /// List of explicit dimensional members
    #[serde(rename = "explicitMember", default)]
    pub explicit_members: Vec<ExplicitMember>,
}

/// Represents dimensional information in a scenario
///
/// Scenarios provide additional contextual information such as whether
/// the data represents actual results, budgets, forecasts, or pro forma
/// adjustments.
#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Scenario {
    /// List of explicit dimensional members
    #[serde(rename = "explicitMember", default)]
    pub explicit_members: Vec<ExplicitMember>,
}

/// Represents a single dimensional member
///
/// Explicit members define specific values for dimensional axes,
/// allowing for detailed breakdowns of financial data.
#[derive(Deserialize, Debug, Clone)]
pub struct ExplicitMember {
    /// The dimension this member belongs to
    #[serde(rename = "@dimension")]
    pub dimension: String,

    /// The specific member value
    #[serde(rename = "$text")]
    pub value: String,
}

/// Represents a unit of measure for numeric facts
///
/// Units define how numeric values should be interpreted, such as
/// currency (USD), shares, or ratios. They can be simple measures
/// or complex ratios.
///
/// # Unit Types
///
/// - **Simple**: Single measure (e.g., "usd" for US Dollars)
/// - **Ratio**: Division of measures (e.g., "usd/shares" for price per share)
#[derive(Deserialize, Debug, Clone)]
pub struct Unit {
    /// Unique identifier for this unit definition
    #[serde(rename = "@id")]
    pub id: String,

    /// Simple measure for basic units (e.g., "usd", "shares")
    pub measure: Option<String>,

    /// Division specification for ratio units
    pub divide: Option<Divide>,
}

/// Represents a division operation for ratio units
///
/// Used to create complex units that are ratios of two measures,
/// such as earnings per share (usd/shares) or price-to-earnings ratios.
///
/// Supports both camelCase (unitNumerator) and lowercase (unitnumerator) XML tags.
#[derive(Debug, Clone)]
pub struct Divide {
    /// The numerator unit measure
    pub unit_numerator: UnitMeasure,

    /// The denominator unit measure
    pub unit_denominator: UnitMeasure,
}

// Custom Deserialize implementation to handle both camelCase and lowercase tags
impl<'de> Deserialize<'de> for Divide {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct DivideVisitor;

        impl<'de> Visitor<'de> for DivideVisitor {
            type Value = Divide;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a Divide element with unitnumerator/unitdenominator or unitNumerator/unitDenominator")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut numerator: Option<UnitMeasure> = None;
                let mut denominator: Option<UnitMeasure> = None;

                while let Some(key) = map.next_key::<String>()? {
                    let key_lower = key.to_lowercase();
                    if key_lower == "unitnumerator" {
                        numerator = Some(map.next_value()?);
                    } else if key_lower == "unitdenominator" {
                        denominator = Some(map.next_value()?);
                    } else {
                        // Skip unknown fields
                        let _: serde::de::IgnoredAny = map.next_value()?;
                    }
                }

                Ok(Divide {
                    unit_numerator: numerator
                        .ok_or_else(|| de::Error::missing_field("unitNumerator"))?,
                    unit_denominator: denominator
                        .ok_or_else(|| de::Error::missing_field("unitDenominator"))?,
                })
            }
        }

        deserializer.deserialize_map(DivideVisitor)
    }
}

/// Represents a single measure within a unit definition
#[derive(Deserialize, Debug, Clone)]
pub struct UnitMeasure {
    /// The measure identifier (e.g., "iso4217:USD", "xbrli:shares")
    pub measure: String,
}

/// Typed enumeration for XBRL fact values
///
/// This enum provides a simple, untyped representation for fact values. The parser
/// extracts the raw string content, and the `serde` deserializer is responsible for
/// converting this raw string into the specific type requested by the target struct.
///
/// # Nil Values
///
/// XBRL supports explicit nil values (`xsi:nil="true"`) or empty tags, which are
/// represented as `XbrlValue::Nil` to distinguish from non-empty strings.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum XbrlValue {
    /// Text content, preserved exactly as found in the XML.
    String(String),

    /// Explicit nil values (xsi:nil="true" or empty tags).
    #[default]
    Nil,
}

/// Represents a single XBRL fact (data point)
///
/// Facts are the core data elements in XBRL documents, representing actual
/// reported values for specific concepts (e.g., total assets, revenue).
/// Each fact is associated with a context and may have a unit.
///
/// # Fact Processing
///
/// Facts undergo several processing steps:
/// 1. **Extraction**: Parsed from XML with attributes
/// 2. **Type Inference**: Value converted to appropriate Rust type
/// 3. **Context Resolution**: Associated with temporal/dimensional context
/// 4. **Selection**: Best fact chosen when multiple exist for same concept
#[derive(Default, Deserialize, Debug, Clone)]
pub struct Fact {
    /// The full, namespaced concept name (e.g., "us-gaap:Assets")
    pub full_name: String,

    /// The local concept name without namespace (e.g., "Assets")
    pub local_name: String,

    /// Reference to the context defining when/where this fact applies
    pub context_ref: Option<String>,

    /// Reference to the unit defining how to interpret numeric values
    pub unit_ref: Option<String>,

    /// Decimal precision indicator for numeric values
    pub decimals: Option<String>,

    /// The transformation rule applied to this fact (iXBRL only).
    pub format: Option<String>,

    /// Unique identifier for this fact instance.
    pub id: Option<String>,

    /// The typed value of this fact
    #[serde(skip_deserializing)] // Populated manually during parsing
    pub value: XbrlValue,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xbrl_value_types() {
        let string_val = XbrlValue::String("Test".to_string());
        let nil_val = XbrlValue::Nil;

        // Test that variants can be created and are distinct
        assert_ne!(string_val, nil_val);
        assert_eq!(nil_val, XbrlValue::default());
    }

    #[test]
    fn test_fact_defaults() {
        let fact = Fact::default();
        assert_eq!(fact.full_name, "");
        assert_eq!(fact.local_name, "");
        assert_eq!(fact.value, XbrlValue::Nil);
    }
}
