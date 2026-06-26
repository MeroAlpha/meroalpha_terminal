use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq)]
pub enum MeroAlphaApiError {
    MissingPrice,
    InvalidJson(String),
    HttpStatus { route: String, status: u16 },
    Http(String),
}

impl MeroAlphaApiError {
    pub fn is_unavailable_price(&self) -> bool {
        match self {
            Self::MissingPrice => true,
            Self::HttpStatus { route, status } => {
                matches!(status, 400 | 404) || (*status == 403 && route == "/v1/mutual-funds/nav")
            }
            Self::InvalidJson(_) | Self::Http(_) => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PriceUpdate {
    pub symbol: String,
    pub ltp: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PriceFailure {
    pub symbol: String,
    pub error: MeroAlphaApiError,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MarketIndexUpdate {
    pub label: String,
    pub value: f64,
    pub change: f64,
    pub change_pct: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MarketMoverUpdate {
    pub symbol: String,
    pub ltp: f64,
    pub change: f64,
    pub change_pct: f64,
    pub volume: f64,
    pub turnover: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MarketMoverBuckets {
    pub gainers: Vec<MarketMoverUpdate>,
    pub losers: Vec<MarketMoverUpdate>,
    pub turnover: Vec<MarketMoverUpdate>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MarketStatusUpdate {
    pub status: String,
    pub is_open: bool,
    pub as_of: Option<String>,
    pub last_traded_date: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverviewMarketData {
    pub market_status: MarketStatusUpdate,
    pub indices: Vec<MarketIndexUpdate>,
    pub gainers: Vec<MarketMoverUpdate>,
    pub losers: Vec<MarketMoverUpdate>,
    pub turnover: Vec<MarketMoverUpdate>,
}

pub struct MeroAlphaClient {
    base_url: String,
    api_key: String,
}

impl MeroAlphaClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            base_url: "https://www.meroalpha.com".to_string(),
            api_key: api_key.into(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn latest_price(&self, symbol: &str) -> Result<f64, MeroAlphaApiError> {
        let symbol = symbol.trim().to_ascii_uppercase();
        let market_status = self.market_status()?;
        self.latest_price_on_date(&symbol, &market_status.last_traded_date)
    }

    fn latest_price_on_date(&self, symbol: &str, date: &str) -> Result<f64, MeroAlphaApiError> {
        match self.get(&daily_price_path(symbol, date)) {
            Ok(body) => latest_price_from_daily_response(&body),
            Err(MeroAlphaApiError::HttpStatus { status: 400, .. }) => {
                let body = self.get(&mutual_fund_nav_path(symbol))?;
                latest_price_from_nav_response(&body)
            }
            Err(error) => Err(error),
        }
    }

    pub fn latest_prices(&self, symbols: &[String]) -> Vec<Result<PriceUpdate, PriceFailure>> {
        eprintln!(
            "[meroalpha-api] latest-prices start base_url={} symbols={:?}",
            self.base_url, symbols
        );
        let market_status = match self.market_status() {
            Ok(market_status) => market_status,
            Err(error) => {
                return symbols
                    .iter()
                    .map(|symbol| {
                        Err(PriceFailure {
                            symbol: symbol.trim().to_ascii_uppercase(),
                            error: error.clone(),
                        })
                    })
                    .collect();
            }
        };
        symbols
            .iter()
            .map(|symbol| {
                let symbol = symbol.trim().to_ascii_uppercase();
                self.latest_price_on_date(&symbol, &market_status.last_traded_date)
                    .map(|ltp| PriceUpdate {
                        symbol: symbol.clone(),
                        ltp,
                    })
                    .map_err(|error| PriceFailure { symbol, error })
            })
            .collect()
    }

    pub fn overview_market_data(&self) -> Result<OverviewMarketData, MeroAlphaApiError> {
        eprintln!("[meroalpha-api] overview start base_url={}", self.base_url);
        let market_status = self.market_status()?;
        eprintln!(
            "[meroalpha-api] overview market status last_traded_date={} status={} is_open={}",
            market_status.last_traded_date, market_status.status, market_status.is_open
        );
        let indices = self.market_indices()?;
        let movers = self.market_movers(&market_status.last_traded_date)?;

        Ok(OverviewMarketData {
            market_status,
            indices,
            gainers: movers.gainers,
            losers: movers.losers,
            turnover: movers.turnover,
        })
    }

    fn market_status(&self) -> Result<MarketStatusUpdate, MeroAlphaApiError> {
        let body = self.get("/v1/market/status")?;
        market_status_from_response(&body)
    }

    fn market_indices(&self) -> Result<Vec<MarketIndexUpdate>, MeroAlphaApiError> {
        eprintln!("[meroalpha-api] overview indices primary path=/v1/indices?limit=10");
        let primary_body = self.get("/v1/indices?limit=10")?;
        let mut indices = market_indices_from_response(&primary_body)?;
        eprintln!(
            "[meroalpha-api] overview indices primary parsed count={}",
            indices.len()
        );
        if indices.is_empty() {
            eprintln!("[meroalpha-api] overview indices fallback path=/v1/sub-indices?limit=10");
            let fallback_body = self.get("/v1/sub-indices?limit=10")?;
            indices = market_indices_from_response(&fallback_body)?;
            eprintln!(
                "[meroalpha-api] overview indices fallback parsed count={}",
                indices.len()
            );
        }
        indices.truncate(4);
        Ok(indices)
    }

    fn market_movers(&self, date: &str) -> Result<MarketMoverBuckets, MeroAlphaApiError> {
        let body = self.get(&market_movers_path(date, 10))?;
        market_movers_from_response(&body)
    }

    fn get(&self, path: &str) -> Result<String, MeroAlphaApiError> {
        let url = format!("{}{}", self.base_url, path);
        let route = path.split('?').next().unwrap_or(path).to_string();
        let request_id = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let started_at = Instant::now();

        eprintln!(
            "[meroalpha-api] GET id={} base_url={} path={} route={} url={}",
            request_id, self.base_url, path, route, url
        );
        let response = ureq::get(&url)
            .timeout(Duration::from_secs(12))
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .call()
            .map_err(|error| match error {
                ureq::Error::Status(status, response) => {
                    let body = response
                        .into_string()
                        .unwrap_or_else(|error| format!("<failed to read body: {error}>"));
                    eprintln!(
                        "[meroalpha-api] HTTP status id={} elapsed_ms={} base_url={} route={} status={} body={}",
                        request_id,
                        started_at.elapsed().as_millis(),
                        self.base_url,
                        route,
                        status,
                        debug_body_snippet(&body)
                    );
                    MeroAlphaApiError::HttpStatus {
                        route: route.clone(),
                        status,
                    }
                }
                other => {
                    eprintln!(
                        "[meroalpha-api] HTTP error id={} elapsed_ms={} base_url={} route={} error={}",
                        request_id,
                        started_at.elapsed().as_millis(),
                        self.base_url,
                        route,
                        other
                    );
                    MeroAlphaApiError::Http(other.to_string())
                }
            })?;

        let body = response.into_string().map_err(|error| {
            eprintln!(
                "[meroalpha-api] body error id={} elapsed_ms={} base_url={} route={} error={}",
                request_id,
                started_at.elapsed().as_millis(),
                self.base_url,
                route,
                error
            );
            MeroAlphaApiError::Http(error.to_string())
        })?;

        eprintln!(
            "[meroalpha-api] OK id={} elapsed_ms={} base_url={} route={} bytes={} body={}",
            request_id,
            started_at.elapsed().as_millis(),
            self.base_url,
            route,
            body.len(),
            debug_body_snippet(&body)
        );

        Ok(body)
    }
}

fn debug_body_snippet(body: &str) -> String {
    let mut snippet = body.split_whitespace().collect::<Vec<_>>().join(" ");
    if snippet.chars().count() > 500 {
        snippet = snippet.chars().take(500).collect::<String>();
        snippet.push_str("...");
    }
    snippet
}

pub fn latest_price_from_daily_response(body: &str) -> Result<f64, MeroAlphaApiError> {
    latest_number_from_response(
        body,
        &[
            "ltp",
            "close",
            "closing_price",
            "last_traded_price",
            "last_transaction_price",
        ],
    )
}

pub fn latest_price_from_nav_response(body: &str) -> Result<f64, MeroAlphaApiError> {
    latest_number_from_response(
        body,
        &[
            "nav",
            "nav_per_unit",
            "net_asset_value",
            "latest_nav",
            "close",
            "ltp",
        ],
    )
}

pub fn market_status_from_response(body: &str) -> Result<MarketStatusUpdate, MeroAlphaApiError> {
    let json: serde_json::Value = serde_json::from_str(body)
        .map_err(|error| MeroAlphaApiError::InvalidJson(error.to_string()))?;
    let row = json
        .get("data")
        .filter(|data| data.is_object())
        .unwrap_or(&json);
    let last_traded_date =
        first_string(row, &["last_traded_date"]).ok_or(MeroAlphaApiError::MissingPrice)?;

    Ok(MarketStatusUpdate {
        status: first_string(row, &["status"]).unwrap_or_else(|| "UNKNOWN".to_string()),
        is_open: row
            .get("is_open")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        as_of: first_string(row, &["as_of"]),
        last_traded_date,
    })
}

pub fn market_indices_from_response(
    body: &str,
) -> Result<Vec<MarketIndexUpdate>, MeroAlphaApiError> {
    let json: serde_json::Value = serde_json::from_str(body)
        .map_err(|error| MeroAlphaApiError::InvalidJson(error.to_string()))?;
    let rows = response_rows(&json)?;

    Ok(rows
        .iter()
        .filter_map(|row| {
            let label = first_string(
                row,
                &[
                    "name",
                    "index_name",
                    "sector",
                    "sector_name",
                    "symbol",
                    "sub_index",
                ],
            )?;
            let value = first_number(
                row,
                &[
                    "value",
                    "current_value",
                    "index_value",
                    "close",
                    "ltp",
                    "last_value",
                ],
            )?;
            let change = first_number(
                row,
                &[
                    "change",
                    "change_value",
                    "point_change",
                    "absolute_change",
                    "net_change",
                ],
            )
            .unwrap_or(0.0);
            let change_pct = first_number(
                row,
                &[
                    "percent_change",
                    "change_percent",
                    "change_pct",
                    "pct_change",
                    "percentage_change",
                ],
            )
            .unwrap_or(0.0);

            Some(MarketIndexUpdate {
                label,
                value,
                change,
                change_pct,
            })
        })
        .collect())
}

pub fn market_movers_from_response(body: &str) -> Result<MarketMoverBuckets, MeroAlphaApiError> {
    let json: serde_json::Value = serde_json::from_str(body)
        .map_err(|error| MeroAlphaApiError::InvalidJson(error.to_string()))?;
    let data = json.get("data").unwrap_or(&json);

    Ok(MarketMoverBuckets {
        gainers: market_mover_bucket(data, "top_gainers")?,
        losers: market_mover_bucket(data, "top_losers")?,
        turnover: market_mover_bucket(data, "top_turnover")?,
    })
}

fn market_mover_bucket(
    data: &serde_json::Value,
    field: &str,
) -> Result<Vec<MarketMoverUpdate>, MeroAlphaApiError> {
    let Some(rows) = data.get(field).and_then(|rows| rows.as_array()) else {
        return Ok(Vec::new());
    };

    rows.iter()
        .map(|row| market_mover_from_row("", row))
        .collect()
}

fn market_mover_from_row(
    symbol: &str,
    row: &serde_json::Value,
) -> Result<MarketMoverUpdate, MeroAlphaApiError> {
    let ltp = first_number(
        row,
        &[
            "ltp",
            "close",
            "closing_price",
            "last_traded_price",
            "last_transaction_price",
        ],
    )
    .ok_or(MeroAlphaApiError::MissingPrice)?;
    let change = first_number(
        row,
        &[
            "change",
            "change_value",
            "price_change",
            "net_change",
            "absolute_change",
        ],
    )
    .or_else(|| {
        first_number(row, &["previous_close", "prev_close", "previous_ltp"]).map(|prev| ltp - prev)
    })
    .unwrap_or(0.0);
    let change_pct = first_number(
        row,
        &[
            "percent_change",
            "change_percent",
            "change_pct",
            "pct_change",
            "percentage_change",
        ],
    )
    .or_else(|| {
        first_number(row, &["previous_close", "prev_close", "previous_ltp"]).map(|prev| {
            if prev == 0.0 {
                0.0
            } else {
                (change / prev) * 100.0
            }
        })
    })
    .unwrap_or(0.0);
    let volume = first_number(row, &["volume", "vol", "traded_quantity"]).unwrap_or(0.0);
    let turnover = first_number(row, &["turnover", "amount", "traded_value"]).unwrap_or(0.0);

    Ok(MarketMoverUpdate {
        symbol: first_string(row, &["symbol"])
            .unwrap_or_else(|| symbol.trim().to_ascii_uppercase()),
        ltp,
        change,
        change_pct,
        volume,
        turnover,
    })
}

pub fn company_symbols_from_response(body: &str) -> Result<Vec<String>, MeroAlphaApiError> {
    let json: serde_json::Value = serde_json::from_str(body)
        .map_err(|error| MeroAlphaApiError::InvalidJson(error.to_string()))?;
    let rows = response_rows(&json)?;
    Ok(rows
        .iter()
        .filter_map(|row| first_string(row, &["symbol", "ticker", "scrip"]))
        .map(|symbol| symbol.trim().to_ascii_uppercase())
        .filter(|symbol| !symbol.is_empty())
        .collect())
}

fn latest_number_from_response(
    body: &str,
    candidate_fields: &[&str],
) -> Result<f64, MeroAlphaApiError> {
    let json: serde_json::Value = serde_json::from_str(body)
        .map_err(|error| MeroAlphaApiError::InvalidJson(error.to_string()))?;

    let first_row = response_rows(&json)?
        .first()
        .copied()
        .ok_or(MeroAlphaApiError::MissingPrice)?;

    for field in candidate_fields {
        if let Some(price) = first_row.get(field).and_then(json_number) {
            return Ok(price);
        }
    }

    Err(MeroAlphaApiError::MissingPrice)
}

fn json_number(value: &serde_json::Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_str().and_then(|text| text.parse::<f64>().ok()))
}

fn response_rows(json: &serde_json::Value) -> Result<Vec<&serde_json::Value>, MeroAlphaApiError> {
    if let Some(rows) = json.get("data").and_then(|data| data.as_array()) {
        Ok(rows.iter().collect())
    } else if let Some(rows) = json.as_array() {
        Ok(rows.iter().collect())
    } else if json.is_object() {
        Ok(vec![json])
    } else {
        Err(MeroAlphaApiError::MissingPrice)
    }
}

fn first_number(row: &serde_json::Value, fields: &[&str]) -> Option<f64> {
    fields
        .iter()
        .find_map(|field| row.get(field).and_then(json_number))
}

fn first_string(row: &serde_json::Value, fields: &[&str]) -> Option<String> {
    fields.iter().find_map(|field| {
        row.get(field).and_then(|value| {
            value
                .as_str()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
        })
    })
}

fn daily_price_path(symbol: &str, date: &str) -> String {
    format!(
        "/v1/prices/daily?symbol={}&date={}&adjusted=false&limit=1",
        symbol.trim().to_ascii_uppercase(),
        date.trim()
    )
}

fn market_movers_path(date: &str, limit: usize) -> String {
    format!("/v1/market/movers?date={}&limit={}", date.trim(), limit)
}

fn mutual_fund_nav_path(symbol: &str) -> String {
    format!(
        "/v1/mutual-funds/nav?symbol={}&period=range&limit=1",
        symbol.trim().to_ascii_uppercase()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_latest_close_from_daily_price_response() {
        let body = r#"{
            "data": [
                { "symbol": "NABIL", "trade_date": "2026-06-15", "close": 612.4 }
            ],
            "meta": { "next_cursor": null }
        }"#;

        assert_eq!(latest_price_from_daily_response(body), Ok(612.4));
    }

    #[test]
    fn extracts_string_ltp_from_daily_price_response() {
        let body = r#"{ "data": [{ "symbol": "NABIL", "ltp": "612.40" }] }"#;

        assert_eq!(latest_price_from_daily_response(body), Ok(612.4));
    }

    #[test]
    fn extracts_nav_from_mutual_fund_response() {
        let body = r#"{ "data": [{ "symbol": "NICADF", "nav": "10.07" }] }"#;

        assert_eq!(latest_price_from_nav_response(body), Ok(10.07));
    }

    #[test]
    fn extracts_market_indices_from_api_response() {
        let body = r#"{
            "data": [
                { "name": "NEPSE", "value": 2145.67, "change": 12.45, "percent_change": 0.58 },
                { "index_name": "Banking", "current_value": "1204.30", "change_value": "-4.20", "change_percent": "-0.35" }
            ]
        }"#;

        assert_eq!(
            market_indices_from_response(body),
            Ok(vec![
                MarketIndexUpdate {
                    label: "NEPSE".to_string(),
                    value: 2145.67,
                    change: 12.45,
                    change_pct: 0.58,
                },
                MarketIndexUpdate {
                    label: "Banking".to_string(),
                    value: 1204.30,
                    change: -4.20,
                    change_pct: -0.35,
                },
            ])
        );
    }

    #[test]
    fn extracts_market_mover_buckets_from_market_movers_response() {
        let body = r#"{
            "data": {
                "top_gainers": [
                    { "symbol": "TPKHL", "close": 456.2, "change": 59.5, "percent_change": 14.99874, "volume": 240, "turnover": 108332 }
                ],
                "top_losers": [
                    { "symbol": "SOPL", "close": 988.4, "change": -52.6, "percent_change": -5.052834, "volume": 172660, "turnover": 172271880.7 }
                ],
                "top_turnover": [
                    { "symbol": "AKJCL", "close": 371.5, "change": 3.0, "percent_change": 0.814111, "volume": 815172, "turnover": 296233269.3 }
                ]
            }
        }"#;

        assert_eq!(
            market_movers_from_response(body),
            Ok(MarketMoverBuckets {
                gainers: vec![MarketMoverUpdate {
                    symbol: "TPKHL".to_string(),
                    ltp: 456.2,
                    change: 59.5,
                    change_pct: 14.99874,
                    volume: 240.0,
                    turnover: 108332.0,
                }],
                losers: vec![MarketMoverUpdate {
                    symbol: "SOPL".to_string(),
                    ltp: 988.4,
                    change: -52.6,
                    change_pct: -5.052834,
                    volume: 172660.0,
                    turnover: 172271880.7,
                }],
                turnover: vec![MarketMoverUpdate {
                    symbol: "AKJCL".to_string(),
                    ltp: 371.5,
                    change: 3.0,
                    change_pct: 0.814111,
                    volume: 815172.0,
                    turnover: 296233269.3,
                }],
            })
        );
    }

    #[test]
    fn extracts_last_traded_date_from_market_status_response() {
        let body = r#"{
            "data": {
                "status": "CLOSE",
                "is_open": false,
                "as_of": "2026-06-25T09:15:00Z",
                "last_traded_date": "2026-06-25"
            }
        }"#;

        assert_eq!(
            market_status_from_response(body),
            Ok(MarketStatusUpdate {
                status: "CLOSE".to_string(),
                is_open: false,
                as_of: Some("2026-06-25T09:15:00Z".to_string()),
                last_traded_date: "2026-06-25".to_string(),
            })
        );
    }

    #[test]
    fn rejects_blank_last_traded_date_from_market_status_response() {
        let body = r#"{
            "data": {
                "status": "CLOSE",
                "is_open": false,
                "as_of": "2026-06-25T09:15:00Z",
                "last_traded_date": "   "
            }
        }"#;

        assert_eq!(
            market_status_from_response(body),
            Err(MeroAlphaApiError::MissingPrice)
        );
    }

    #[test]
    fn builds_refresh_paths_for_equity_and_mutual_fund_routes() {
        assert_eq!(
            daily_price_path("NABIL", "2026-06-25"),
            "/v1/prices/daily?symbol=NABIL&date=2026-06-25&adjusted=false&limit=1"
        );
        assert_eq!(
            mutual_fund_nav_path("NICADF"),
            "/v1/mutual-funds/nav?symbol=NICADF&period=range&limit=1"
        );
    }

    #[test]
    fn builds_market_movers_path_from_trade_date() {
        assert_eq!(
            market_movers_path("2026-06-25", 10),
            "/v1/market/movers?date=2026-06-25&limit=10"
        );
    }

    #[test]
    fn classifies_missing_or_not_found_prices_as_unavailable() {
        assert!(MeroAlphaApiError::MissingPrice.is_unavailable_price());
        assert!(
            MeroAlphaApiError::HttpStatus {
                route: "/v1/mutual-funds/nav".to_string(),
                status: 404,
            }
            .is_unavailable_price()
        );
        assert!(
            MeroAlphaApiError::HttpStatus {
                route: "/v1/mutual-funds/nav".to_string(),
                status: 403,
            }
            .is_unavailable_price()
        );

        assert!(!MeroAlphaApiError::Http("connection refused".to_string()).is_unavailable_price());
        assert!(
            !MeroAlphaApiError::HttpStatus {
                route: "/v1/prices/daily".to_string(),
                status: 403,
            }
            .is_unavailable_price()
        );
        assert!(
            !MeroAlphaApiError::InvalidJson("expected value".to_string()).is_unavailable_price()
        );
    }
}
