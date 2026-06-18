# MeroAlpha Terminal

Desktop terminal for local NEPSE portfolio tracking and market analysis.

## Portfolio CSV

The first portfolio slice accepts a holdings snapshot CSV with these internal columns:

```csv
symbol,quantity,avg_cost,ltp
NABIL,2450,450.20,562.10
GBIME,3100,190.50,215.00
```

This snapshot model replaces current holdings on import. The portfolio domain computes market value, cost basis, unrealized P/L, unrealized P/L percent, holding weights, and top holding.

It also accepts the observed MeroShare-style export shape:

```csv
"S.N","Scrip","Current Balance","Last Closing Price","Value as of Last Closing Price","Last Transaction Price (LTP)","Value as of LTP"
"1","ALBSL","12.0","1097.6","13171.20","1097.8","13173.60"
```

For that export, `Last Closing Price` is used as the temporary cost basis until the app has a true trade-ledger import.

## Current Slice

- GPUI/gpui-component desktop shell.
- Portfolio page based on `design/screen.png`.
- Tested CSV parsing and portfolio math.
- SQLite-backed portfolio repository using `rusqlite`.

## Next Slices

- Native file picker for CSV import.
- File-backed SQLite database path and migrations.
- MeroAlpha market-data sync for LTP and corporate actions.
- Broker analysis and market watch pages.
