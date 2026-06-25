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
- Portfolio page based on real imported holdings.
- Native MeroShare CSV file picker import.
- Tested CSV parsing and portfolio math.
- SQLite-backed portfolio repository using `rusqlite`.
- Local profile name and MeroAlpha Data Platform API key settings stored in SQLite.
- Overview page backed by MeroAlpha Data API market/index data, with live local portfolio performance when holdings are imported.

## Overview Page Data Boundary

The Overview page is a local terminal dashboard. Market Pulse uses `GET /v1/indices` with `GET /v1/sub-indices` as an empty-index fallback. Top Movers use `GET /v1/market/movers` for gainers, losers, and turnover leaders on the latest traded date.

Portfolio Performance is derived from the locally imported holdings snapshot when available. If no API key is configured or the API returns no rows, Overview panels show an empty/error state rather than placeholder market data.

The page is informational only. It does not place trades, send broker orders, or provide execution controls.

## Local Settings

The sidebar footer opens the local profile editor. Users can update:

- Profile name shown in the sidebar.
- MeroAlpha Data Platform API key for upcoming market-data integration.

The app stores the API key locally in the same SQLite database. It is only used for explicit MeroAlpha Data Platform refresh actions.

## Price Refresh

After importing holdings and saving a MeroAlpha Data Platform API key, use `Refresh Prices` on the portfolio page to update stale CSV prices.

The app calls:

```text
GET /v1/prices/daily?symbol={SYMBOL}&date={LAST_TRADED_DATE}&adjusted=false&limit=1
Authorization: Bearer {API_KEY}
```

If the daily-price endpoint rejects a symbol with `400`, the app treats it as a possible mutual fund and falls back to:

```text
GET /v1/mutual-funds/nav?symbol={SYMBOL}&period=range&limit=1
Authorization: Bearer {API_KEY}
```

Only `ltp` is replaced. Quantity and cost basis stay local, so imported holdings remain the source of truth for position size and average cost.

Some internal or closed mutual funds may not exist on NEPSE or in the MeroAlpha Data Platform. When both upstream price routes are unavailable, the app keeps the imported local LTP; if that LTP is zero or negative, it applies a small `NPR 1.00` local floor so the holding can still be valued.

## Next Slices

- Corporate actions sync.
- Broker analysis and market watch pages.
