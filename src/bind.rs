//! # Binding facts to structs
//!
//! An XBRL document is a table of facts: one value per concept, per period,
//! per combination of dimension members. A struct is a view of that table —
//! "the balance sheet as of the reporting date", "the income statement for
//! each period the filing reports". This module is how a view is described and
//! filled.
//!
//! ```text
//!   Instance (contexts, units, facts)      <- parser: everything the filing tags
//!           │
//!           ▼
//!   Document                                <- indexed once per filing
//!           │
//!           │  extract::<T>()               <- T: FromXbrl, derived
//!           ▼
//!   Scope ── which period a struct is being read for
//!           │
//!           ├─ #[xbrl(concept = "..")]  field: Option<f64>        one value
//!           ├─ #[xbrl(concept = "..")]  field: Option<Fact<f64>>  one value, with its context
//!           ├─ #[xbrl(concept = "..")]  field: Vec<Fact<f64>>     every period and member
//!           ├─ #[xbrl(nested)]          field: OtherStruct        same scope
//!           └─ #[xbrl(each_period)]     field: Vec<OtherStruct>   one scope per period
//! ```
//!
//! ## Describing a view
//!
//! ```
//! use serde::Serialize;
//! use xbrlkit::{Fact, FromXbrl};
//!
//! #[derive(Debug, Default, Serialize, FromXbrl)]
//! #[xbrl(instant)]
//! struct BalanceSheet {
//!     /// The date the figures are as of.
//!     #[xbrl(period_end)]
//!     as_of: Option<String>,
//!
//!     #[xbrl(concept = "us-gaap:Assets")]
//!     assets: Option<f64>,
//!
//!     /// The first concept the filing reports wins.
//!     #[xbrl(
//!         concept = "us-gaap:StockholdersEquity",
//!         alias = "us-gaap:StockholdersEquityIncludingPortionAttributableToNoncontrollingInterest"
//!     )]
//!     equity: Option<f64>,
//!
//!     /// Every class of stock, each with its dimension members.
//!     #[xbrl(concept = "us-gaap:CommonStockSharesOutstanding")]
//!     shares_outstanding: Vec<Fact<f64>>,
//! }
//!
//! #[derive(Debug, Default, Serialize, FromXbrl)]
//! struct Report {
//!     /// Field by field, the best the filing offers.
//!     #[xbrl(nested)]
//!     latest: BalanceSheet,
//!
//!     /// One balance sheet per date the filing reports, latest first.
//!     #[xbrl(each_period)]
//!     balance_sheets: Vec<BalanceSheet>,
//! }
//! ```
//!
//! ### Field attributes
//!
//! | Attribute                            | The field is                                             |
//! | ------------------------------------ | -------------------------------------------------------- |
//! | `concept = ".."` [, `alias = ".."`]… | read from these concepts, in order of preference         |
//! | `nested`                             | another `FromXbrl` struct, read in the same scope        |
//! | `each_period`                        | a `Vec` of a `FromXbrl` struct, one per reported period  |
//! | `period_start` / `period_end`        | the dates of the period the struct was read for          |
//! | *(none)*                             | left at its `Default`                                    |
//!
//! A concept written with its prefix (`us-gaap:Assets`) matches that concept;
//! written bare (`Assets`) it matches the name under any prefix.
//!
//! What a `concept` field holds is decided by its type: see [`FromFacts`].
//!
//! ### Struct attributes
//!
//! `#[xbrl(instant)]` or `#[xbrl(duration)]` says which kind of period the
//! struct's concepts are reported for, so that `each_period` does not produce
//! a balance sheet for a quarter or an income statement for a date.
//!
//! ## Which fact a field gets
//!
//! A struct is read in a [`Scope`]. Read on its own — [`Document::extract`] —
//! the scope is *unpinned*: each field takes the fact that best matches the
//! period the filing reports on, falling back to a comparative or a
//! dimensional breakdown when that is all the filing tags. Fields are chosen
//! independently, so an unpinned struct can hold figures from different
//! periods.
//!
//! Read through `#[xbrl(each_period)]`, the scope is *pinned*: every field
//! comes from the same period, from consolidated contexts only, and a concept
//! the filing does not report for that period is `None`. That is the view to
//! use when the period matters — a quarter's expenses rather than the year to
//! date's.
//!
//! Where a filing prints one figure twice — exactly on the statement, rounded
//! in a note — the more exact one is the value.
//!
//! ## Why the binding is not a serde rename
//!
//! `#[serde(rename = "us-gaap:Assets")]` with a custom `Deserializer` is the
//! obvious design, and this crate started with it. A serde name is the
//! field's name in *every* format, so the concept became the JSON key and the
//! Parquet column; and a rename split by direction does not fix that, because
//! tools that derive a schema from one direction and write with the other
//! (`serde_arrow` does) come out with null columns. `alias` fails differently:
//! serde rejects a second key for a field it has already seen, so a fallback
//! concept worked only while a filing tagged one of the two. Nor does serde
//! have anywhere to say *which* fact a field wants. So the binding has its own
//! attribute, and serde keeps the Rust names.

use crate::error::{Result, XbrlError};
use crate::instance::{Context, Instance, RawFact, Unit, XbrlValue};
use crate::parser;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::cmp::{Ordering, Reverse};
use std::collections::{BTreeSet, HashMap, HashSet};

/// The time a context covers: a date, or the span between two.
///
/// Ordered by `end`, then `start`, with an instant before any duration ending
/// the same day.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Span {
    /// The instant, or the last day of the duration (`"2025-03-31"`).
    pub end: String,

    /// The first day of the duration. `None` for an instant.
    pub start: Option<String>,
}

impl Span {
    /// A point in time, as a balance sheet is reported.
    pub fn instant(date: impl Into<String>) -> Self {
        Span {
            end: date.into(),
            start: None,
        }
    }

    /// A span of time, as an income statement is reported.
    pub fn duration(start: impl Into<String>, end: impl Into<String>) -> Self {
        Span {
            end: end.into(),
            start: Some(start.into()),
        }
    }

    /// Whether this is a point in time rather than a span.
    pub fn is_instant(&self) -> bool {
        self.start.is_none()
    }

    fn of(context: &Context) -> Option<Span> {
        let period = &context.period;
        let end = period.end_date.as_ref().or(period.instant.as_ref())?;
        let end = end.trim();
        if end.is_empty() {
            return None;
        }
        Some(Span {
            end: end.to_string(),
            start: period.start_date.as_ref().map(|s| s.trim().to_string()),
        })
    }
}

/// Which kind of period a struct's concepts are reported for.
///
/// Set with `#[xbrl(instant)]` or `#[xbrl(duration)]` on the struct; it limits
/// the periods `#[xbrl(each_period)]` produces one for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeriodKind {
    /// No restriction: a struct mixing both kinds.
    Any,
    /// Points in time: a balance sheet.
    Instant,
    /// Spans of time: an income statement, a cash flow statement.
    Duration,
}

impl PeriodKind {
    fn admits(self, span: &Span) -> bool {
        match self {
            PeriodKind::Any => true,
            PeriodKind::Instant => span.is_instant(),
            PeriodKind::Duration => !span.is_instant(),
        }
    }
}

/// One member of one axis: what narrows a fact below the whole entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dimension {
    /// The axis, e.g. `"us-gaap:StatementClassOfStockAxis"`.
    pub axis: String,

    /// The member on it, e.g. `"us-gaap:CommonClassAMember"`. For a typed
    /// dimension, the value the filing gives instead of a listed member.
    pub member: String,
}

/// A value together with the context it was reported in.
///
/// Use `Option<Fact<T>>` for a field whose period or unit matters, and
/// `Vec<Fact<T>>` for every period and dimension member a concept is reported
/// for — the shares outstanding of each class, say.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fact<T> {
    /// The value, converted to the field's type.
    pub value: T,

    /// First day of the period. `None` when the fact is as of a date.
    pub period_start: Option<String>,

    /// The date the fact is as of, or the last day of its period.
    pub period_end: Option<String>,

    /// The unit of measure without its namespace: `"USD"`, `"shares"`, `"USD/shares"`.
    pub unit: Option<String>,

    /// Decimal places the value is accurate to; negative for rounding to
    /// thousands (`-3`) or millions (`-6`). `None` when exact or not stated.
    pub decimals: Option<i32>,

    /// The dimension members the fact is reported for. Empty for the entity as a whole.
    #[serde(default)]
    pub dimensions: Vec<Dimension>,
}

/// How well a period lines up with the period the filing reports on.
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
    fn of(span: Option<&Span>, report: Option<&Span>) -> Self {
        let Some(span) = span else {
            return PeriodMatch::Unknown;
        };
        let Some(report) = report else {
            return PeriodMatch::BeforeReport;
        };

        match span.end.cmp(&report.end) {
            Ordering::Greater => PeriodMatch::AfterReport,
            Ordering::Less => PeriodMatch::BeforeReport,
            Ordering::Equal => match (&span.start, &report.start) {
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

/// A fact that could fill a field, with what it takes to choose between several.
struct Candidate<'a> {
    /// Position of the fact's concept in the field's list: `0` is the
    /// `concept`, the aliases follow in the order written.
    preference: usize,
    fact: &'a RawFact,
    context: Option<&'a Context>,
    span: Option<Span>,
    /// Number of dimension members narrowing the context.
    members: usize,
}

/// How exactly a fact states its value: its `decimals`, with `INF` above every
/// number and an absent or unreadable one below.
///
/// A filing tags a figure wherever it prints it, and the notes print it
/// rounded: a balance is `276,012,327` on the balance sheet (`decimals="0"`)
/// and "$276.0 million" in a note (`decimals="-5"`), under one concept and one
/// context. Between two such facts the exact one is the figure.
///
/// It says nothing between facts in different contexts: a class with no shares
/// outstanding is an exact `0`, and that is not a better count of another class.
fn precision(fact: &RawFact) -> i64 {
    match fact.decimals.as_deref().map(str::trim) {
        Some(decimals) if decimals.eq_ignore_ascii_case("INF") => i64::MAX,
        Some(decimals) => decimals.parse().unwrap_or(i64::MIN),
        None => i64::MIN,
    }
}

impl<'a> Candidate<'a> {
    /// What two printings of one fact have in common.
    fn duplicate_key(&self) -> (&'a str, Option<&'a str>) {
        (
            self.fact.full_name.as_str(),
            self.fact.context_ref.as_deref(),
        )
    }
}

/// Counts the dimension members that narrow a context. Zero means
/// consolidated, entity-wide data — the figure on a financial statement line.
fn member_count(context: &Context) -> usize {
    context.explicit_members().count() + context.typed_members().count()
}

fn dimensions_of(context: &Context) -> Vec<Dimension> {
    let explicit = context.explicit_members().map(|m| (&m.dimension, &m.value));
    let typed = context.typed_members().map(|m| (&m.dimension, &m.value));
    explicit
        .chain(typed)
        .map(|(axis, member)| Dimension {
            axis: axis.trim().to_string(),
            member: member.trim().to_string(),
        })
        .collect()
}

/// `"iso4217:USD"` → `"USD"`.
fn without_namespace(measure: &str) -> &str {
    let measure = measure.trim();
    measure.rsplit_once(':').map_or(measure, |(_, local)| local)
}

fn unit_label(unit: &Unit) -> String {
    match (&unit.measure, &unit.divide) {
        (Some(measure), _) => without_namespace(measure).to_string(),
        (None, Some(divide)) => format!(
            "{}/{}",
            without_namespace(&divide.unit_numerator.measure),
            without_namespace(&divide.unit_denominator.measure)
        ),
        (None, None) => unit.id.clone(),
    }
}

/// `YYYY-MM-DD`, by shape. Enough to tell a normalised date from "September 30, 2025".
fn is_iso_date(s: &str) -> bool {
    let bytes = s.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
}

/// A parsed filing, indexed for reading structs from it.
///
/// Parse once, then extract as many views as you need; every extraction
/// borrows the document.
///
/// ```
/// use xbrlkit::Document;
///
/// let doc = Document::parse(r#"
///     <xbrl xmlns="http://www.xbrl.org/2003/instance" xmlns:us-gaap="http://fasb.org/us-gaap/2025">
///       <context id="fy">
///         <entity><identifier scheme="http://www.sec.gov/CIK">0000320193</identifier></entity>
///         <period><instant>2025-09-27</instant></period>
///       </context>
///       <unit id="usd"><measure>iso4217:USD</measure></unit>
///       <us-gaap:Assets contextRef="fy" unitRef="usd" decimals="-6">359241000000</us-gaap:Assets>
///     </xbrl>"#)?;
///
/// assert_eq!(doc.facts().len(), 1);
/// assert_eq!(doc.get::<Option<f64>>(&["us-gaap:Assets"]), Some(359_241_000_000.0));
/// # Ok::<(), xbrlkit::XbrlError>(())
/// ```
pub struct Document {
    /// Every context, unit and fact the filing tags.
    instance: Instance,

    /// Fact positions by full concept name (`"us-gaap:Assets"`).
    by_full_name: HashMap<String, Vec<usize>>,

    /// Fact positions by local concept name (`"Assets"`).
    by_local_name: HashMap<String, Vec<usize>>,

    /// Context positions by id.
    contexts: HashMap<String, usize>,

    /// Unit labels by id.
    units: HashMap<String, String>,

    /// The period this filing reports on, which anchors fact selection.
    reporting_period: Option<Span>,
}

impl Document {
    /// Parses a filing, inline XBRL or a traditional XML instance, telling
    /// the two apart by the root element.
    ///
    /// Use [`from_ixbrl`](Self::from_ixbrl) or [`from_xml`](Self::from_xml)
    /// when you know which one you have.
    pub fn parse(content: &str) -> Result<Self> {
        match parser::is_xml_instance(content) {
            true => Self::from_xml(content),
            false => Self::from_ixbrl(content),
        }
    }

    /// Parses inline XBRL: the HTML of a 10-K, 10-Q or 8-K as EDGAR serves it.
    pub fn from_ixbrl(html: &str) -> Result<Self> {
        parser::parse_ixbrl(html).map(Self::new)
    }

    /// Parses a traditional XBRL instance: the `.xml` exhibit of an older
    /// filing, or the `_htm.xml` EDGAR extracts from an inline one.
    pub fn from_xml(xml: &str) -> Result<Self> {
        parser::parse_xml(xml).map(Self::new)
    }

    /// Indexes an already parsed instance.
    pub fn new(xbrl: Instance) -> Self {
        let mut by_full_name = HashMap::<String, Vec<usize>>::new();
        let mut by_local_name = HashMap::<String, Vec<usize>>::new();
        for (i, fact) in xbrl.facts.iter().enumerate() {
            by_full_name
                .entry(fact.full_name.clone())
                .or_default()
                .push(i);
            by_local_name
                .entry(fact.local_name.clone())
                .or_default()
                .push(i);
        }

        let contexts = xbrl
            .contexts
            .iter()
            .enumerate()
            .map(|(i, c)| (c.id.clone(), i))
            .collect();
        let units = xbrl
            .units
            .iter()
            .map(|u| (u.id.clone(), unit_label(u)))
            .collect();

        let mut data = Document {
            instance: xbrl,
            by_full_name,
            by_local_name,
            contexts,
            units,
            reporting_period: None,
        };
        data.reporting_period = data.resolve_reporting_period();
        data
    }

    /// Everything the parser read: contexts, units and facts, in document order.
    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    /// Gives the parsed instance back, dropping the indexes.
    pub fn into_instance(self) -> Instance {
        self.instance
    }

    /// Every fact the filing tags, in document order, as the parser read it.
    pub fn facts(&self) -> &[RawFact] {
        &self.instance.facts
    }

    /// Every context: the periods and dimension members facts are reported for.
    pub fn contexts(&self) -> &[Context] {
        &self.instance.contexts
    }

    /// Every unit of measure the filing declares.
    pub fn units(&self) -> &[Unit] {
        &self.instance.units
    }

    /// The period this filing reports on, from the SEC "Required Context".
    ///
    /// Every SEC filing tags `dei:DocumentPeriodEndDate` against the context
    /// that covers the reporting period, so that fact gives both the closing
    /// date (its value) and the span (its context). In a Q3 10-Q the required
    /// context runs from the start of the fiscal year to the quarter end, which
    /// is what tells the year-to-date column from the three-month column that
    /// shares its end date. `None` for a document that omits the fact, in which
    /// case selection degrades to preferring the most recent period.
    pub fn reporting_period(&self) -> Option<&Span> {
        self.reporting_period.as_ref()
    }

    fn resolve_reporting_period(&self) -> Option<Span> {
        let fact = self
            .facts_named("dei:DocumentPeriodEndDate")
            .next()
            .or_else(|| self.facts_named("DocumentPeriodEndDate").next())?;
        let context_span = self.context_of(fact).and_then(Span::of);

        // Prefer the tagged value; fall back to the context's own dates when the
        // value is missing or not a normalised ISO date.
        let end = match &fact.value {
            XbrlValue::String(s) if is_iso_date(s.trim()) => Some(s.trim().to_string()),
            _ => None,
        }
        .or_else(|| {
            context_span
                .as_ref()
                .map(|s| s.end.clone())
                .filter(|d| is_iso_date(d))
        })?;

        Some(Span {
            end,
            start: context_span.and_then(|s| s.start),
        })
    }

    /// Facts for a concept, in document order. A name with a prefix
    /// (`"us-gaap:Assets"`) matches exactly; a bare one (`"Assets"`) matches
    /// the concept in any namespace.
    fn facts_named<'a>(&'a self, concept: &str) -> impl Iterator<Item = &'a RawFact> + use<'a> {
        let index = if concept.contains(':') {
            &self.by_full_name
        } else {
            &self.by_local_name
        };
        index
            .get(concept)
            .into_iter()
            .flatten()
            .map(|&i| &self.instance.facts[i])
    }

    fn context_of(&self, fact: &RawFact) -> Option<&Context> {
        let id = fact.context_ref.as_deref()?;
        self.contexts.get(id).map(|&i| &self.instance.contexts[i])
    }

    fn candidates<'a>(&'a self, concepts: &[&str]) -> Vec<Candidate<'a>> {
        let mut candidates = Vec::new();
        for (preference, concept) in concepts.iter().enumerate() {
            for fact in self.facts_named(concept) {
                let context = self.context_of(fact);
                candidates.push(Candidate {
                    preference,
                    fact,
                    context,
                    span: context.and_then(Span::of),
                    members: context.map_or(0, member_count),
                });
            }
        }

        // One fact per concept and context: of the places a filing prints a
        // figure, the most exact. Equally exact ones all stay, in order.
        let mut most_exact: HashMap<(&str, Option<&str>), i64> = HashMap::new();
        for candidate in &candidates {
            let exact = most_exact
                .entry(candidate.duplicate_key())
                .or_insert(i64::MIN);
            *exact = (*exact).max(precision(candidate.fact));
        }
        candidates.retain(|c| precision(c.fact) == most_exact[&c.duplicate_key()]);
        candidates
    }

    /// Ranks two candidates for an unpinned field. Greater is better.
    ///
    /// 1. **Existence**: a fact with a context beats one without
    /// 2. **Period alignment**: the filing's reporting period beats
    ///    comparatives and post-period dates (see [`PeriodMatch`])
    /// 3. **Preference**: the `concept` beats its aliases, in the order written
    /// 4. **Proximity**: within a class, the period nearest the reporting date
    /// 5. **Dimensional simplicity**: consolidated data over a breakdown
    ///
    /// Alignment outranks both preference and dimensionality deliberately: a
    /// fallback concept, or a segment-tagged figure, for the period under
    /// report is still a figure about this period, whereas the preferred
    /// consolidated figure from the prior year is not.
    fn rank(&self, a: &Candidate<'_>, b: &Candidate<'_>) -> Ordering {
        let report = self.reporting_period.as_ref();
        let class_a = PeriodMatch::of(a.span.as_ref(), report);
        let class_b = PeriodMatch::of(b.span.as_ref(), report);

        a.context
            .is_some()
            .cmp(&b.context.is_some())
            .then_with(|| class_a.cmp(&class_b))
            .then_with(|| b.preference.cmp(&a.preference))
            .then_with(|| {
                let end_a = a.span.as_ref().map_or("", |s| s.end.as_str());
                let end_b = b.span.as_ref().map_or("", |s| s.end.as_str());
                // After the report, "nearer" means earlier: of a cover-page
                // date and a subsequent event, the cover page is the closer
                // approximation of the reporting date. Everywhere else the
                // later date wins, which is also the ordering when the
                // reporting period is unknown.
                match class_a {
                    PeriodMatch::AfterReport => end_b.cmp(end_a),
                    _ => end_a.cmp(end_b),
                }
            })
            .then_with(|| b.members.cmp(&a.members))
    }

    /// Reads one concept without declaring a struct for it: what a field of
    /// type `T` bound to `concepts` would hold.
    ///
    /// `T` chooses how much comes back, as it does for a field:
    /// `Option<f64>` for the best fact's value, `Option<Fact<f64>>` for that
    /// fact with its period, unit and dimensions, `Vec<Fact<f64>>` for every
    /// period and dimension member. `concepts` are in order of preference. A
    /// value that does not convert is left out.
    ///
    /// ```
    /// # use xbrlkit::{Document, Fact};
    /// # let doc = Document::new(Default::default());
    /// let revenue: Option<f64> = doc.get(&[
    ///     "us-gaap:RevenueFromContractWithCustomerExcludingAssessedTax",
    ///     "us-gaap:Revenues",
    /// ]);
    /// let shares_by_class: Vec<Fact<f64>> = doc.get(&["dei:EntityCommonStockSharesOutstanding"]);
    /// # assert!(revenue.is_none() && shares_by_class.is_empty());
    /// ```
    pub fn get<T: FromFacts>(&self, concepts: &[&str]) -> T {
        let problems = RefCell::new(Vec::new());
        let scope = Scope {
            data: self,
            period: None,
            problems: &problems,
        };
        T::from_facts(
            &scope,
            concepts,
            concepts.first().copied().unwrap_or_default(),
        )
    }

    /// Reads `T`, each field from the fact that best matches the filing's
    /// reporting period. Fails on the first value that does not convert to its
    /// field's type.
    pub fn extract<T: FromXbrl>(&self) -> Result<T> {
        let (value, mut problems) = self.extract_lenient::<T>();
        match problems.is_empty() {
            true => Ok(value),
            false => Err(problems.remove(0)),
        }
    }

    /// Reads `T` as [`extract`](Self::extract) does, but a value that does not
    /// convert leaves its field empty and is reported alongside, rather than
    /// costing the whole struct. A filer tagging "N/A" as a number should not
    /// lose the filing its balance sheet.
    pub fn extract_lenient<T: FromXbrl>(&self) -> (T, Vec<XbrlError>) {
        self.read(None)
    }

    /// Reads `T` for one period: every field from that period's consolidated
    /// contexts, `None` where the filing reports nothing for it.
    pub fn extract_for<T: FromXbrl>(&self, period: &Span) -> (T, Vec<XbrlError>) {
        self.read(Some(period.clone()))
    }

    /// Reads one `T` per period the filing reports `T`'s concepts for, latest first.
    pub fn extract_each_period<T: FromXbrl>(&self) -> (Vec<T>, Vec<XbrlError>) {
        let problems = RefCell::new(Vec::new());
        let scope = Scope {
            data: self,
            period: None,
            problems: &problems,
        };
        let value = each_period(&scope);
        (value, problems.into_inner())
    }

    fn read<T: FromXbrl>(&self, period: Option<Span>) -> (T, Vec<XbrlError>) {
        let problems = RefCell::new(Vec::new());
        let scope = Scope {
            data: self,
            period,
            problems: &problems,
        };
        let value = T::from_xbrl(&scope);
        (value, problems.into_inner())
    }
}

/// Where a struct is being read from: a document, and optionally one period of it.
pub struct Scope<'a> {
    data: &'a Document,

    /// The period every field must come from. `None` when unpinned.
    period: Option<Span>,

    /// Values that did not convert to their field's type.
    problems: &'a RefCell<Vec<XbrlError>>,
}

impl<'a> Scope<'a> {
    /// First day of the period this struct is read for. `None` for an instant,
    /// and when the scope is unpinned.
    pub fn period_start(&self) -> Option<String> {
        self.period.as_ref().and_then(|p| p.start.clone())
    }

    /// The date, or last day of the period, this struct is read for. `None`
    /// when the scope is unpinned.
    pub fn period_end(&self) -> Option<String> {
        self.period.as_ref().map(|p| p.end.clone())
    }

    /// The one fact a field gets.
    fn pick(&self, concepts: &[&str]) -> Option<Candidate<'a>> {
        let candidates = self.data.candidates(concepts);
        match &self.period {
            // Pinned: this period, the entity as a whole, the preferred concept.
            // `min_by_key` keeps the first of equals, so reverse to keep the last —
            // the same tie-break `max_by` gives the unpinned case.
            Some(period) => candidates
                .into_iter()
                .rev()
                .filter(|c| c.members == 0 && c.span.as_ref() == Some(period))
                .min_by_key(|c| c.preference),
            None => candidates.into_iter().max_by(|a, b| self.data.rank(a, b)),
        }
    }

    /// Every fact a field could show: all periods when unpinned, this one when
    /// pinned; all dimension members either way. Latest first, consolidated
    /// before its breakdowns.
    fn all(&self, concepts: &[&str]) -> Vec<Candidate<'a>> {
        let mut seen = HashSet::new();
        let mut facts: Vec<Candidate<'a>> = self
            .data
            .candidates(concepts)
            .into_iter()
            .filter(|c| c.fact.value != XbrlValue::Nil && c.span.is_some())
            .filter(|c| self.period.is_none() || c.span == self.period)
            // A filing tags the same fact wherever it prints it.
            .filter(|c| {
                seen.insert((
                    c.fact.full_name.as_str(),
                    c.fact.context_ref.as_deref(),
                    c.fact.value_key(),
                ))
            })
            .collect();
        facts.sort_by(|a, b| {
            Reverse(&a.span)
                .cmp(&Reverse(&b.span))
                .then_with(|| a.members.cmp(&b.members))
                .then_with(|| a.preference.cmp(&b.preference))
        });
        facts
    }

    /// Converts a fact's value, noting the failure rather than propagating it.
    fn convert<T: FactValue>(&self, fact: &RawFact, field: &str) -> Option<T> {
        let XbrlValue::String(raw) = &fact.value else {
            return None;
        };
        let converted = T::from_fact_value(raw);
        if converted.is_none() {
            self.problems.borrow_mut().push(XbrlError::ValueConversion {
                field: field.to_string(),
                concept: fact.full_name.clone(),
                value: raw.chars().take(80).collect(),
                target_type: T::TYPE_NAME,
            });
        }
        converted
    }

    fn with_context<T: FactValue>(
        &self,
        candidate: &Candidate<'a>,
        field: &str,
    ) -> Option<Fact<T>> {
        let value = self.convert(candidate.fact, field)?;
        let fact = candidate.fact;
        Some(Fact {
            value,
            period_start: candidate.span.as_ref().and_then(|s| s.start.clone()),
            period_end: candidate.span.as_ref().map(|s| s.end.clone()),
            unit: fact
                .unit_ref
                .as_deref()
                .and_then(|id| self.data.units.get(id).cloned()),
            decimals: fact.decimals.as_deref().and_then(|d| d.trim().parse().ok()),
            dimensions: candidate.context.map(dimensions_of).unwrap_or_default(),
        })
    }
}

impl RawFact {
    /// What makes two tags the same reported value.
    fn value_key(&self) -> &str {
        match &self.value {
            XbrlValue::String(s) => s.as_str(),
            XbrlValue::Nil => "",
        }
    }
}

/// A type a fact's value converts to.
pub trait FactValue: Sized {
    /// For error messages.
    const TYPE_NAME: &'static str;

    /// Converts the normalised text of a fact. `None` when it is not one of these.
    fn from_fact_value(raw: &str) -> Option<Self>;
}

impl FactValue for String {
    const TYPE_NAME: &'static str = "String";

    fn from_fact_value(raw: &str) -> Option<Self> {
        Some(raw.to_string())
    }
}

impl FactValue for f64 {
    const TYPE_NAME: &'static str = "f64";

    fn from_fact_value(raw: &str) -> Option<Self> {
        raw.trim().replace(',', "").parse().ok()
    }
}

impl FactValue for bool {
    const TYPE_NAME: &'static str = "bool";

    fn from_fact_value(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        if raw.eq_ignore_ascii_case("true") || raw.eq_ignore_ascii_case("yes") {
            Some(true)
        } else if raw.eq_ignore_ascii_case("false") || raw.eq_ignore_ascii_case("no") {
            Some(false)
        } else {
            None
        }
    }
}

macro_rules! integer_fact_value {
    ($($ty:ty),*) => {$(
        impl FactValue for $ty {
            const TYPE_NAME: &'static str = stringify!($ty);

            fn from_fact_value(raw: &str) -> Option<Self> {
                let raw = raw.trim().replace(',', "");
                // "3" and "3.0" are both three; "3.5" is not an integer.
                let whole = match raw.split_once('.') {
                    Some((whole, frac)) if frac.bytes().all(|b| b == b'0') => whole,
                    Some(_) => return None,
                    None => raw.as_str(),
                };
                whole.parse().ok()
            }
        }
    )*};
}
integer_fact_value!(i32, i64, u32, u64);

/// What a `#[xbrl(concept = "..")]` field can be. The field's *type* chooses
/// how much of the fact table it sees:
///
/// | Field type        | Holds                                                    |
/// | ----------------- | -------------------------------------------------------- |
/// | `Option<T>`       | the value of the one best fact                           |
/// | `Option<Fact<T>>` | that fact with its period, unit and dimensions           |
/// | `Vec<Fact<T>>`    | every fact for the concepts: each period, each member    |
///
/// where `T` is a [`FactValue`]: `f64`, `i64`, `i32`, `u64`, `u32`, `bool` or `String`.
pub trait FromFacts: Sized {
    /// `concepts` are in order of preference; `field` names the field for error messages.
    fn from_facts(scope: &Scope<'_>, concepts: &[&str], field: &str) -> Self;
}

impl<T: FactValue> FromFacts for Option<T> {
    fn from_facts(scope: &Scope<'_>, concepts: &[&str], field: &str) -> Self {
        scope
            .pick(concepts)
            .and_then(|c| scope.convert(c.fact, field))
    }
}

impl<T: FactValue> FromFacts for Option<Fact<T>> {
    fn from_facts(scope: &Scope<'_>, concepts: &[&str], field: &str) -> Self {
        scope
            .pick(concepts)
            .and_then(|c| scope.with_context(&c, field))
    }
}

impl<T: FactValue> FromFacts for Vec<Fact<T>> {
    fn from_facts(scope: &Scope<'_>, concepts: &[&str], field: &str) -> Self {
        scope
            .all(concepts)
            .iter()
            .filter_map(|c| scope.with_context(c, field))
            .collect()
    }
}

/// A struct that can be read from an XBRL document. Derive it:
/// `#[derive(FromXbrl)]`, with the fields bound by `#[xbrl(..)]`.
pub trait FromXbrl: Sized {
    /// Which kind of period the struct's concepts are reported for.
    const PERIOD_KIND: PeriodKind = PeriodKind::Any;

    /// Every concept the struct, and the structs nested in it, read from.
    fn concepts(out: &mut Vec<&'static str>);

    /// Reads the struct in a scope.
    fn from_xbrl(scope: &Scope<'_>) -> Self;
}

/// One `T` per period the filing reports any of `T`'s concepts for, latest
/// first. Only consolidated contexts count: a period that appears solely in a
/// dimensional breakdown has no statement of its own.
///
/// Within a scope that is already pinned, the periods are narrowed to that one.
pub fn each_period<T: FromXbrl>(scope: &Scope<'_>) -> Vec<T> {
    let mut concepts = Vec::new();
    T::concepts(&mut concepts);

    let periods: BTreeSet<Span> = scope
        .data
        .candidates(&concepts)
        .into_iter()
        .filter(|c| c.members == 0 && c.fact.value != XbrlValue::Nil)
        .filter_map(|c| c.span)
        .filter(|span| T::PERIOD_KIND.admits(span))
        .filter(|span| scope.period.as_ref().is_none_or(|p| p == span))
        .collect();

    periods
        .into_iter()
        .rev()
        .map(|period| {
            T::from_xbrl(&Scope {
                data: scope.data,
                period: Some(period),
                problems: scope.problems,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::{Entity, ExplicitMember, Identifier, Period, Segment};

    fn context(id: &str, period: Period) -> Context {
        Context {
            id: id.to_string(),
            entity: Entity {
                identifier: Identifier {
                    scheme: "cik".to_string(),
                    value: "123".to_string(),
                },
                segment: None,
            },
            period,
            scenario: None,
        }
    }

    fn instant(id: &str, date: &str) -> Context {
        context(
            id,
            Period {
                instant: Some(date.to_string()),
                start_date: None,
                end_date: None,
            },
        )
    }

    fn duration(id: &str, start: &str, end: &str) -> Context {
        context(
            id,
            Period {
                instant: None,
                start_date: Some(start.to_string()),
                end_date: Some(end.to_string()),
            },
        )
    }

    fn with_member(mut context: Context, axis: &str, member: &str) -> Context {
        context.entity.segment = Some(Segment {
            explicit_members: vec![ExplicitMember {
                dimension: axis.to_string(),
                value: member.to_string(),
            }],
            typed_members: Vec::new(),
        });
        context
    }

    fn fact(name: &str, context: &str, value: &str) -> RawFact {
        RawFact {
            full_name: name.to_string(),
            local_name: name.rsplit(':').next().unwrap().to_string(),
            context_ref: Some(context.to_string()),
            value: XbrlValue::String(value.to_string()),
            ..Default::default()
        }
    }

    /// A Q3 10-Q: year-to-date is the required context.
    fn q3_document(mut facts: Vec<RawFact>, mut contexts: Vec<Context>) -> Document {
        contexts.push(duration("ytd", "2024-01-01", "2024-09-30"));
        contexts.push(duration("q3", "2024-07-01", "2024-09-30"));
        contexts.push(duration("prior_ytd", "2023-01-01", "2023-09-30"));
        contexts.push(instant("now", "2024-09-30"));
        contexts.push(instant("year_end", "2023-12-31"));
        contexts.push(instant("cover", "2024-11-12"));
        facts.push(fact("dei:DocumentPeriodEndDate", "ytd", "2024-09-30"));
        Document::new(Instance {
            contexts,
            units: Vec::new(),
            facts,
        })
    }

    fn one<T: FromFacts>(data: &Document, period: Option<Span>, concepts: &[&str]) -> T {
        let problems = RefCell::new(Vec::new());
        let scope = Scope {
            data,
            period,
            problems: &problems,
        };
        T::from_facts(&scope, concepts, "field")
    }

    #[test]
    fn reporting_period_is_the_required_context() {
        let data = q3_document(Vec::new(), Vec::new());
        assert_eq!(
            data.reporting_period(),
            Some(&Span::duration("2024-01-01", "2024-09-30"))
        );
    }

    #[test]
    fn period_match_classification() {
        let report = Span::duration("2025-01-01", "2025-03-31");
        let class = |span: Span| PeriodMatch::of(Some(&span), Some(&report));

        assert_eq!(
            class(Span::instant("2025-03-31")),
            PeriodMatch::RequiredContext
        );
        assert_eq!(
            class(Span::instant("2024-12-31")),
            PeriodMatch::BeforeReport
        );
        assert_eq!(class(Span::instant("2025-05-12")), PeriodMatch::AfterReport);
        assert_eq!(
            class(Span::duration("2025-02-01", "2025-03-31")),
            PeriodMatch::EndsOnReportDate
        );
        assert_eq!(PeriodMatch::of(None, Some(&report)), PeriodMatch::Unknown);
    }

    #[test]
    fn balance_sheet_date_beats_prior_year_end_and_cover_page() {
        // Undimensioned instants either side of the reporting date: the shape
        // that makes a naive reader report the prior year end's total assets.
        let data = q3_document(
            vec![
                fact("us-gaap:Assets", "cover", "3"),
                fact("us-gaap:Assets", "now", "1"),
                fact("us-gaap:Assets", "year_end", "2"),
            ],
            Vec::new(),
        );
        assert_eq!(
            one::<Option<f64>>(&data, None, &["us-gaap:Assets"]),
            Some(1.0)
        );
    }

    #[test]
    fn required_context_beats_shorter_span_with_same_end() {
        let data = q3_document(
            vec![
                fact("us-gaap:OperatingExpenses", "ytd", "900"),
                fact("us-gaap:OperatingExpenses", "q3", "300"),
            ],
            Vec::new(),
        );
        assert_eq!(
            one::<Option<f64>>(&data, None, &["us-gaap:OperatingExpenses"]),
            Some(900.0)
        );
    }

    #[test]
    fn consolidated_beats_dimensional_within_same_period() {
        let class_a = with_member(
            instant("now_a", "2024-09-30"),
            "us-gaap:StatementClassOfStockAxis",
            "us-gaap:CommonClassAMember",
        );
        let data = q3_document(
            vec![
                fact("us-gaap:StockholdersEquity", "now", "10"),
                fact("us-gaap:StockholdersEquity", "now_a", "4"),
            ],
            vec![class_a],
        );
        assert_eq!(
            one::<Option<f64>>(&data, None, &["us-gaap:StockholdersEquity"]),
            Some(10.0)
        );
    }

    #[test]
    fn the_exact_figure_beats_its_rounded_restatement() {
        let stated = |value: &str, decimals: &str| RawFact {
            decimals: Some(decimals.to_string()),
            ..fact("us-gaap:AssetsHeldInTrustNoncurrent", "now", value)
        };
        // Document order: the balance sheet, then the note that rounds it.
        let data = q3_document(
            vec![stated("276012327", "0"), stated("276000000", "-5")],
            Vec::new(),
        );
        let concepts = ["us-gaap:AssetsHeldInTrustNoncurrent"];

        assert_eq!(
            one::<Option<f64>>(&data, None, &concepts),
            Some(276012327.0)
        );
        assert_eq!(
            one::<Option<f64>>(&data, Some(Span::instant("2024-09-30")), &concepts),
            Some(276012327.0)
        );
        // And once in a list of every value, not once per printing.
        assert_eq!(one::<Vec<Fact<f64>>>(&data, None, &concepts).len(), 1);
    }

    #[test]
    fn precision_does_not_choose_between_contexts() {
        let class = |id: &str, member: &str| {
            with_member(
                instant(id, "2024-09-30"),
                "us-gaap:StatementClassOfStockAxis",
                member,
            )
        };
        let stated = |context: &str, value: &str, decimals: &str| RawFact {
            decimals: Some(decimals.to_string()),
            ..fact("us-gaap:CommonStockSharesOutstanding", context, value)
        };
        // Class A has none outstanding, stated exactly; class B's count is not
        // a worse answer for being a whole number of shares.
        let data = q3_document(
            vec![stated("a", "0", "INF"), stated("b", "7906250", "0")],
            vec![
                class("a", "us-gaap:CommonClassAMember"),
                class("b", "us-gaap:CommonClassBMember"),
            ],
        );
        assert_eq!(
            one::<Option<f64>>(&data, None, &["us-gaap:CommonStockSharesOutstanding"]),
            Some(7906250.0)
        );
    }

    #[test]
    fn without_a_reporting_period_the_most_recent_wins() {
        let data = Document::new(Instance {
            contexts: vec![instant("old", "2023-03-31"), instant("new", "2024-03-31")],
            units: Vec::new(),
            facts: vec![
                fact("us-gaap:Assets", "new", "2"),
                fact("us-gaap:Assets", "old", "1"),
            ],
        });
        assert_eq!(
            one::<Option<f64>>(&data, None, &["us-gaap:Assets"]),
            Some(2.0)
        );
    }

    #[test]
    fn alias_is_a_fallback_in_the_order_written() {
        let both = q3_document(
            vec![
                fact("us-gaap:AssetsHeldInTrust", "now", "2"),
                fact("us-gaap:AssetsHeldInTrustNoncurrent", "now", "1"),
            ],
            Vec::new(),
        );
        let concepts = [
            "us-gaap:AssetsHeldInTrustNoncurrent",
            "us-gaap:AssetsHeldInTrust",
        ];
        // Both tagged: the concept wins. With serde's `alias` this is a
        // "duplicate field" error that costs the filing its whole struct.
        assert_eq!(one::<Option<f64>>(&both, None, &concepts), Some(1.0));

        let only_alias = q3_document(
            vec![fact("us-gaap:AssetsHeldInTrust", "now", "2")],
            Vec::new(),
        );
        assert_eq!(one::<Option<f64>>(&only_alias, None, &concepts), Some(2.0));
    }

    #[test]
    fn an_alias_for_this_period_beats_the_concept_for_last_year() {
        let data = q3_document(
            vec![
                fact("us-gaap:AssetsHeldInTrustNoncurrent", "year_end", "1"),
                fact("us-gaap:AssetsHeldInTrust", "now", "2"),
            ],
            Vec::new(),
        );
        let concepts = [
            "us-gaap:AssetsHeldInTrustNoncurrent",
            "us-gaap:AssetsHeldInTrust",
        ];
        assert_eq!(one::<Option<f64>>(&data, None, &concepts), Some(2.0));
    }

    #[test]
    fn a_pinned_scope_reads_only_its_own_period() {
        let class_a = with_member(
            duration("q3_a", "2024-07-01", "2024-09-30"),
            "us-gaap:StatementClassOfStockAxis",
            "us-gaap:CommonClassAMember",
        );
        let data = q3_document(
            vec![
                fact("us-gaap:OperatingExpenses", "ytd", "900"),
                fact("us-gaap:OperatingExpenses", "q3", "300"),
                fact("us-gaap:NetIncomeLoss", "ytd", "-50"),
                fact("us-gaap:EarningsPerShareBasic", "q3_a", "0.02"),
            ],
            vec![class_a],
        );
        let q3 = Some(Span::duration("2024-07-01", "2024-09-30"));

        assert_eq!(
            one::<Option<f64>>(&data, q3.clone(), &["us-gaap:OperatingExpenses"]),
            Some(300.0)
        );
        // Reported for the year to date only: nothing for the quarter.
        assert_eq!(
            one::<Option<f64>>(&data, q3.clone(), &["us-gaap:NetIncomeLoss"]),
            None
        );
        // Reported for the quarter, but only per class: not a consolidated figure.
        assert_eq!(
            one::<Option<f64>>(&data, q3, &["us-gaap:EarningsPerShareBasic"]),
            None
        );
    }

    #[test]
    fn a_vec_of_facts_holds_every_period_and_member() {
        let class_a = with_member(
            instant("cover_a", "2024-11-12"),
            "us-gaap:StatementClassOfStockAxis",
            "us-gaap:CommonClassAMember",
        );
        let class_b = with_member(
            instant("cover_b", "2024-11-12"),
            "us-gaap:StatementClassOfStockAxis",
            "us-gaap:CommonClassBMember",
        );
        let concept = "dei:EntityCommonStockSharesOutstanding";
        let data = q3_document(
            vec![
                fact(concept, "cover_a", "23000000"),
                fact(concept, "cover_b", "5750000"),
                // The cover page and the balance sheet both print it.
                fact(concept, "cover_b", "5750000"),
            ],
            vec![class_a, class_b],
        );

        let by_class = one::<Vec<Fact<f64>>>(&data, None, &[concept]);
        let seen: Vec<(f64, &str)> = by_class
            .iter()
            .map(|f| (f.value, f.dimensions[0].member.as_str()))
            .collect();
        assert_eq!(
            seen,
            vec![
                (23000000.0, "us-gaap:CommonClassAMember"),
                (5750000.0, "us-gaap:CommonClassBMember"),
            ]
        );
        assert_eq!(by_class[0].period_end.as_deref(), Some("2024-11-12"));
        assert_eq!(by_class[0].period_start, None);
    }

    #[test]
    fn a_value_that_does_not_convert_is_reported_and_left_empty() {
        let data = q3_document(
            vec![
                fact("us-gaap:Assets", "now", "N/A"),
                fact("us-gaap:Liabilities", "now", "1,250"),
            ],
            Vec::new(),
        );
        let problems = RefCell::new(Vec::new());
        let scope = Scope {
            data: &data,
            period: None,
            problems: &problems,
        };
        assert_eq!(
            <Option<f64>>::from_facts(&scope, &["us-gaap:Assets"], "assets"),
            None
        );
        assert_eq!(
            <Option<f64>>::from_facts(&scope, &["us-gaap:Liabilities"], "liabilities"),
            Some(1250.0)
        );
        let problems = problems.into_inner();
        assert_eq!(problems.len(), 1);
        assert!(
            problems[0]
                .to_string()
                .contains("`assets` (us-gaap:Assets)")
        );
    }

    #[test]
    fn integers_accept_a_whole_number_however_it_is_written() {
        assert_eq!(i64::from_fact_value("6,900,000"), Some(6_900_000));
        assert_eq!(i32::from_fact_value("1.0"), Some(1));
        assert_eq!(i32::from_fact_value("1.5"), None);
        assert_eq!(bool::from_fact_value("Yes"), Some(true));
        assert_eq!(bool::from_fact_value("maybe"), None);
    }
}
