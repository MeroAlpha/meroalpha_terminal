REFERENCE
GET
/v1/reference

PUBLIC
NO CURSOR
Machine-readable API contract for public and session-scoped v1 routes.


curl -X GET "https://api.meroalpha.com/v1/reference" \
  -H "Authorization: Bearer YOUR_API_KEY"
PRICES
GET
/v1/prices/daily

PUBLIC
CURSOR
Daily OHLCV rows with source metadata and signed opaque cursors for paging.

Cursor pagination: Keep the meta.next_cursor token returned from a response and send it unchanged as cursor query parameter to fetch the next page.
PARAMETERS

symbol
REQUIRED
Symbol · symbol
Default: NABIL
period
OPTIONAL
Period · period
Default: single
date
OPTIONAL
Date · date
Default: today
from
OPTIONAL
From · from
Default: today-30
to
OPTIONAL
To · to
Default: today
adjusted
OPTIONAL
Adjusted · boolean
Default: false
limit
OPTIONAL
Limit · limit
Default: 10
cursor
OPTIONAL
Signed cursor · cursor

curl -X GET "https://api.meroalpha.com/v1/prices/daily?symbol=NABIL&period=single&date=2026-06-15&from=2026-05-16&to=2026-06-15&adjusted=false&limit=10&cursor=NABIL" \
  -H "Authorization: Bearer YOUR_API_KEY"
AUTH
POST
/v1/auth/signup

PUBLIC
NO CURSOR
Create a user account and session.


curl -X POST "https://api.meroalpha.com/v1/auth/signup" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
  "email": "user@example.com",
  "password": "••••••••"
}'
POST
/v1/auth/login

PUBLIC
NO CURSOR
Create a cookie-backed user session.


curl -X POST "https://api.meroalpha.com/v1/auth/login" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
  "email": "user@example.com",
  "password": "••••••••"
}'
POST
/v1/auth/logout

USER_SESSION
NO CURSOR
Clear the active user session cookie.


curl -X POST "https://api.meroalpha.com/v1/auth/logout" \
  -H "Authorization: Bearer YOUR_API_KEY"
GET
/v1/auth/me

USER_SESSION
NO CURSOR
Return the active user session profile.


curl -X GET "https://api.meroalpha.com/v1/auth/me" \
  -H "Authorization: Bearer YOUR_API_KEY"
DASHBOARD
GET
/v1/dashboard/activity

USER_SESSION
NO CURSOR
Recent dashboard activity for the active user.


curl -X GET "https://api.meroalpha.com/v1/dashboard/activity" \
  -H "Authorization: Bearer YOUR_API_KEY"
COMPANIES
GET
/v1/companies

PUBLIC
CURSOR
List all listed companies on the exchange with signed opaque cursor paging.

Cursor pagination: Keep the meta.next_cursor token returned from a response and send it unchanged as cursor query parameter to fetch the next page.
PARAMETERS

limit
OPTIONAL
Limit · limit
Default: 10
cursor
OPTIONAL
Signed cursor · cursor

curl -X GET "https://api.meroalpha.com/v1/companies?limit=10&cursor=NABIL" \
  -H "Authorization: Bearer YOUR_API_KEY"
GET
/v1/companies/:symbol

PUBLIC
NO CURSOR
Corporate metadata, status, listing dates, and description.

PARAMETERS

symbol
REQUIRED
Symbol · symbol
Default: NABIL

curl -X GET "https://api.meroalpha.com/v1/companies/:symbol?symbol=NABIL" \
  -H "Authorization: Bearer YOUR_API_KEY"
GET
/v1/companies/:symbol/profile

PUBLIC
NO CURSOR
Company profile alias with corporate metadata, status, listing dates, and description.

PARAMETERS

symbol
REQUIRED
Symbol · symbol
Default: NABIL

curl -X GET "https://api.meroalpha.com/v1/companies/:symbol/profile?symbol=NABIL" \
  -H "Authorization: Bearer YOUR_API_KEY"
GET
/v1/companies/:symbol/corporate-actions

PUBLIC
CURSOR
Corporate action history for one company with signed opaque cursor paging.

Cursor pagination: Keep the meta.next_cursor token returned from a response and send it unchanged as cursor query parameter to fetch the next page.
PARAMETERS

symbol
REQUIRED
Symbol · symbol
Default: NABIL
limit
OPTIONAL
Limit · limit
Default: 10
cursor
OPTIONAL
Signed cursor · cursor

curl -X GET "https://api.meroalpha.com/v1/companies/:symbol/corporate-actions?symbol=NABIL&limit=10&cursor=NABIL" \
  -H "Authorization: Bearer YOUR_API_KEY"
GET
/v1/companies/:symbol/metrics

PUBLIC
NO CURSOR
Computed metrics summary for one company.

PARAMETERS

symbol
REQUIRED
Symbol · symbol
Default: NABIL

curl -X GET "https://api.meroalpha.com/v1/companies/:symbol/metrics?symbol=NABIL" \
  -H "Authorization: Bearer YOUR_API_KEY"
FINANCIALS
GET
/v1/companies/:symbol/financials

PUBLIC
CURSOR
Historical financial report metadata with signed opaque cursor paging.

Cursor pagination: Keep the meta.next_cursor token returned from a response and send it unchanged as cursor query parameter to fetch the next page.
PARAMETERS

symbol
REQUIRED
Symbol · symbol
Default: NABIL
limit
OPTIONAL
Limit · limit
Default: 10
cursor
OPTIONAL
Signed cursor · cursor

curl -X GET "https://api.meroalpha.com/v1/companies/:symbol/financials?symbol=NABIL&limit=10&cursor=NABIL" \
  -H "Authorization: Bearer YOUR_API_KEY"
GET
/v1/companies/:symbol/financials/sources

PUBLIC
CURSOR
Source linkages and upstream document references with signed opaque cursor paging.

Cursor pagination: Keep the meta.next_cursor token returned from a response and send it unchanged as cursor query parameter to fetch the next page.
PARAMETERS

symbol
REQUIRED
Symbol · symbol
Default: NABIL
limit
OPTIONAL
Limit · limit
Default: 10
cursor
OPTIONAL
Signed cursor · cursor

curl -X GET "https://api.meroalpha.com/v1/companies/:symbol/financials/sources?symbol=NABIL&limit=10&cursor=NABIL" \
  -H "Authorization: Bearer YOUR_API_KEY"
GET
/v1/companies/:symbol/financials/income-statement

PUBLIC
CURSOR
Income statement items including revenue, EPS, and profits with signed opaque cursor paging.

Cursor pagination: Keep the meta.next_cursor token returned from a response and send it unchanged as cursor query parameter to fetch the next page.
PARAMETERS

symbol
REQUIRED
Symbol · symbol
Default: NABIL
limit
OPTIONAL
Limit · limit
Default: 10
cursor
OPTIONAL
Signed cursor · cursor

curl -X GET "https://api.meroalpha.com/v1/companies/:symbol/financials/income-statement?symbol=NABIL&limit=10&cursor=NABIL" \
  -H "Authorization: Bearer YOUR_API_KEY"
GET
/v1/companies/:symbol/financials/balance-sheet

PUBLIC
CURSOR
Balance sheet statement items containing assets and liabilities with signed opaque cursor paging.

Cursor pagination: Keep the meta.next_cursor token returned from a response and send it unchanged as cursor query parameter to fetch the next page.
PARAMETERS

symbol
REQUIRED
Symbol · symbol
Default: NABIL
limit
OPTIONAL
Limit · limit
Default: 10
cursor
OPTIONAL
Signed cursor · cursor

curl -X GET "https://api.meroalpha.com/v1/companies/:symbol/financials/balance-sheet?symbol=NABIL&limit=10&cursor=NABIL" \
  -H "Authorization: Bearer YOUR_API_KEY"
GET
/v1/companies/:symbol/financials/key-metrics

PUBLIC
CURSOR
Quarterly key metrics including EPS and capital ratios with signed opaque cursor paging.

Cursor pagination: Keep the meta.next_cursor token returned from a response and send it unchanged as cursor query parameter to fetch the next page.
PARAMETERS

symbol
REQUIRED
Symbol · symbol
Default: NABIL
limit
OPTIONAL
Limit · limit
Default: 10
cursor
OPTIONAL
Signed cursor · cursor

curl -X GET "https://api.meroalpha.com/v1/companies/:symbol/financials/key-metrics?symbol=NABIL&limit=10&cursor=NABIL" \
  -H "Authorization: Bearer YOUR_API_KEY"
MARKET
GET
/v1/market/status

PUBLIC
NO CURSOR
Current status of the exchange.


curl -X GET "https://api.meroalpha.com/v1/market/status" \
  -H "Authorization: Bearer YOUR_API_KEY"
GET
/v1/indices

PUBLIC
CURSOR
Current sector index values and changes with signed opaque cursor paging.

Cursor pagination: Keep the meta.next_cursor token returned from a response and send it unchanged as cursor query parameter to fetch the next page.
PARAMETERS

limit
OPTIONAL
Limit · limit
Default: 10
cursor
OPTIONAL
Signed cursor · cursor

curl -X GET "https://api.meroalpha.com/v1/indices?limit=10&cursor=NABIL" \
  -H "Authorization: Bearer YOUR_API_KEY"
GET
/v1/sub-indices

PUBLIC
CURSOR
Current sector sub-index values and changes with signed opaque cursor paging.

Cursor pagination: Keep the meta.next_cursor token returned from a response and send it unchanged as cursor query parameter to fetch the next page.
PARAMETERS

limit
OPTIONAL
Limit · limit
Default: 10
cursor
OPTIONAL
Signed cursor · cursor

curl -X GET "https://api.meroalpha.com/v1/sub-indices?limit=10&cursor=NABIL" \
  -H "Authorization: Bearer YOUR_API_KEY"
FLOORSHEET
GET
/v1/floorsheet

PUBLIC
CURSOR
Page through individual trade transactions with signed opaque cursors.

Cursor pagination: Keep the meta.next_cursor token returned from a response and send it unchanged as cursor query parameter to fetch the next page.
PARAMETERS

symbol
REQUIRED
Symbol · symbol
Default: NABIL
date
OPTIONAL
Date · date
Default: today
limit
OPTIONAL
Limit · limit
Default: 10
cursor
OPTIONAL
Signed cursor · cursor

curl -X GET "https://api.meroalpha.com/v1/floorsheet?symbol=NABIL&date=2026-06-15&limit=10&cursor=NABIL" \
  -H "Authorization: Bearer YOUR_API_KEY"
GET
/v1/floorsheet/summary

PUBLIC
NO CURSOR
Aggregated trade details by broker pair or total transaction volume.

PARAMETERS

symbol
REQUIRED
Symbol · symbol
Default: NABIL
date
OPTIONAL
Date · date
Default: today

curl -X GET "https://api.meroalpha.com/v1/floorsheet/summary?symbol=NABIL&date=2026-06-15" \
  -H "Authorization: Bearer YOUR_API_KEY"
BROKERS
GET
/v1/brokers/:broker_id/activity

PUBLIC
NO CURSOR
Transaction and volume summary metrics for a given broker.

PARAMETERS

broker_id
REQUIRED
Broker ID · broker_id
Default: 22
date
OPTIONAL
Date · date
Default: today

curl -X GET "https://api.meroalpha.com/v1/brokers/:broker_id/activity?broker_id=22&date=2026-06-15" \
  -H "Authorization: Bearer YOUR_API_KEY"
GET
/v1/brokers/:broker_id/symbols

PUBLIC
CURSOR
Distinct symbols traded by a broker on a given trading day with signed opaque cursor paging.

Cursor pagination: Keep the meta.next_cursor token returned from a response and send it unchanged as cursor query parameter to fetch the next page.
PARAMETERS

broker_id
REQUIRED
Broker ID · broker_id
Default: 22
date
OPTIONAL
Date · date
Default: today
limit
OPTIONAL
Limit · limit
Default: 10
cursor
OPTIONAL
Signed cursor · cursor

curl -X GET "https://api.meroalpha.com/v1/brokers/:broker_id/symbols?broker_id=22&date=2026-06-15&limit=10&cursor=NABIL" \
  -H "Authorization: Bearer YOUR_API_KEY"
CORPORATE ACTIONS
GET
/v1/corporate-actions/dividends

PUBLIC
CURSOR
Dividend announcements with cash and bonus percentages using signed opaque cursor paging.

Cursor pagination: Keep the meta.next_cursor token returned from a response and send it unchanged as cursor query parameter to fetch the next page.
PARAMETERS

symbol
REQUIRED
Symbol · symbol
Default: NABIL
limit
OPTIONAL
Limit · limit
Default: 10
cursor
OPTIONAL
Signed cursor · cursor

curl -X GET "https://api.meroalpha.com/v1/corporate-actions/dividends?symbol=NABIL&limit=10&cursor=NABIL" \
  -H "Authorization: Bearer YOUR_API_KEY"
GET
/v1/corporate-actions/agms

PUBLIC
CURSOR
Annual General Meeting schedules, locations, and agenda highlights using signed opaque cursor paging.

Cursor pagination: Keep the meta.next_cursor token returned from a response and send it unchanged as cursor query parameter to fetch the next page.
PARAMETERS

symbol
REQUIRED
Symbol · symbol
Default: NABIL
limit
OPTIONAL
Limit · limit
Default: 10
cursor
OPTIONAL
Signed cursor · cursor

curl -X GET "https://api.meroalpha.com/v1/corporate-actions/agms?symbol=NABIL&limit=10&cursor=NABIL" \
  -H "Authorization: Bearer YOUR_API_KEY"
GET
/v1/corporate-actions/book-closures

PUBLIC
CURSOR
Share registry book closure schedules for benefit eligibility tracking using signed opaque cursor paging.

Cursor pagination: Keep the meta.next_cursor token returned from a response and send it unchanged as cursor query parameter to fetch the next page.
PARAMETERS

symbol
REQUIRED
Symbol · symbol
Default: NABIL
limit
OPTIONAL
Limit · limit
Default: 10
cursor
OPTIONAL
Signed cursor · cursor

curl -X GET "https://api.meroalpha.com/v1/corporate-actions/book-closures?symbol=NABIL&limit=10&cursor=NABIL" \
  -H "Authorization: Bearer YOUR_API_KEY"
GET
/v1/corporate-actions/bonus-shares

PUBLIC
CURSOR
Bonus share allocations, distribution ratios, and listing details using signed opaque cursor paging.

Cursor pagination: Keep the meta.next_cursor token returned from a response and send it unchanged as cursor query parameter to fetch the next page.
PARAMETERS

symbol
REQUIRED
Symbol · symbol
Default: NABIL
limit
OPTIONAL
Limit · limit
Default: 10
cursor
OPTIONAL
Signed cursor · cursor

curl -X GET "https://api.meroalpha.com/v1/corporate-actions/bonus-shares?symbol=NABIL&limit=10&cursor=NABIL" \
  -H "Authorization: Bearer YOUR_API_KEY"
GET
/v1/corporate-actions/right-shares

PUBLIC
CURSOR
Rights issue announcements, premium costs, and subscription dates using signed opaque cursor paging.

Cursor pagination: Keep the meta.next_cursor token returned from a response and send it unchanged as cursor query parameter to fetch the next page.
PARAMETERS

symbol
REQUIRED
Symbol · symbol
Default: NABIL
limit
OPTIONAL
Limit · limit
Default: 10
cursor
OPTIONAL
Signed cursor · cursor

curl -X GET "https://api.meroalpha.com/v1/corporate-actions/right-shares?symbol=NABIL&limit=10&cursor=NABIL" \
  -H "Authorization: Bearer YOUR_API_KEY"
MUTUAL FUNDS
GET
/v1/mutual-funds/nav

PUBLIC
CURSOR
Weekly Net Asset Values of listed mutual funds with signed opaque cursor paging.

Cursor pagination: Keep the meta.next_cursor token returned from a response and send it unchanged as cursor query parameter to fetch the next page.
PARAMETERS

symbol
REQUIRED
Symbol · symbol
Default: NIBLSF
period
OPTIONAL
Period · period
Default: single
date
OPTIONAL
Date · date
Default: today
from
OPTIONAL
From · from
Default: today-30
to
OPTIONAL
To · to
Default: today
limit
OPTIONAL
Limit · limit
Default: 10
cursor
OPTIONAL
Signed cursor · cursor

curl -X GET "https://api.meroalpha.com/v1/mutual-funds/nav?symbol=NIBLSF&period=single&date=2026-06-15&from=2026-05-16&to=2026-06-15&limit=10&cursor=NABIL" \
  -H "Authorization: Bearer YOUR_API_KEY"
NEWS
GET
/v1/news

PUBLIC
CURSOR
Company-specific announcements and structural market updates with signed opaque cursor paging.

Cursor pagination: Keep the meta.next_cursor token returned from a response and send it unchanged as cursor query parameter to fetch the next page.
PARAMETERS

symbol
REQUIRED
Symbol · symbol
Default: NABIL
limit
OPTIONAL
Limit · limit
Default: 10
cursor
OPTIONAL
Signed cursor · cursor

curl -X GET "https://api.meroalpha.com/v1/news?symbol=NABIL&limit=10&cursor=NABIL" \
  -H "Authorization: Bearer YOUR_API_KEY"
INTERNAL
GET
/v1/api-keys

USER_SESSION
CURSOR
List API keys owned by the active user with signed opaque cursor paging.

Cursor pagination: Keep the meta.next_cursor token returned from a response and send it unchanged as cursor query parameter to fetch the next page.
PARAMETERS

limit
OPTIONAL
Limit · limit
Default: 10
cursor
OPTIONAL
Signed cursor · cursor

curl -X GET "https://api.meroalpha.com/v1/api-keys?limit=10&cursor=NABIL" \
  -H "Authorization: Bearer YOUR_API_KEY"
POST
/v1/api-keys

USER_SESSION
NO CURSOR
Create an API key and show plaintext once.


curl -X POST "https://api.meroalpha.com/v1/api-keys" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
  "name": "Production key"
}'
GET
/v1/auth/check

API_KEY
NO CURSOR
Validate a Bearer API key.


curl -X GET "https://api.meroalpha.com/v1/auth/check" \
  -H "Authorization: Bearer YOUR_API_KEY"
GET
/v1/api-keys/:id/usage

USER_SESSION
CURSOR
Usage rows for one API key with signed opaque cursor paging.

Cursor pagination: Keep the meta.next_cursor token returned from a response and send it unchanged as cursor query parameter to fetch the next page.
PARAMETERS

id
REQUIRED
API key ID · text
Default: key_123
limit
OPTIONAL
Limit · limit
Default: 10
cursor
OPTIONAL
Signed cursor · cursor

curl -X GET "https://api.meroalpha.com/v1/api-keys/:id/usage?id=key_123&limit=10&cursor=NABIL" \
  -H "Authorization: Bearer YOUR_API_KEY"
POST
/v1/api-keys/:id/revoke

USER_SESSION
NO CURSOR
Revoke an API key owned by the active user.

PARAMETERS

id
REQUIRED
API key ID · text
Default: key_123

curl -X POST "https://api.meroalpha.com/v1/api-keys/:id/revoke?id=key_123" \
  -H "Authorization: Bearer YOUR_API_KEY"
GET
/v1/dashboard/overview

USER_SESSION
NO CURSOR
Aggregated client usage stats and streak tracking.


curl -X GET "https://api.meroalpha.com/v1/dashboard/overview" \
  -H "Authorization: Bearer YOUR_API_KEY"
GET
/v1/usage/summary

USER_SESSION
NO CURSOR
Request method and response code breakdown metrics.


curl -X GET "https://api.meroalpha.com/v1/usage/summary" \
  -H "Authorization: Bearer YOUR_API_KEY"
Admin only
Operational admin endpoints
These routes are shown only after the backend session check confirms an admin user. They still require backend admin authorization on every request.
GET
/admin/api/historic/imports
List historic CSV import batches
POST
/admin/api/historic/imports
Import historic daily-price CSV content
{
  "file_name": "2024_03_04.csv",
  "content": "Symbol,Open,High,Low,Close,Vol,Turnover,Trans.\nNABIL,850,875,840,860,120000,103200000,1420\n"
}
POST
/admin/api/companies/sync
Synchronize company directory from source collectors
POST
/admin/api/news/sync
Synchronize market news and announcements
POST
/admin/api/floorsheet/sync
Synchronize floorsheet trades for a trade date
{
  "date": "2026-06-01"
}
POST
/admin/api/market/sync
Synchronize market status and index snapshots
POST
/admin/api/corporate-actions/sync
Synchronize dividends, AGMs, and security actions
{
  "symbol": "NABIL"
}
POST
/admin/api/financials/sync
Synchronize financial report metadata
{
  "symbol": "NABIL"
}
POST
/admin/api/financials/statements/sync
Parse financial statement line items
{
  "symbol": "NABIL"
}
POST
/admin/api/mutual-funds/nav/sync
Mutual fund NAV sync status endpoint
