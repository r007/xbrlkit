//! `#[derive(FromXbrl)]` through the public API: every attribute, and every
//! type a field can have.
#![cfg(feature = "derive")]

use serde::{Deserialize, Serialize};
use xbrlkit::{Document, Fact, FromXbrl, Span, XbrlError};

/// A small first-quarter report: this quarter and last year's, a balance
/// sheet for the quarter end and the year end, and two classes of stock.
const QUARTERLY_REPORT: &str = r#"<html xmlns:ix="http://www.xbrl.org/2013/inlineXBRL"><body>
<ix:header><ix:resources>
  <xbrli:context id="q1"><xbrli:entity><xbrli:identifier scheme="http://www.sec.gov/CIK">0001234567</xbrli:identifier></xbrli:entity>
    <xbrli:period><xbrli:startDate>2025-01-01</xbrli:startDate><xbrli:endDate>2025-03-31</xbrli:endDate></xbrli:period></xbrli:context>
  <xbrli:context id="prior_q1"><xbrli:entity><xbrli:identifier scheme="http://www.sec.gov/CIK">0001234567</xbrli:identifier></xbrli:entity>
    <xbrli:period><xbrli:startDate>2024-01-01</xbrli:startDate><xbrli:endDate>2024-03-31</xbrli:endDate></xbrli:period></xbrli:context>
  <xbrli:context id="now"><xbrli:entity><xbrli:identifier scheme="http://www.sec.gov/CIK">0001234567</xbrli:identifier></xbrli:entity>
    <xbrli:period><xbrli:instant>2025-03-31</xbrli:instant></xbrli:period></xbrli:context>
  <xbrli:context id="year_end"><xbrli:entity><xbrli:identifier scheme="http://www.sec.gov/CIK">0001234567</xbrli:identifier></xbrli:entity>
    <xbrli:period><xbrli:instant>2024-12-31</xbrli:instant></xbrli:period></xbrli:context>
  <xbrli:context id="now_a"><xbrli:entity><xbrli:identifier scheme="http://www.sec.gov/CIK">0001234567</xbrli:identifier>
      <xbrli:segment><xbrldi:explicitMember dimension="us-gaap:StatementClassOfStockAxis">us-gaap:CommonClassAMember</xbrldi:explicitMember></xbrli:segment></xbrli:entity>
    <xbrli:period><xbrli:instant>2025-03-31</xbrli:instant></xbrli:period></xbrli:context>
  <xbrli:context id="now_b"><xbrli:entity><xbrli:identifier scheme="http://www.sec.gov/CIK">0001234567</xbrli:identifier>
      <xbrli:segment><xbrldi:explicitMember dimension="us-gaap:StatementClassOfStockAxis">us-gaap:CommonClassBMember</xbrldi:explicitMember></xbrli:segment></xbrli:entity>
    <xbrli:period><xbrli:instant>2025-03-31</xbrli:instant></xbrli:period></xbrli:context>
  <xbrli:unit id="usd"><xbrli:measure>iso4217:USD</xbrli:measure></xbrli:unit>
  <xbrli:unit id="shares"><xbrli:measure>xbrli:shares</xbrli:measure></xbrli:unit>
</ix:resources></ix:header>

<ix:nonNumeric name="dei:EntityRegistrantName" contextRef="q1"><b>Example Holdings Inc.</b></ix:nonNumeric>
<ix:nonNumeric name="dei:DocumentPeriodEndDate" contextRef="q1" format="ixt:date-monthname-day-year-en">March 31, 2025</ix:nonNumeric>
<ix:nonNumeric name="dei:EntityEmergingGrowthCompany" contextRef="q1" format="ixt-sec:boolballotbox">☒</ix:nonNumeric>
<ix:nonFraction name="dei:EntityNumberOfEmployees" contextRef="q1" unitRef="shares" decimals="0" format="ixt:num-dot-decimal">1,250</ix:nonFraction>

<ix:nonFraction name="us-gaap:Assets" contextRef="now" unitRef="usd" scale="3" decimals="-3" format="ixt:num-dot-decimal">5,400</ix:nonFraction>
<ix:nonFraction name="us-gaap:Assets" contextRef="year_end" unitRef="usd" scale="3" decimals="-3" format="ixt:num-dot-decimal">5,000</ix:nonFraction>
<ix:nonFraction name="us-gaap:LiabilitiesNoncurrent" contextRef="now" unitRef="usd" scale="3" decimals="-3" format="ixt:num-dot-decimal">1,100</ix:nonFraction>
<ix:nonFraction name="us-gaap:Liabilities" contextRef="year_end" unitRef="usd" scale="3" decimals="-3" format="ixt:num-dot-decimal">900</ix:nonFraction>

<ix:nonFraction name="us-gaap:Revenues" contextRef="q1" unitRef="usd" scale="3" decimals="-3" format="ixt:num-dot-decimal">2,000</ix:nonFraction>
<ix:nonFraction name="us-gaap:Revenues" contextRef="prior_q1" unitRef="usd" scale="3" decimals="-3" format="ixt:num-dot-decimal">1,600</ix:nonFraction>
(<ix:nonFraction name="us-gaap:NetIncomeLoss" contextRef="q1" unitRef="usd" scale="3" decimals="-3" sign="-" format="ixt:num-dot-decimal">150</ix:nonFraction>)
<ix:nonFraction name="us-gaap:NetIncomeLoss" contextRef="prior_q1" unitRef="usd" scale="3" decimals="-3" format="ixt:num-dot-decimal">80</ix:nonFraction>

<ix:nonFraction name="us-gaap:CommonStockSharesOutstanding" contextRef="now_a" unitRef="shares" decimals="0" format="ixt:num-dot-decimal">10,000,000</ix:nonFraction>
<ix:nonFraction name="us-gaap:CommonStockSharesOutstanding" contextRef="now_b" unitRef="shares" decimals="0" format="ixt:num-dot-decimal">2,500,000</ix:nonFraction>
</body></html>"#;

fn report() -> Document {
    Document::parse(QUARTERLY_REPORT).expect("the report parses")
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, FromXbrl)]
#[xbrl(instant)]
struct BalanceSheet {
    #[xbrl(period_end)]
    as_of: Option<String>,

    #[xbrl(concept = "us-gaap:Assets")]
    assets: Option<f64>,

    #[xbrl(
        concept = "us-gaap:Liabilities",
        alias = "us-gaap:LiabilitiesNoncurrent"
    )]
    liabilities: Option<f64>,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, FromXbrl)]
#[xbrl(duration)]
struct IncomeStatement {
    #[xbrl(period_start)]
    from: Option<String>,

    #[xbrl(period_end)]
    to: Option<String>,

    #[xbrl(concept = "us-gaap:Revenues")]
    revenue: Option<f64>,

    #[xbrl(concept = "us-gaap:NetIncomeLoss")]
    net_income: Option<f64>,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, FromXbrl)]
struct Report {
    #[xbrl(concept = "dei:EntityRegistrantName")]
    name: Option<String>,

    #[xbrl(concept = "dei:EntityEmergingGrowthCompany")]
    emerging_growth: Option<bool>,

    #[xbrl(concept = "dei:EntityNumberOfEmployees")]
    employees: Option<i64>,

    /// With its period, unit and precision.
    #[xbrl(concept = "us-gaap:Revenues")]
    revenue: Option<Fact<f64>>,

    /// Every class.
    #[xbrl(concept = "us-gaap:CommonStockSharesOutstanding")]
    shares: Vec<Fact<f64>>,

    #[xbrl(nested)]
    balance_sheet: BalanceSheet,

    #[xbrl(each_period)]
    balance_sheets: Vec<BalanceSheet>,

    #[xbrl(each_period)]
    income_statements: Vec<IncomeStatement>,

    /// Not bound: filled by the caller.
    accession_number: Option<String>,
}

#[test]
fn a_field_takes_the_value_for_the_period_the_filing_reports_on() {
    let report: Report = report().extract().unwrap();

    assert_eq!(report.name.as_deref(), Some("Example Holdings Inc."));
    assert_eq!(report.emerging_growth, Some(true));
    assert_eq!(report.employees, Some(1250));
    // The quarter end, not the year end that is tagged after it.
    assert_eq!(report.balance_sheet.assets, Some(5_400_000.0));
    assert_eq!(report.balance_sheet.as_of, None);
    assert_eq!(report.accession_number, None);
}

#[test]
fn an_alias_stands_in_for_a_concept_the_filing_does_not_report() {
    let report: Report = report().extract().unwrap();

    // `Liabilities` is tagged only for last year end, `LiabilitiesNoncurrent`
    // for this quarter: the alias for this period is the better answer.
    assert_eq!(report.balance_sheet.liabilities, Some(1_100_000.0));
    // Pinned to one date, each sheet takes what that date has.
    assert_eq!(report.balance_sheets[0].liabilities, Some(1_100_000.0));
    assert_eq!(report.balance_sheets[1].liabilities, Some(900_000.0));
}

#[test]
fn each_period_gives_one_struct_per_period_latest_first() {
    let report: Report = report().extract().unwrap();

    assert_eq!(
        report.balance_sheets,
        vec![
            BalanceSheet {
                as_of: Some("2025-03-31".into()),
                assets: Some(5_400_000.0),
                liabilities: Some(1_100_000.0),
            },
            BalanceSheet {
                as_of: Some("2024-12-31".into()),
                assets: Some(5_000_000.0),
                liabilities: Some(900_000.0),
            },
        ]
    );
    assert_eq!(
        report.income_statements,
        vec![
            IncomeStatement {
                from: Some("2025-01-01".into()),
                to: Some("2025-03-31".into()),
                revenue: Some(2_000_000.0),
                // Printed as (150): the sign lives in the tag.
                net_income: Some(-150_000.0),
            },
            IncomeStatement {
                from: Some("2024-01-01".into()),
                to: Some("2024-03-31".into()),
                revenue: Some(1_600_000.0),
                net_income: Some(80_000.0),
            },
        ]
    );
}

#[test]
fn a_fact_carries_its_period_unit_precision_and_dimensions() {
    let report: Report = report().extract().unwrap();

    let revenue = report.revenue.unwrap();
    assert_eq!(revenue.value, 2_000_000.0);
    assert_eq!(revenue.period_start.as_deref(), Some("2025-01-01"));
    assert_eq!(revenue.period_end.as_deref(), Some("2025-03-31"));
    assert_eq!(revenue.unit.as_deref(), Some("USD"));
    assert_eq!(revenue.decimals, Some(-3));
    assert!(revenue.dimensions.is_empty());

    let classes: Vec<(&str, f64)> = report
        .shares
        .iter()
        .map(|fact| (fact.dimensions[0].member.as_str(), fact.value))
        .collect();
    assert_eq!(
        classes,
        vec![
            ("us-gaap:CommonClassAMember", 10_000_000.0),
            ("us-gaap:CommonClassBMember", 2_500_000.0),
        ]
    );
    assert_eq!(
        report.shares[0].dimensions[0].axis,
        "us-gaap:StatementClassOfStockAxis"
    );
    assert_eq!(report.shares[0].unit.as_deref(), Some("shares"));
}

#[test]
fn a_struct_serializes_under_its_field_names() {
    let report: Report = report().extract().unwrap();

    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(json["balance_sheet"]["assets"], 5_400_000.0);
    assert_eq!(json["income_statements"][0]["net_income"], -150_000.0);
    assert_eq!(
        json["shares"][1]["dimensions"][0]["member"],
        "us-gaap:CommonClassBMember"
    );
    assert!(!json.to_string().contains("us-gaap:Assets"));

    assert_eq!(serde_json::from_value::<Report>(json).unwrap(), report);
}

#[test]
fn one_concept_can_be_read_without_a_struct() {
    let doc = report();

    assert_eq!(
        doc.get::<Option<f64>>(&["us-gaap:Assets"]),
        Some(5_400_000.0)
    );
    assert_eq!(
        doc.get::<Option<f64>>(&["us-gaap:Liabilities", "us-gaap:LiabilitiesNoncurrent"]),
        Some(1_100_000.0)
    );
    assert_eq!(doc.get::<Option<String>>(&["us-gaap:Goodwill"]), None);

    // Every period: this quarter and last year's.
    let revenue: Vec<Fact<f64>> = doc.get(&["us-gaap:Revenues"]);
    assert_eq!(revenue.len(), 2);
    assert_eq!(revenue[0].period_end.as_deref(), Some("2025-03-31"));

    // A name without a prefix matches the concept under any.
    assert_eq!(doc.get::<Option<f64>>(&["Assets"]), Some(5_400_000.0));
}

#[test]
fn one_period_can_be_asked_for_by_name() {
    let doc = report();

    let (sheet, problems) = doc.extract_for::<BalanceSheet>(&Span::instant("2024-12-31"));
    assert!(problems.is_empty());
    assert_eq!(sheet.assets, Some(5_000_000.0));
    assert_eq!(sheet.as_of.as_deref(), Some("2024-12-31"));

    // A date the filing reports nothing for is an empty struct, not an error.
    let (sheet, _) = doc.extract_for::<BalanceSheet>(&Span::instant("2020-12-31"));
    assert_eq!(sheet.assets, None);
}

#[derive(Debug, Default, FromXbrl)]
struct Mistyped {
    /// A name, read as a number.
    #[xbrl(concept = "dei:EntityRegistrantName")]
    name: Option<f64>,

    #[xbrl(concept = "us-gaap:Assets")]
    assets: Option<f64>,
}

#[test]
fn a_value_of_the_wrong_type_fails_strictly_and_is_skipped_leniently() {
    let doc = report();

    match doc.extract::<Mistyped>() {
        Err(XbrlError::ValueConversion {
            field,
            concept,
            value,
            target_type,
        }) => {
            assert_eq!(field, "name");
            assert_eq!(concept, "dei:EntityRegistrantName");
            assert_eq!(value, "Example Holdings Inc.");
            assert_eq!(target_type, "f64");
        }
        other => panic!("expected a conversion error, got {other:?}"),
    }

    let (mistyped, problems) = doc.extract_lenient::<Mistyped>();
    assert_eq!(mistyped.name, None);
    assert_eq!(mistyped.assets, Some(5_400_000.0));
    assert_eq!(problems.len(), 1);
}

#[test]
fn an_empty_document_gives_an_empty_struct() {
    let doc = Document::parse("<html><body>nothing tagged</body></html>").unwrap();
    assert!(doc.facts().is_empty());
    assert_eq!(doc.reporting_period(), None);
    assert_eq!(doc.extract::<Report>().unwrap(), Report::default());
}
