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

pub struct MeroAlphaClient {
    base_url: String,
    api_key: String,
}

impl MeroAlphaClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            base_url: "http://localhost:3010".to_string(),
            api_key: api_key.into(),
        }
    }

    pub fn latest_price(&self, symbol: &str) -> Result<f64, MeroAlphaApiError> {
        let symbol = symbol.trim().to_ascii_uppercase();
        match self.get(&daily_price_path(&symbol)) {
            Ok(body) => latest_price_from_daily_response(&body),
            Err(MeroAlphaApiError::HttpStatus { status: 400, .. }) => {
                let body = self.get(&mutual_fund_nav_path(&symbol))?;
                latest_price_from_nav_response(&body)
            }
            Err(error) => Err(error),
        }
    }

    pub fn latest_prices(&self, symbols: &[String]) -> Vec<Result<PriceUpdate, PriceFailure>> {
        symbols
            .iter()
            .map(|symbol| {
                let symbol = symbol.trim().to_ascii_uppercase();
                self.latest_price(&symbol)
                    .map(|ltp| PriceUpdate {
                        symbol: symbol.clone(),
                        ltp,
                    })
                    .map_err(|error| PriceFailure { symbol, error })
            })
            .collect()
    }

    fn get(&self, path: &str) -> Result<String, MeroAlphaApiError> {
        let url = format!("{}{}", self.base_url, path);
        let route = path.split('?').next().unwrap_or(path).to_string();

        ureq::get(&url)
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .call()
            .map_err(|error| match error {
                ureq::Error::Status(status, _) => MeroAlphaApiError::HttpStatus {
                    route: route.clone(),
                    status,
                },
                other => MeroAlphaApiError::Http(other.to_string()),
            })?
            .into_string()
            .map_err(|error| MeroAlphaApiError::Http(error.to_string()))
    }
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

fn latest_number_from_response(
    body: &str,
    candidate_fields: &[&str],
) -> Result<f64, MeroAlphaApiError> {
    let json: serde_json::Value = serde_json::from_str(body)
        .map_err(|error| MeroAlphaApiError::InvalidJson(error.to_string()))?;

    let first_row = json
        .get("data")
        .and_then(|data| data.as_array())
        .and_then(|rows| rows.first())
        .or_else(|| json.as_array().and_then(|rows| rows.first()))
        .or_else(|| json.as_object().map(|_| &json))
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
    fn builds_refresh_paths_for_equity_and_mutual_fund_routes() {
        assert_eq!(
            daily_price_path("NABIL"),
            "/v1/prices/daily?symbol=NABIL&period=single&adjusted=false&limit=1"
        );
        assert_eq!(
            mutual_fund_nav_path("NICADF"),
            "/v1/mutual-funds/nav?symbol=NICADF&period=range&limit=1"
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

fn daily_price_path(symbol: &str) -> String {
    format!(
        "/v1/prices/daily?symbol={}&period=single&adjusted=false&limit=1",
        symbol.trim().to_ascii_uppercase()
    )
}

fn mutual_fund_nav_path(symbol: &str) -> String {
    format!(
        "/v1/mutual-funds/nav?symbol={}&period=range&limit=1",
        symbol.trim().to_ascii_uppercase()
    )
}
