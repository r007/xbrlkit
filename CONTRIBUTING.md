# Contributing

## Reporting a filing that reads wrongly

This is the most useful kind of report. Include:

- the filing: its accession number, or the URL of the document on EDGAR
- the concept, e.g. `us-gaap:Assets`
- what you got and what the filing shows

If the value is wrong coming out of the parser, `cargo run --example facts --
filing.htm us-gaap:Assets` shows every fact the parser found for the concept.
If the facts are right and a struct picked the wrong one, say which struct and
field.

## Development

```bash
cargo test --workspace --all-features
cargo clippy --workspace --all-features --all-targets -- -D warnings
cargo fmt --all
```

CI also builds the crate without its default features, on the minimum
supported Rust version, and builds the documentation with warnings denied.

### Layout

| Path                     | Holds                                                              |
| ------------------------ | ------------------------------------------------------------------ |
| `src/parser.rs`          | Text to facts: the inline XBRL and XML instance parsers            |
| `src/transformations.rs` | The `format` rules of inline facts                                 |
| `src/instance.rs`        | What the parser produces: contexts, units, raw facts               |
| `src/bind.rs`            | `Document`, fact selection, and the traits the derive expands to   |
| `src/taxonomies/`        | The ready-made `dei` and `us_gaap` structs                         |
| `derive/`                | `xbrlkit-derive`, the proc-macro crate                             |
| `tests/`                 | Integration tests, and the filings they read in `tests/fixtures`   |
| `benches/parse.rs`       | `cargo bench`                                                      |
| `benches/compare/`       | A standalone project timing other XBRL crates on the same files    |
| `examples/edgar/`        | A standalone project fetching a filing with edgarkit               |

### Tests

A parser fix should come with the markup that needed it. For something a few
lines long, write the inline document in the test, as
`tests/parser_inline_values.rs` does. Add a whole filing to `tests/fixtures`
only when the behaviour depends on the filing as a whole; fixtures are real
documents of a megabyte or more, and they are excluded from the published
crate.

`tests/sec_conformance.rs` compares the inline parser with the instance EDGAR
extracted from the same filing. A change to how inline values are read should
keep it passing.

### Adding a field to `taxonomies`

The `us_gaap` structs hold the line items most filers report on the face of
their statements. A new field should be one of those, name its concepts in
order of how common they are, and say in its doc comment what it is in plain
words. Fields for one industry or one kind of filer belong in a struct of your
own; that is what the derive is for.

## Releasing

1. Move the `[Unreleased]` entries in `CHANGELOG.md` under the new version
2. Set the version in the workspace `Cargo.toml`, and the `xbrlkit-derive`
   requirement beside it
3. `cargo publish -p xbrlkit-derive`, then `cargo publish -p xbrlkit`
4. Tag the commit `vX.Y.Z` and push the tag
