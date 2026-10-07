//! # What reaches storage
//!
//! The taxonomy structs are written to Delta through `serde_arrow`, and read
//! back the same way. These tests hold them to that: Rust names for columns,
//! a lossless round trip, and the per-period statements a real filing yields.

use arrow::datatypes::{DataType, FieldRef};
use serde::{Serialize, de::DeserializeOwned};
use serde_arrow::schema::{SchemaLike, TracingOptions};
use serde_json::Value;
use std::fs::read_to_string;
use xbrl::taxonomies::{dei::DeiInfo, us_gaap::Financials};
use xbrl::{XbrlDataContext, from_ixbrl_str};

/// A Q3 10-Q: three-month and nine-month columns, two share classes, a
/// stockholders' deficit, and a trust most of the way through redemptions.
const FORM_10Q_Q3: &str = "../fixtures/html/10-q_3.html";

fn q3() -> XbrlDataContext {
    let content = read_to_string(FORM_10Q_Q3).expect("fixture should be readable");
    from_ixbrl_str(&content).expect("fixture should parse")
}

/// The options the pipeline derives its Delta schemas with (`common::delta::tracing_options`).
fn tracing_options() -> TracingOptions {
    TracingOptions::default()
        .strings_as_large_utf8(false)
        .map_as_struct(true)
        .coerce_numbers(true)
        .enums_without_data_as_strings(true)
        .allow_null_fields(true)
}

fn json_keys(value: &Value, keys: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                keys.push(key.clone());
                json_keys(child, keys);
            }
        }
        Value::Array(items) => items.iter().for_each(|item| json_keys(item, keys)),
        _ => {}
    }
}

fn column_names(fields: &[FieldRef], names: &mut Vec<String>) {
    for field in fields {
        names.push(field.name().clone());
        match field.data_type() {
            DataType::Struct(children) => column_names(children, names),
            DataType::List(item) | DataType::LargeList(item) => {
                if let DataType::Struct(children) = item.data_type() {
                    column_names(children, names);
                }
            }
            _ => {}
        }
    }
}

/// Derives the schema, writes one row, reads it back, and returns the columns.
fn round_trip<T: Serialize + DeserializeOwned>(value: &T) -> Vec<String> {
    let fields = Vec::<FieldRef>::from_type::<T>(tracing_options()).expect("schema should derive");
    let batch = serde_arrow::to_record_batch(&fields, &[value]).expect("row should serialize");
    let back: Vec<T> = serde_arrow::from_record_batch(&batch).expect("row should deserialize");

    assert_eq!(
        serde_json::to_value(&back[0]).unwrap(),
        serde_json::to_value(value).unwrap(),
        "what is read back from Arrow is what was written"
    );

    let mut names = Vec::new();
    column_names(&fields, &mut names);
    names
}

#[test]
fn stored_names_are_rust_names_not_concepts() {
    let data = q3();
    let financials: Financials = data.extract().unwrap();
    let dei: DeiInfo = data.extract().unwrap();

    let mut keys = Vec::new();
    json_keys(&serde_json::to_value(&financials).unwrap(), &mut keys);
    json_keys(&serde_json::to_value(&dei).unwrap(), &mut keys);
    keys.extend(round_trip(&financials));
    keys.extend(round_trip(&dei));

    let namespaced: Vec<&String> = keys.iter().filter(|k| k.contains(':')).collect();
    assert!(
        namespaced.is_empty(),
        "concept names leaked into storage: {namespaced:?}"
    );
    for expected in [
        "assets",
        "assets_held_in_trust_noncurrent",
        "auditor_name",
        "period_end",
    ] {
        assert!(
            keys.iter().any(|k| k == expected),
            "missing column {expected}"
        );
    }
}

#[test]
fn an_empty_document_round_trips_too() {
    // Every list empty, every figure null: the shape of a filing with no iXBRL.
    round_trip(&Financials::default());
    round_trip(&DeiInfo::default());
}

#[test]
fn a_row_written_before_a_field_existed_still_reads() {
    // Rows already in the lake have no `balance_sheets`, no `as_of`.
    let old: Financials = serde_json::from_str(r#"{"balance_sheet": {"assets": 5.0}}"#).unwrap();
    assert_eq!(old.balance_sheet.assets, Some(5.0));
    assert!(old.income_statements.is_empty());
}

#[test]
fn unpinned_statements_report_the_year_to_date() {
    let financials: Financials = q3().extract().unwrap();
    let income = &financials.income_statement;

    assert_eq!(income.general_and_administrative_expense, Some(1840635.0));
    // Unpinned fields can come from different periods, so the struct names none.
    assert_eq!(income.period_end, None);
    // A deficit is negative: the filing prints "(16,068,717)" and tags sign="-".
    assert_eq!(
        financials.balance_sheet.stockholders_equity,
        Some(-16068717.0)
    );
}

#[test]
fn each_period_has_its_own_income_statement() {
    let financials: Financials = q3().extract().unwrap();
    let expense = |start: &str, end: &str| {
        financials
            .income_statements
            .iter()
            .find(|s| {
                s.period_start.as_deref() == Some(start) && s.period_end.as_deref() == Some(end)
            })
            .unwrap_or_else(|| panic!("no income statement for {start}..{end}"))
            .general_and_administrative_expense
    };

    // The quarter, beside the year to date it is part of.
    assert_eq!(expense("2023-07-01", "2023-09-30"), Some(557054.0));
    assert_eq!(expense("2023-01-01", "2023-09-30"), Some(1840635.0));
    // And last year's comparatives.
    assert_eq!(expense("2022-07-01", "2022-09-30"), Some(163133.0));
    assert_eq!(expense("2022-01-01", "2022-09-30"), Some(783051.0));

    // Latest first; an income statement is never for a single date.
    let ends: Vec<&str> = financials
        .income_statements
        .iter()
        .map(|s| s.period_end.as_deref().unwrap())
        .collect();
    assert!(ends.is_sorted_by(|a, b| a >= b));
    assert!(
        financials
            .income_statements
            .iter()
            .all(|s| s.period_start.is_some())
    );
}

#[test]
fn each_date_has_its_own_balance_sheet() {
    let financials: Financials = q3().extract().unwrap();
    let trust = |date: &str| {
        financials
            .balance_sheets
            .iter()
            .find(|b| b.as_of.as_deref() == Some(date))
            .unwrap_or_else(|| panic!("no balance sheet as of {date}"))
            .assets_held_in_trust_noncurrent
    };

    // Redemptions between the two dates took the trust from $258M to $12.5M.
    assert_eq!(trust("2023-09-30"), Some(12518199.0));
    assert_eq!(trust("2022-12-31"), Some(257725405.0));
    assert_eq!(
        financials.balance_sheets[0].as_of.as_deref(),
        Some("2023-09-30")
    );
}

#[test]
fn shares_outstanding_are_reported_per_class() {
    let dei: DeiInfo = q3().extract().unwrap();
    let by_class: Vec<(&str, f64)> = dei
        .entity
        .common_stock_shares_outstanding_by_class
        .iter()
        .map(|fact| (fact.dimensions[0].member.as_str(), fact.value))
        .collect();

    assert_eq!(
        by_class,
        vec![
            ("us-gaap:CommonClassAMember", 2253021.0),
            ("us-gaap:CommonClassBMember", 6535000.0),
        ]
    );
    let class_a = &dei.entity.common_stock_shares_outstanding_by_class[0];
    assert_eq!(class_a.unit.as_deref(), Some("shares"));
    assert_eq!(
        class_a.dimensions[0].axis,
        "us-gaap:StatementClassOfStockAxis"
    );
}
