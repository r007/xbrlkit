# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

`xbrlkit` and `xbrlkit-derive` are released together under one version.

## [Unreleased]

## [0.1.1] - 2026-10-09

### Changed

- Upgraded `quick-xml` from 0.37 to 0.42. Names and text are `&str` there, and
  entity and character references arrive as events of their own, so the parser
  joins the pieces around them. An instance fact whose text holds a reference
  (`Smith &amp; Sons`) is read whole, and its whitespace is trimmed after the
  join, not before

## [0.1.0] - 2026-10-09

The first release as a crate of its own. The parser was written in 2025 inside
a data pipeline for SEC filings and has run in production there since; the
history before this release is that crate's.

### Added

- `Document`, a parsed filing indexed for reading structs from it:
  `Document::parse` tells inline XBRL from an XML instance by the root
  element, and `from_ixbrl` and `from_xml` read one or the other
- `#[derive(FromXbrl)]`, binding a struct's fields to concepts with
  `#[xbrl(concept = "..", alias = "..")]`, `nested`, `each_period`,
  `period_start` and `period_end`. A field's type chooses what it holds:
  `Option<T>` for the best fact's value, `Option<Fact<T>>` for that fact with
  its period, unit, precision and dimensions, `Vec<Fact<T>>` for every period
  and dimension member
- `Document::extract`, `extract_lenient`, `extract_for` and
  `extract_each_period`, and `Document::get` for reading one concept without
  declaring a struct
- An inline XBRL parser that reads a fact's value through whatever HTML wraps
  it, follows `ix:continuation`, leaves out `ix:exclude`, reads facts nested in
  other facts, and applies `format`, `scale` and `sign`
- An XML instance parser for traditional filings and for the instances EDGAR
  extracts from inline ones
- Inline transformations: the SEC's `ixt-sec` registry, and the numeric and
  English date rules of the Transformation Rules Registry under both their
  hyphenated and run-together names
- `taxonomies::dei`, the cover page of a filing, and `taxonomies::us_gaap`,
  the balance sheet, income statement and cash flow statement, once for the
  reporting period and once per period the filing reports
- Explicit and typed dimension members on contexts, and `Fact::dimensions`
- The `derive` and `taxonomies` features, both on by default

[Unreleased]: https://github.com/r007/xbrlkit/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/r007/xbrlkit/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/r007/xbrlkit/releases/tag/v0.1.0
