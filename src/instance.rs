//! # The parsed document
//!
//! What the [parser](crate::parser) produces: every context, unit and fact a
//! filing tags, in document order, with nothing interpreted yet. The types
//! mirror the XBRL elements they are read from.
//!
//! ```text
//!   Instance
//!     ├── contexts: Vec<Context>     who, when, and for which dimension members
//!     │     ├── entity ── identifier, segment (dimension members)
//!     │     ├── period ── instant | startDate..endDate
//!     │     └── scenario (dimension members)
//!     ├── units:    Vec<Unit>        iso4217:USD, xbrli:shares, USD per share
//!     └── facts:    Vec<RawFact>     concept, contextRef, unitRef, value
//! ```
//!
//! Most callers never touch these directly: [`Document`](crate::Document)
//! indexes an [`Instance`] and reads typed structs from it. They are public
//! for the cases a struct does not cover — walking every fact, say.

use serde::Deserialize;
use serde::de::{self, IgnoredAny, MapAccess, Visitor};
use std::fmt;

/// A parsed filing: the contents of an `<xbrl>` instance, or of the inline
/// XBRL tags of an HTML document.
#[derive(Debug, Default, Clone)]
pub struct Instance {
    /// The contexts facts refer to by `contextRef`.
    pub contexts: Vec<Context>,

    /// The units numeric facts refer to by `unitRef`.
    pub units: Vec<Unit>,

    /// Every fact, in document order.
    pub facts: Vec<RawFact>,
}

/// When, and for which slice of the entity, a fact is reported.
///
/// ```xml
/// <context id="c-12">
///   <entity>
///     <identifier scheme="http://www.sec.gov/CIK">0000320193</identifier>
///     <segment>
///       <xbrldi:explicitMember dimension="us-gaap:StatementClassOfStockAxis">us-gaap:CommonStockMember</xbrldi:explicitMember>
///     </segment>
///   </entity>
///   <period><instant>2025-09-27</instant></period>
/// </context>
/// ```
#[derive(Deserialize, Debug, Clone)]
pub struct Context {
    /// The id facts refer to.
    #[serde(rename = "@id")]
    pub id: String,

    /// The reporting entity, and the dimension members narrowing it.
    pub entity: Entity,

    /// The date or span of time the context covers.
    pub period: Period,

    /// Dimension members given in a `<scenario>` rather than a `<segment>`.
    /// SEC filings use the segment; other jurisdictions use the scenario.
    pub scenario: Option<Scenario>,
}

impl Context {
    /// The explicit dimension members narrowing this context, from its
    /// segment and its scenario.
    pub fn explicit_members(&self) -> impl Iterator<Item = &ExplicitMember> {
        let segment = self.entity.segment.iter().flat_map(|s| &s.explicit_members);
        let scenario = self.scenario.iter().flat_map(|s| &s.explicit_members);
        segment.chain(scenario)
    }

    /// The typed dimension members narrowing this context, from its segment
    /// and its scenario.
    pub fn typed_members(&self) -> impl Iterator<Item = &TypedMember> {
        let segment = self.entity.segment.iter().flat_map(|s| &s.typed_members);
        let scenario = self.scenario.iter().flat_map(|s| &s.typed_members);
        segment.chain(scenario)
    }

    /// Whether the context is for the entity as a whole: no dimension member
    /// narrows it. These are the figures on the face of a financial statement.
    pub fn is_consolidated(&self) -> bool {
        self.explicit_members().next().is_none() && self.typed_members().next().is_none()
    }
}

/// The reporting entity of a context.
#[derive(Deserialize, Debug, Clone)]
pub struct Entity {
    /// Who is reporting. For an SEC filing, the CIK.
    pub identifier: Identifier,

    /// The dimension members the context is narrowed to, if any.
    pub segment: Option<Segment>,
}

/// An entity identifier and the scheme it belongs to.
#[derive(Deserialize, Debug, Clone)]
pub struct Identifier {
    /// The scheme, e.g. `http://www.sec.gov/CIK`.
    #[serde(rename = "@scheme")]
    pub scheme: String,

    /// The identifier, e.g. `0000320193`.
    #[serde(rename = "$text")]
    pub value: String,
}

/// The time a context covers: either `instant`, or `start_date` and
/// `end_date`. Dates are as written, normally `YYYY-MM-DD`.
#[derive(Deserialize, Debug, Clone)]
pub struct Period {
    /// A point in time, as a balance sheet is reported.
    pub instant: Option<String>,

    /// First day of a duration, as an income statement is reported.
    #[serde(rename = "startDate")]
    pub start_date: Option<String>,

    /// Last day of a duration.
    #[serde(rename = "endDate")]
    pub end_date: Option<String>,
}

/// The dimension members in an entity's `<segment>`.
#[derive(Deserialize, Debug, Clone, Default)]
pub struct Segment {
    /// Members of explicit dimensions: an axis and one of its listed members.
    #[serde(rename = "explicitMember", default)]
    pub explicit_members: Vec<ExplicitMember>,

    /// Members of typed dimensions: an axis and a free-form value.
    #[serde(rename = "typedMember", default)]
    pub typed_members: Vec<TypedMember>,
}

/// The dimension members in a context's `<scenario>`.
#[derive(Deserialize, Debug, Clone, Default)]
pub struct Scenario {
    /// Members of explicit dimensions: an axis and one of its listed members.
    #[serde(rename = "explicitMember", default)]
    pub explicit_members: Vec<ExplicitMember>,

    /// Members of typed dimensions: an axis and a free-form value.
    #[serde(rename = "typedMember", default)]
    pub typed_members: Vec<TypedMember>,
}

/// One member of an explicit dimension.
#[derive(Deserialize, Debug, Clone)]
pub struct ExplicitMember {
    /// The axis, e.g. `us-gaap:StatementClassOfStockAxis`.
    #[serde(rename = "@dimension")]
    pub dimension: String,

    /// The member, e.g. `us-gaap:CommonClassAMember`.
    #[serde(rename = "$text")]
    pub value: String,
}

/// One member of a typed dimension: the axis, and a value that is not drawn
/// from a list.
///
/// ```xml
/// <xbrldi:typedMember dimension="us-gaap:RevenueRemainingPerformanceObligationExpectedTimingOfSatisfactionStartDateAxis">
///   <us-gaap:RevenueRemainingPerformanceObligationExpectedTimingOfSatisfactionStartDateAxis.domain>2025-09-28</us-gaap:RevenueRemainingPerformanceObligationExpectedTimingOfSatisfactionStartDateAxis.domain>
/// </xbrldi:typedMember>
/// ```
#[derive(Debug, Clone)]
pub struct TypedMember {
    /// The axis.
    pub dimension: String,

    /// The text of the element inside the member. Empty when that element is
    /// nil or holds markup rather than text.
    pub value: String,
}

impl<'de> Deserialize<'de> for TypedMember {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        /// The text of whatever element the member wraps.
        #[derive(Deserialize)]
        struct Text {
            #[serde(rename = "$text", default)]
            text: String,
        }

        struct TypedMemberVisitor;

        impl<'de> Visitor<'de> for TypedMemberVisitor {
            type Value = TypedMember;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a typedMember element")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut dimension = None;
                let mut value = String::new();
                while let Some(key) = map.next_key::<String>()? {
                    if key == "@dimension" {
                        dimension = Some(map.next_value()?);
                    } else if key.starts_with('@') {
                        map.next_value::<IgnoredAny>()?;
                    } else if let Ok(text) = map.next_value::<Text>() {
                        value = text.text;
                    }
                }
                Ok(TypedMember {
                    dimension: dimension.ok_or_else(|| de::Error::missing_field("dimension"))?,
                    value,
                })
            }
        }

        deserializer.deserialize_map(TypedMemberVisitor)
    }
}

/// A unit of measure: a single `measure`, or the ratio of two in `divide`.
///
/// ```xml
/// <unit id="usd"><measure>iso4217:USD</measure></unit>
/// <unit id="usdPerShare">
///   <divide>
///     <unitNumerator><measure>iso4217:USD</measure></unitNumerator>
///     <unitDenominator><measure>xbrli:shares</measure></unitDenominator>
///   </divide>
/// </unit>
/// ```
#[derive(Deserialize, Debug, Clone)]
pub struct Unit {
    /// The id facts refer to.
    #[serde(rename = "@id")]
    pub id: String,

    /// The measure of a simple unit, e.g. `iso4217:USD`.
    pub measure: Option<String>,

    /// The two measures of a ratio.
    pub divide: Option<Divide>,
}

/// A ratio of two measures, such as dollars per share.
#[derive(Debug, Clone)]
pub struct Divide {
    /// What is being measured.
    pub unit_numerator: UnitMeasure,

    /// Per what.
    pub unit_denominator: UnitMeasure,
}

// Filings spell these `unitNumerator` in XML and, lowercased by whatever
// produced the HTML, `unitnumerator` inline. Accept both.
impl<'de> Deserialize<'de> for Divide {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DivideVisitor;

        impl<'de> Visitor<'de> for DivideVisitor {
            type Value = Divide;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a divide element with a unitNumerator and a unitDenominator")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut numerator: Option<UnitMeasure> = None;
                let mut denominator: Option<UnitMeasure> = None;

                while let Some(key) = map.next_key::<String>()? {
                    if key.eq_ignore_ascii_case("unitnumerator") {
                        numerator = Some(map.next_value()?);
                    } else if key.eq_ignore_ascii_case("unitdenominator") {
                        denominator = Some(map.next_value()?);
                    } else {
                        map.next_value::<IgnoredAny>()?;
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

/// One side of a [`Divide`].
#[derive(Deserialize, Debug, Clone)]
pub struct UnitMeasure {
    /// The measure, e.g. `iso4217:USD` or `xbrli:shares`.
    pub measure: String,
}

/// The value of a fact, before it is converted to a field's type.
///
/// Values are kept as text: the same concept can be a number in one filing
/// and `N/A` in the next, and only the struct being read knows which type it
/// wants. For an inline fact the text is already normalised — the `format`
/// transformation, `scale` and `sign` have been applied, so `(1,234)` in
/// thousands is `"-1234000"`.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum XbrlValue {
    /// The fact's text.
    String(String),

    /// The fact is reported with no value: `xsi:nil="true"`, or an empty element.
    #[default]
    Nil,
}

impl XbrlValue {
    /// The text of the value. `None` for a nil fact.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            XbrlValue::String(s) => Some(s),
            XbrlValue::Nil => None,
        }
    }

    /// Whether the fact was reported with no value.
    pub fn is_nil(&self) -> bool {
        matches!(self, XbrlValue::Nil)
    }
}

/// One fact as the parser read it: a concept, the references that say what it
/// is a value *of*, and its value as text.
///
/// [`Fact<T>`](crate::Fact) is the same fact with its references resolved and
/// its value converted.
#[derive(Default, Debug, Clone)]
pub struct RawFact {
    /// The concept with its namespace prefix, e.g. `us-gaap:Assets`.
    pub full_name: String,

    /// The concept without the prefix, e.g. `Assets`.
    pub local_name: String,

    /// The id of the [`Context`] the fact is reported in.
    pub context_ref: Option<String>,

    /// The id of the [`Unit`] a numeric fact is measured in.
    pub unit_ref: Option<String>,

    /// How exact a numeric value is, as written: `"0"`, `"-3"` for thousands,
    /// `"INF"` for exact.
    pub decimals: Option<String>,

    /// The inline XBRL transformation the value was written in, e.g.
    /// `ixt:num-dot-decimal`. Already applied to `value`. `None` for a fact
    /// from an XML instance.
    pub format: Option<String>,

    /// The fact's own id, if the filing gives it one.
    pub id: Option<String>,

    /// The value.
    pub value: XbrlValue,
}

#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::de::from_str;

    #[test]
    fn a_nil_value_is_the_default() {
        assert_eq!(RawFact::default().value, XbrlValue::Nil);
        assert!(XbrlValue::Nil.is_nil());
        assert_eq!(XbrlValue::String("1".into()).as_str(), Some("1"));
    }

    #[test]
    fn a_context_reads_explicit_and_typed_members() {
        let context: Context = from_str(
            r#"<xbrli:context id="c-1">
                 <xbrli:entity>
                   <xbrli:identifier scheme="http://www.sec.gov/CIK">0000320193</xbrli:identifier>
                   <xbrli:segment>
                     <xbrldi:explicitMember dimension="us-gaap:StatementClassOfStockAxis">us-gaap:CommonStockMember</xbrldi:explicitMember>
                     <xbrldi:typedMember dimension="us-gaap:StartDateAxis"><us-gaap:StartDateAxis.domain>2025-09-28</us-gaap:StartDateAxis.domain></xbrldi:typedMember>
                   </xbrli:segment>
                 </xbrli:entity>
                 <xbrli:period><xbrli:instant>2025-09-27</xbrli:instant></xbrli:period>
               </xbrli:context>"#,
        )
        .unwrap();

        assert_eq!(context.id, "c-1");
        assert_eq!(context.entity.identifier.value, "0000320193");
        assert_eq!(context.period.instant.as_deref(), Some("2025-09-27"));
        assert!(!context.is_consolidated());

        let explicit: Vec<_> = context.explicit_members().collect();
        assert_eq!(explicit[0].value, "us-gaap:CommonStockMember");
        let typed: Vec<_> = context.typed_members().collect();
        assert_eq!(typed[0].dimension, "us-gaap:StartDateAxis");
        assert_eq!(typed[0].value, "2025-09-28");
    }

    #[test]
    fn a_context_with_only_a_typed_member_is_not_consolidated() {
        let context: Context = from_str(
            r#"<context id="c-2">
                 <entity>
                   <identifier scheme="http://www.sec.gov/CIK">1</identifier>
                   <segment><typedMember dimension="x:Axis"><x:Domain xsi:nil="true"/></typedMember></segment>
                 </entity>
                 <period><startDate>2025-01-01</startDate><endDate>2025-12-31</endDate></period>
               </context>"#,
        )
        .unwrap();
        assert!(!context.is_consolidated());
        assert_eq!(context.typed_members().next().unwrap().value, "");
    }

    #[test]
    fn a_ratio_unit_reads_in_either_spelling() {
        for (numerator, denominator) in [
            ("unitNumerator", "unitDenominator"),
            ("unitnumerator", "unitdenominator"),
        ] {
            let unit: Unit = from_str(&format!(
                "<unit id=\"usdPerShare\"><divide>\
                   <{numerator}><measure>iso4217:USD</measure></{numerator}>\
                   <{denominator}><measure>xbrli:shares</measure></{denominator}>\
                 </divide></unit>"
            ))
            .unwrap();
            let divide = unit.divide.unwrap();
            assert_eq!(divide.unit_numerator.measure, "iso4217:USD");
            assert_eq!(divide.unit_denominator.measure, "xbrli:shares");
        }
    }
}
