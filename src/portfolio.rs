use rusqlite::{Connection, params};

#[derive(Debug, Clone, PartialEq)]
pub struct HoldingImport {
    pub symbol: String,
    pub quantity: f64,
    pub avg_cost: f64,
    pub ltp: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PortfolioPosition {
    pub symbol: String,
    pub quantity: f64,
    pub avg_cost: f64,
    pub ltp: f64,
    pub cost_value: f64,
    pub market_value: f64,
    pub unrealized_pl: f64,
    pub unrealized_pl_pct: f64,
    pub weight_pct: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PortfolioSnapshot {
    pub positions: Vec<PortfolioPosition>,
    pub total_cost: f64,
    pub total_value: f64,
    pub unrealized_pl: f64,
    pub unrealized_pl_pct: f64,
    pub top_holding_symbol: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSettings {
    pub profile_name: String,
    pub meroalpha_api_key: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            profile_name: "NEPSE Investor".to_string(),
            meroalpha_api_key: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortfolioImportError {
    EmptyCsv,
    MissingColumn(&'static str),
    InvalidColumnCount {
        line: usize,
        expected: usize,
        actual: usize,
    },
    InvalidNumber {
        line: usize,
        column: &'static str,
        value: String,
    },
    Storage(String),
}

pub trait PortfolioRepository {
    fn replace_holdings(&mut self, holdings: Vec<HoldingImport>);
    fn load_snapshot(&self) -> PortfolioSnapshot;
}

#[derive(Debug, Default)]
pub struct InMemoryPortfolioRepository {
    holdings: Vec<HoldingImport>,
}

impl InMemoryPortfolioRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl PortfolioRepository for InMemoryPortfolioRepository {
    fn replace_holdings(&mut self, holdings: Vec<HoldingImport>) {
        self.holdings = holdings;
    }

    fn load_snapshot(&self) -> PortfolioSnapshot {
        portfolio_snapshot(&self.holdings)
    }
}

pub struct SqlitePortfolioRepository {
    connection: Connection,
}

impl SqlitePortfolioRepository {
    pub fn open_in_memory() -> Result<Self, PortfolioImportError> {
        Self::from_connection(Connection::open_in_memory().map_storage_error()?)
    }

    pub fn from_connection(connection: Connection) -> Result<Self, PortfolioImportError> {
        connection
            .execute_batch(
                "
                CREATE TABLE IF NOT EXISTS holdings (
                    symbol TEXT PRIMARY KEY NOT NULL,
                    quantity REAL NOT NULL,
                    avg_cost REAL NOT NULL,
                    ltp REAL NOT NULL
                );

                CREATE TABLE IF NOT EXISTS app_settings (
                    id INTEGER PRIMARY KEY CHECK (id = 1),
                    profile_name TEXT NOT NULL,
                    meroalpha_api_key TEXT
                );
                ",
            )
            .map_storage_error()?;

        Ok(Self { connection })
    }

    pub fn load_settings(&self) -> AppSettings {
        let result = self.connection.query_row(
            "
            SELECT profile_name, meroalpha_api_key
            FROM app_settings
            WHERE id = 1
            ",
            [],
            |row| {
                let profile_name: String = row.get(0)?;
                let api_key: Option<String> = row.get(1)?;
                Ok(AppSettings {
                    profile_name,
                    meroalpha_api_key: normalize_api_key(api_key.as_deref()),
                })
            },
        );

        match result {
            Ok(settings) => settings,
            Err(rusqlite::Error::QueryReturnedNoRows) => AppSettings::default(),
            Err(error) => panic!("load app settings from sqlite: {error}"),
        }
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<(), PortfolioImportError> {
        let profile_name = normalize_profile_name(&settings.profile_name);
        let api_key = normalize_api_key(settings.meroalpha_api_key.as_deref());

        self.connection
            .execute(
                "
                INSERT INTO app_settings (id, profile_name, meroalpha_api_key)
                VALUES (1, ?1, ?2)
                ON CONFLICT(id) DO UPDATE SET
                    profile_name = excluded.profile_name,
                    meroalpha_api_key = excluded.meroalpha_api_key
                ",
                params![profile_name, api_key],
            )
            .map_storage_error()?;

        Ok(())
    }

    pub fn update_ltp(&self, symbol: &str, ltp: f64) -> Result<(), PortfolioImportError> {
        self.connection
            .execute(
                "
                UPDATE holdings
                SET ltp = ?1
                WHERE symbol = ?2
                ",
                params![ltp, symbol.trim().to_ascii_uppercase()],
            )
            .map_storage_error()?;

        Ok(())
    }

    fn load_holdings(&self) -> Result<Vec<HoldingImport>, PortfolioImportError> {
        let mut statement = self
            .connection
            .prepare(
                "
                SELECT symbol, quantity, avg_cost, ltp
                FROM holdings
                ORDER BY symbol
                ",
            )
            .map_storage_error()?;

        let rows = statement
            .query_map([], |row| {
                Ok(HoldingImport {
                    symbol: row.get(0)?,
                    quantity: row.get(1)?,
                    avg_cost: row.get(2)?,
                    ltp: row.get(3)?,
                })
            })
            .map_storage_error()?;

        let mut holdings = Vec::new();
        for row in rows {
            holdings.push(row.map_storage_error()?);
        }
        Ok(holdings)
    }
}

impl PortfolioRepository for SqlitePortfolioRepository {
    fn replace_holdings(&mut self, holdings: Vec<HoldingImport>) {
        let transaction = self
            .connection
            .transaction()
            .expect("open holdings transaction");
        transaction
            .execute("DELETE FROM holdings", [])
            .expect("clear holdings table");

        for holding in holdings {
            transaction
                .execute(
                    "
                    INSERT INTO holdings (symbol, quantity, avg_cost, ltp)
                    VALUES (?1, ?2, ?3, ?4)
                    ",
                    params![
                        holding.symbol,
                        holding.quantity,
                        holding.avg_cost,
                        holding.ltp
                    ],
                )
                .expect("insert holding");
        }

        transaction.commit().expect("commit holdings import");
    }

    fn load_snapshot(&self) -> PortfolioSnapshot {
        let holdings = self.load_holdings().expect("load holdings from sqlite");
        portfolio_snapshot(&holdings)
    }
}

trait SqliteResultExt<T> {
    fn map_storage_error(self) -> Result<T, PortfolioImportError>;
}

impl<T> SqliteResultExt<T> for rusqlite::Result<T> {
    fn map_storage_error(self) -> Result<T, PortfolioImportError> {
        self.map_err(|error| PortfolioImportError::Storage(error.to_string()))
    }
}

pub fn parse_holdings_csv(input: &str) -> Result<Vec<HoldingImport>, PortfolioImportError> {
    let mut lines = input.lines().filter(|line| !line.trim().is_empty());
    let header = lines.next().ok_or(PortfolioImportError::EmptyCsv)?;
    let headers = split_csv_line(header);
    let symbol_ix = find_column(&headers, "symbol", &["scrip"])?;
    let quantity_ix = find_column(&headers, "quantity", &["current balance"])?;
    let avg_cost_ix = find_column(&headers, "avg_cost", &["last closing price"])?;
    let ltp_ix = find_column(&headers, "ltp", &["last transaction price (ltp)"])?;

    let mut holdings = Vec::new();
    for (offset, line) in lines.enumerate() {
        let line_no = offset + 2;
        let values = split_csv_line(line);
        if values.len() != headers.len() {
            return Err(PortfolioImportError::InvalidColumnCount {
                line: line_no,
                expected: headers.len(),
                actual: values.len(),
            });
        }

        let symbol = values[symbol_ix].trim().to_ascii_uppercase();
        if symbol.is_empty() || symbol == "TOTAL :" {
            continue;
        }

        holdings.push(HoldingImport {
            symbol,
            quantity: parse_number(&values[quantity_ix], line_no, "quantity")?,
            avg_cost: parse_number(&values[avg_cost_ix], line_no, "avg_cost")?,
            ltp: parse_number(&values[ltp_ix], line_no, "ltp")?,
        });
    }

    Ok(holdings)
}

pub fn portfolio_snapshot(holdings: &[HoldingImport]) -> PortfolioSnapshot {
    let total_value: f64 = holdings
        .iter()
        .map(|holding| holding.quantity * holding.ltp)
        .sum();
    let total_cost: f64 = holdings
        .iter()
        .map(|holding| holding.quantity * holding.avg_cost)
        .sum();

    let mut positions: Vec<PortfolioPosition> = holdings
        .iter()
        .map(|holding| {
            let cost_value = holding.quantity * holding.avg_cost;
            let market_value = holding.quantity * holding.ltp;
            let unrealized_pl = market_value - cost_value;
            PortfolioPosition {
                symbol: holding.symbol.clone(),
                quantity: holding.quantity,
                avg_cost: holding.avg_cost,
                ltp: holding.ltp,
                cost_value,
                market_value,
                unrealized_pl,
                unrealized_pl_pct: percent(unrealized_pl, cost_value),
                weight_pct: percent(market_value, total_value),
            }
        })
        .collect();

    positions.sort_by(|a, b| {
        b.market_value
            .partial_cmp(&a.market_value)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.symbol.cmp(&b.symbol))
    });

    PortfolioSnapshot {
        top_holding_symbol: positions.first().map(|position| position.symbol.clone()),
        positions,
        total_cost,
        total_value,
        unrealized_pl: total_value - total_cost,
        unrealized_pl_pct: percent(total_value - total_cost, total_cost),
    }
}

pub fn local_ltp_for_unavailable_price(current_ltp: f64) -> f64 {
    if current_ltp > 0.0 { current_ltp } else { 1.0 }
}

fn split_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' if in_quotes && chars.peek() == Some(&'"') => {
                field.push('"');
                chars.next();
            }
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                fields.push(field.trim().to_string());
                field.clear();
            }
            _ => field.push(ch),
        }
    }

    fields.push(field.trim().to_string());
    fields
}

fn find_column(
    headers: &[String],
    name: &'static str,
    aliases: &[&str],
) -> Result<usize, PortfolioImportError> {
    headers
        .iter()
        .position(|header| {
            header.eq_ignore_ascii_case(name)
                || aliases
                    .iter()
                    .any(|alias| header.eq_ignore_ascii_case(alias))
        })
        .ok_or(PortfolioImportError::MissingColumn(name))
}

fn parse_number(
    value: &str,
    line: usize,
    column: &'static str,
) -> Result<f64, PortfolioImportError> {
    value
        .trim()
        .replace('_', "")
        .parse::<f64>()
        .map_err(|_| PortfolioImportError::InvalidNumber {
            line,
            column,
            value: value.to_string(),
        })
}

fn percent(numerator: f64, denominator: f64) -> f64 {
    if denominator.abs() < f64::EPSILON {
        0.0
    } else {
        numerator / denominator * 100.0
    }
}

fn normalize_profile_name(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        AppSettings::default().profile_name
    } else {
        trimmed.to_string()
    }
}

fn normalize_api_key(api_key: Option<&str>) -> Option<String> {
    api_key
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_holding_csv_with_required_columns() {
        let csv =
            "symbol,quantity,avg_cost,ltp\nnabil,2450,450.20,562.10\nGBIME,3100,190.50,215.00\n";

        let holdings = parse_holdings_csv(csv).expect("valid holdings CSV");

        assert_eq!(
            holdings,
            vec![
                HoldingImport {
                    symbol: "NABIL".to_string(),
                    quantity: 2450.0,
                    avg_cost: 450.20,
                    ltp: 562.10,
                },
                HoldingImport {
                    symbol: "GBIME".to_string(),
                    quantity: 3100.0,
                    avg_cost: 190.50,
                    ltp: 215.00,
                },
            ]
        );
    }

    #[test]
    fn parses_meroshare_current_balance_export() {
        let csv = "\"S.N\",\"Scrip\",\"Current Balance\",\"Last Closing Price\",\"Value as of Last Closing Price\",\"Last Transaction Price (LTP)\",\"Value as of LTP\"\n\"1\",\"ALBSL\",\"12.0\",\"1097.6\",\"13171.20\",\"1097.8\",\"13173.60\"\n\"Total :\",\" \",\" \",\" \",\"47739.14\",\" \",\"47696.84\"\n";

        let holdings = parse_holdings_csv(csv).expect("valid meroshare holdings CSV");

        assert_eq!(
            holdings,
            vec![HoldingImport {
                symbol: "ALBSL".to_string(),
                quantity: 12.0,
                avg_cost: 1097.6,
                ltp: 1097.8,
            }]
        );
    }

    #[test]
    fn computes_portfolio_snapshot_from_holdings() {
        let holdings = vec![
            HoldingImport {
                symbol: "NABIL".to_string(),
                quantity: 2450.0,
                avg_cost: 450.20,
                ltp: 562.10,
            },
            HoldingImport {
                symbol: "GBIME".to_string(),
                quantity: 3100.0,
                avg_cost: 190.50,
                ltp: 215.00,
            },
        ];

        let snapshot = portfolio_snapshot(&holdings);

        assert_eq!(snapshot.positions.len(), 2);
        assert_eq!(snapshot.top_holding_symbol, Some("NABIL".to_string()));
        assert_close(snapshot.total_cost, 1_693_540.0);
        assert_close(snapshot.total_value, 2_043_645.0);
        assert_close(snapshot.unrealized_pl, 350_105.0);
        assert_close(snapshot.unrealized_pl_pct, 20.67);
        assert_close(snapshot.positions[0].weight_pct, 67.39);
        assert_close(snapshot.positions[1].unrealized_pl_pct, 12.86);
    }

    #[test]
    fn rejects_csv_without_required_columns() {
        let error = parse_holdings_csv("ticker,quantity,avg_cost,ltp\nNABIL,1,2,3\n")
            .expect_err("symbol column is required");

        assert_eq!(error, PortfolioImportError::MissingColumn("symbol"));
    }

    #[test]
    fn repository_replaces_holdings_and_loads_latest_snapshot() {
        let mut repository = InMemoryPortfolioRepository::new();
        repository.replace_holdings(vec![HoldingImport {
            symbol: "SHIVM".to_string(),
            quantity: 1200.0,
            avg_cost: 540.0,
            ltp: 490.5,
        }]);

        let snapshot = repository.load_snapshot();

        assert_eq!(snapshot.top_holding_symbol, Some("SHIVM".to_string()));
        assert_close(snapshot.total_value, 588_600.0);
        assert_close(snapshot.unrealized_pl, -59_400.0);
    }

    #[test]
    fn sqlite_repository_persists_latest_holdings_snapshot() {
        let mut repository =
            SqlitePortfolioRepository::open_in_memory().expect("sqlite repository opens");

        repository.replace_holdings(vec![HoldingImport {
            symbol: "ALBSL".to_string(),
            quantity: 12.0,
            avg_cost: 1097.6,
            ltp: 1097.8,
        }]);

        let snapshot = repository.load_snapshot();

        assert_eq!(snapshot.top_holding_symbol, Some("ALBSL".to_string()));
        assert_close(snapshot.total_value, 13_173.60);
    }

    #[test]
    fn sqlite_repository_updates_ltp_without_touching_quantity_or_cost() {
        let mut repository =
            SqlitePortfolioRepository::open_in_memory().expect("sqlite repository opens");
        repository.replace_holdings(vec![HoldingImport {
            symbol: "NABIL".to_string(),
            quantity: 10.0,
            avg_cost: 500.0,
            ltp: 510.0,
        }]);

        repository
            .update_ltp("NABIL", 620.5)
            .expect("ltp update succeeds");

        let snapshot = repository.load_snapshot();
        let position = &snapshot.positions[0];
        assert_eq!(position.symbol, "NABIL");
        assert_close(position.quantity, 10.0);
        assert_close(position.avg_cost, 500.0);
        assert_close(position.ltp, 620.5);
        assert_close(position.market_value, 6_205.0);
    }

    #[test]
    fn unavailable_upstream_prices_keep_or_fill_small_local_ltp() {
        assert_close(local_ltp_for_unavailable_price(10.06), 10.06);
        assert_close(local_ltp_for_unavailable_price(0.0), 1.0);
        assert_close(local_ltp_for_unavailable_price(-4.0), 1.0);
    }

    #[test]
    fn sqlite_repository_persists_local_profile_and_api_key() {
        let repository =
            SqlitePortfolioRepository::open_in_memory().expect("sqlite repository opens");

        assert_eq!(repository.load_settings(), AppSettings::default());

        repository
            .save_settings(&AppSettings {
                profile_name: "Asha Portfolio".to_string(),
                meroalpha_api_key: Some("ma_test_key".to_string()),
            })
            .expect("settings save");

        assert_eq!(
            repository.load_settings(),
            AppSettings {
                profile_name: "Asha Portfolio".to_string(),
                meroalpha_api_key: Some("ma_test_key".to_string()),
            }
        );
    }

    fn assert_close(actual: f64, expected: f64) {
        let rounded_actual = (actual * 100.0).round() / 100.0;
        assert_eq!(rounded_actual, expected);
    }
}
