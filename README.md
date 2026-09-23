# `xbrl`

Reads the financial statements a filing tags in XBRL — the traditional XML
instance or the inline iXBRL embedded in a 10-K or 10-Q's HTML — into typed
figures. It is how a periodic report's numbers reach the record without a model
call: the 10-K, 10-Q and 8-K processors read their financials through it, and
every `SpacSnapshot` on a SPAC's history is built from those figures alone.

| API                                         | Reads                                                                |
| ------------------------------------------- | -------------------------------------------------------------------- |
| `extract_xbrl_data()` / `from_xbrl_str()`   | A traditional XBRL XML instance                                      |
| `extract_ixbrl_data()` / `from_ixbrl_str()` | Inline XBRL in a filing's HTML                                       |
| `us_gaap::extract_financials()`             | The balance sheet, income statement and cash flows                   |
| `dei::extract_dei()`                        | Document and Entity Information: CIK, entity name, period            |
| `XbrlDataContext`                           | A fact's context, with the SEC's value transformations applied (50+) |
