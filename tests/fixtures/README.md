# Fixtures

Real filings, as EDGAR serves them. They are public documents; none is
modified.

| File                         | Filer                                  | Form | Period     | Accession            |
| ---------------------------- | -------------------------------------- | ---- | ---------- | -------------------- |
| `aapl-10k-2025.htm`          | Apple Inc.                             | 10-K | 2025-09-27 | 0000320193-25-000079 |
| `aapl-10k-2025_htm.xml`      | the instance EDGAR extracted from it   |      |            |                      |
| `tsla-10q-2026q2.htm`        | Tesla, Inc.                            | 10-Q | 2026-06-30 | 0001628280-26-049270 |
| `tsla-10q-2026q2_htm.xml`    | the instance EDGAR extracted from it   |      |            |                      |
| `aapl-8k-2026.htm`           | Apple Inc.                             | 8-K  | 2026-07-30 | 0000320193-26-000018 |

## `spac/`

Periodic and current reports of special purpose acquisition companies: small
filers, produced by a handful of filing agents whose output differs from the
large filers' in the ways that matter to a parser. These are where the crate
was first used.

| File             | Filer                                   | Form | Period     | As                 |
| ---------------- | --------------------------------------- | ---- | ---------- | ------------------ |
| `10-q.html`      | EQV Ventures Acquisition Corp. II       | 10-Q | 2025-06-30 | inline XBRL        |
| `10-q_1.html`    | Cartesian Growth Corp III               | 10-Q | 2025-06-30 | inline XBRL        |
| `10-q_2.html`    | Dune Acquisition Corporation II         | 10-Q | 2025-06-30 | inline XBRL        |
| `10-q_3.html`    | Newcourt Acquisition Corp               | 10-Q | 2023-09-30 | inline XBRL        |
| `8-k.html`       | Alchemy Investments Acquisition Corp 1  | 8-K  | 2025-08-22 | inline XBRL        |
| `8-k_1.html`     | WeWork Inc.                             | 8-K  | 2021-11-29 | inline XBRL        |
| `form_10q.xml`   | Keen Vision Acquisition Corporation     | 10-Q | 2024-09-30 | XML instance       |
| `form_10q_1.xml` | Keen Vision Acquisition Corporation     | 10-Q | 2025-03-31 | XML instance       |
| `form_10q_2.xml` | Cartesian Growth Corp III               | 10-Q | 2025-03-31 | XML instance       |
| `form_10q_3.xml` | AA Mission Acquisition Corp.            | 10-Q | 2025-03-31 | XML instance       |
| `form_10q_4.xml` | Alchemy Investments Acquisition Corp 1  | 10-Q | 2025-03-31 | XML instance       |
