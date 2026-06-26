use crate::{
    meroalpha_api::{MarketIndexUpdate, MarketMoverUpdate, MarketStatusUpdate, OverviewMarketData},
    portfolio::PortfolioSnapshot,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoverTab {
    Gainers,
    Losers,
    Turnover,
}

impl MoverTab {
    pub fn label(self) -> &'static str {
        match self {
            Self::Gainers => "Gainers",
            Self::Losers => "Losers",
            Self::Turnover => "Turnover",
        }
    }

    pub fn metric_label(self) -> &'static str {
        match self {
            Self::Turnover => "TURNOVER (NPR)",
            Self::Gainers | Self::Losers => "VOLUME",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Gainers => "Highest positive price moves",
            Self::Losers => "Largest negative price moves",
            Self::Turnover => "Highest traded value",
        }
    }

    pub fn empty_message(self) -> &'static str {
        match self {
            Self::Gainers => "No gainers returned.",
            Self::Losers => "No losers returned.",
            Self::Turnover => "No turnover leaders returned.",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinancialDirection {
    Up,
    Down,
    Flat,
}

impl FinancialDirection {
    fn from_signed(value: f64) -> Self {
        if display_zero(value) {
            Self::Flat
        } else if value > 0.0 {
            Self::Up
        } else {
            Self::Down
        }
    }

    fn from_change(change: f64, change_pct: f64) -> Self {
        match Self::from_signed(change) {
            Self::Flat => Self::from_signed(change_pct),
            direction => direction,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverviewSnapshot {
    pub market_pulse: Vec<MarketPulseCard>,
    pub portfolio: PortfolioPerformance,
    pub top_movers: Vec<MarketMover>,
    pub selected_mover_tab: MoverTab,
    pub mover_metric_label: &'static str,
    pub market_status: MarketStatus,
    pub last_updated: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MarketStatus {
    pub label: &'static str,
    pub live: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MarketPulseCard {
    pub label: String,
    pub value: String,
    pub change: String,
    pub direction: FinancialDirection,
    pub sparkline: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PortfolioPerformance {
    pub total_value_label: String,
    pub change_label: String,
    pub change_pct_label: String,
    pub risk_label: &'static str,
    pub risk_score_label: &'static str,
    pub direction: FinancialDirection,
    pub is_empty: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MarketMover {
    pub symbol: String,
    pub ltp: String,
    pub change: String,
    pub volume: String,
    pub direction: FinancialDirection,
}

impl OverviewSnapshot {
    pub fn from_portfolio(portfolio: Option<&PortfolioSnapshot>) -> Self {
        Self {
            market_pulse: Vec::new(),
            portfolio: PortfolioPerformance::from_portfolio(portfolio),
            top_movers: Vec::new(),
            selected_mover_tab: MoverTab::Gainers,
            mover_metric_label: MoverTab::Gainers.metric_label(),
            market_status: MarketStatus::waiting(),
            last_updated: "Waiting for API".to_string(),
        }
    }

    pub fn with_market_data(
        mut self,
        market_data: OverviewMarketData,
        selected_tab: MoverTab,
    ) -> Self {
        self.market_pulse = market_data
            .indices
            .iter()
            .map(MarketPulseCard::from_update)
            .collect();
        let movers = match selected_tab {
            MoverTab::Gainers => &market_data.gainers,
            MoverTab::Losers => &market_data.losers,
            MoverTab::Turnover => &market_data.turnover,
        };
        self.top_movers = movers
            .iter()
            .map(|mover| MarketMover::from_update(mover, selected_tab))
            .collect();
        self.selected_mover_tab = selected_tab;
        self.mover_metric_label = selected_tab.metric_label();
        self.market_status = MarketStatus::from_update(&market_data.market_status);
        self.last_updated = market_data.market_status.last_traded_date;
        self
    }

    pub fn with_selected_mover_tab(mut self, selected_tab: MoverTab) -> Self {
        self.selected_mover_tab = selected_tab;
        self.mover_metric_label = selected_tab.metric_label();
        self
    }

    pub fn with_market_error(mut self) -> Self {
        self.market_status = MarketStatus::api_error();
        self.last_updated = "Request failed".to_string();
        self
    }

    pub fn market_recency_label(&self) -> String {
        if self.market_status.is_error() || self.last_updated == "Waiting for API" {
            self.last_updated.clone()
        } else {
            format!("Last traded {}", self.last_updated)
        }
    }

    pub fn top_movers_empty_message(&self) -> &'static str {
        if self.market_status.is_error() {
            "Market data unavailable."
        } else if self.last_updated == "Waiting for API" {
            "No market API data loaded."
        } else {
            self.selected_mover_tab.empty_message()
        }
    }
}

impl MarketStatus {
    pub fn is_error(&self) -> bool {
        self.label == "API Error"
    }

    pub fn badge_label(&self) -> &'static str {
        match self.label {
            "Open" => "Market Open",
            "Closed" => "Market Closed",
            "API Error" => "API Error",
            _ => self.label,
        }
    }

    pub fn empty_title(&self) -> &'static str {
        if self.is_error() {
            "Market data unavailable"
        } else {
            "No market API data loaded"
        }
    }

    pub fn empty_detail(&self) -> &'static str {
        if self.is_error() {
            "Refresh again, or check your API key and connection."
        } else {
            "Set an API key in Profile & API, then refresh Overview."
        }
    }

    fn waiting() -> Self {
        Self {
            label: "Waiting",
            live: false,
        }
    }

    fn from_update(update: &MarketStatusUpdate) -> Self {
        Self {
            label: if update.is_open { "Open" } else { "Closed" },
            live: update.is_open,
        }
    }

    fn api_error() -> Self {
        Self {
            label: "API Error",
            live: false,
        }
    }
}

impl MarketPulseCard {
    fn from_update(update: &MarketIndexUpdate) -> Self {
        Self {
            label: update.label.clone(),
            value: format_number(update.value),
            change: format_change(update.change, update.change_pct),
            direction: FinancialDirection::from_change(update.change, update.change_pct),
            sparkline: sparkline_from_change(update.change_pct),
        }
    }
}

impl MarketMover {
    fn from_update(update: &MarketMoverUpdate, selected_tab: MoverTab) -> Self {
        let metric = if selected_tab == MoverTab::Turnover {
            update.turnover
        } else {
            update.volume
        };

        Self {
            symbol: update.symbol.clone(),
            ltp: format_number(update.ltp),
            change: format_change(update.change, update.change_pct),
            volume: format_whole_number(metric),
            direction: FinancialDirection::from_change(update.change, update.change_pct),
        }
    }
}

impl PortfolioPerformance {
    fn from_portfolio(portfolio: Option<&PortfolioSnapshot>) -> Self {
        match portfolio {
            Some(portfolio) => Self {
                total_value_label: format_npr(portfolio.total_value),
                change_label: format_signed_npr(portfolio.unrealized_pl),
                change_pct_label: format_signed_percent(portfolio.unrealized_pl_pct),
                risk_label: "Local Portfolio",
                risk_score_label: allocation_risk_label(portfolio.positions.len()),
                direction: FinancialDirection::from_signed(portfolio.unrealized_pl),
                is_empty: false,
            },
            None => Self {
                total_value_label: "No holdings".to_string(),
                change_label: "Portfolio inactive".to_string(),
                change_pct_label: String::new(),
                risk_label: "Awaiting holdings",
                risk_score_label: "0 positions",
                direction: FinancialDirection::Flat,
                is_empty: true,
            },
        }
    }
}

fn allocation_risk_label(position_count: usize) -> &'static str {
    if position_count >= 12 {
        "Diversified"
    } else if position_count >= 5 {
        "Moderate"
    } else {
        "Concentrated"
    }
}

fn format_npr(value: f64) -> String {
    format!("NPR {}", format_number(value))
}

fn format_signed_npr(value: f64) -> String {
    if display_zero(value) {
        format_npr(0.0)
    } else if value > 0.0 {
        format!("+{}", format_npr(value))
    } else {
        format!("-{}", format_npr(value.abs()))
    }
}

fn format_signed_percent(value: f64) -> String {
    if display_zero(value) {
        "0.00%".to_string()
    } else {
        format!("{value:+.2}%")
    }
}

fn format_change(change: f64, change_pct: f64) -> String {
    let change = if display_zero(change) {
        "0.00".to_string()
    } else {
        format!("{change:+.2}")
    };
    let change_pct = format_signed_percent(change_pct);
    format!("{change} ({change_pct})")
}

fn display_zero(value: f64) -> bool {
    (value * 100.0).round() == 0.0
}

fn format_number(value: f64) -> String {
    let rounded = format!("{:.2}", value.abs());
    let (whole, decimal) = rounded.split_once('.').unwrap_or((rounded.as_str(), "00"));
    let mut out = String::new();
    for (ix, ch) in whole.chars().rev().enumerate() {
        if ix > 0 && ix % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    let whole = out.chars().rev().collect::<String>();
    if value < 0.0 {
        format!("-{}.{}", whole, decimal)
    } else {
        format!("{}.{}", whole, decimal)
    }
}

fn format_whole_number(value: f64) -> String {
    let rounded = format!("{:.0}", value.abs());
    let mut out = String::new();
    for (ix, ch) in rounded.chars().rev().enumerate() {
        if ix > 0 && ix % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    let whole = out.chars().rev().collect::<String>();
    if value < 0.0 {
        format!("-{}", whole)
    } else {
        whole
    }
}

fn sparkline_from_change(change_pct: f64) -> Vec<f32> {
    let end = (0.45 + (change_pct / 10.0)).clamp(0.12, 0.9) as f32;
    let start = (0.45 - (change_pct / 16.0)).clamp(0.12, 0.9) as f32;
    vec![
        start,
        (start * 0.8 + end * 0.2).clamp(0.12, 0.9),
        (start * 0.6 + end * 0.4).clamp(0.12, 0.9),
        (start * 0.4 + end * 0.6).clamp(0.12, 0.9),
        (start * 0.2 + end * 0.8).clamp(0.12, 0.9),
        end,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meroalpha_api::{MarketIndexUpdate, MarketMoverUpdate, OverviewMarketData};
    use crate::portfolio::{HoldingImport, portfolio_snapshot};

    #[test]
    fn overview_snapshot_uses_imported_portfolio_value_when_available() {
        let portfolio = portfolio_snapshot(&[
            HoldingImport {
                symbol: "NABIL".to_string(),
                quantity: 10.0,
                avg_cost: 500.0,
                ltp: 550.0,
            },
            HoldingImport {
                symbol: "HDL".to_string(),
                quantity: 2.0,
                avg_cost: 1800.0,
                ltp: 2100.0,
            },
        ]);

        let overview = OverviewSnapshot::from_portfolio(Some(&portfolio));

        assert_eq!(overview.portfolio.total_value_label, "NPR 9,700.00");
        assert_eq!(overview.portfolio.change_label, "+NPR 1,100.00");
        assert_eq!(overview.portfolio.change_pct_label, "+12.79%");
        assert!(!overview.portfolio.is_empty);
    }

    #[test]
    fn zero_portfolio_move_is_neutral() {
        let portfolio = portfolio_snapshot(&[HoldingImport {
            symbol: "NABIL".to_string(),
            quantity: 10.0,
            avg_cost: 500.0,
            ltp: 500.0,
        }]);

        let overview = OverviewSnapshot::from_portfolio(Some(&portfolio));

        assert_eq!(overview.portfolio.change_label, "NPR 0.00");
        assert_eq!(overview.portfolio.change_pct_label, "0.00%");
        assert_eq!(overview.portfolio.direction, FinancialDirection::Flat);
    }

    #[test]
    fn overview_snapshot_is_stable_without_imported_holdings() {
        let overview = OverviewSnapshot::from_portfolio(None);

        assert_eq!(overview.market_pulse.len(), 0);
        assert_eq!(overview.top_movers.len(), 0);
        assert_eq!(overview.portfolio.total_value_label, "No holdings");
        assert_eq!(overview.portfolio.change_label, "Portfolio inactive");
        assert_eq!(overview.portfolio.change_pct_label, "");
        assert_eq!(overview.market_status.label, "Waiting");
        assert!(!overview.market_status.live);
        assert!(overview.portfolio.is_empty);
    }

    #[test]
    fn overview_snapshot_marks_market_status_live_when_api_data_loads() {
        let overview = OverviewSnapshot::from_portfolio(None).with_market_data(
            OverviewMarketData {
                market_status: MarketStatusUpdate {
                    status: "CLOSE".to_string(),
                    is_open: false,
                    as_of: Some("2026-06-25T09:15:00Z".to_string()),
                    last_traded_date: "2026-06-25".to_string(),
                },
                indices: vec![MarketIndexUpdate {
                    label: "NEPSE".to_string(),
                    value: 2100.0,
                    change: 12.5,
                    change_pct: 0.6,
                }],
                gainers: vec![MarketMoverUpdate {
                    symbol: "NABIL".to_string(),
                    ltp: 600.0,
                    change: 10.0,
                    change_pct: 1.7,
                    volume: 12000.0,
                    turnover: 7_200_000.0,
                }],
                losers: Vec::new(),
                turnover: Vec::new(),
            },
            MoverTab::Gainers,
        );

        assert_eq!(overview.market_status.label, "Closed");
        assert!(!overview.market_status.live);
        assert_eq!(overview.last_updated, "2026-06-25");
    }

    #[test]
    fn overview_snapshot_uses_selected_mover_tab() {
        let market_data = OverviewMarketData {
            market_status: MarketStatusUpdate {
                status: "CLOSE".to_string(),
                is_open: false,
                as_of: None,
                last_traded_date: "2026-06-25".to_string(),
            },
            indices: Vec::new(),
            gainers: vec![MarketMoverUpdate {
                symbol: "GAIN".to_string(),
                ltp: 100.0,
                change: 10.0,
                change_pct: 11.1,
                volume: 1_000.0,
                turnover: 100_000.0,
            }],
            losers: vec![MarketMoverUpdate {
                symbol: "LOSE".to_string(),
                ltp: 90.0,
                change: -5.0,
                change_pct: -5.2,
                volume: 2_000.0,
                turnover: 180_000.0,
            }],
            turnover: vec![MarketMoverUpdate {
                symbol: "TURN".to_string(),
                ltp: 50.0,
                change: 1.0,
                change_pct: 2.0,
                volume: 3_000.0,
                turnover: 900_000.0,
            }],
        };

        let overview = OverviewSnapshot::from_portfolio(None)
            .with_market_data(market_data, MoverTab::Turnover);

        assert_eq!(overview.selected_mover_tab, MoverTab::Turnover);
        assert_eq!(overview.mover_metric_label, "TURNOVER (NPR)");
        assert_eq!(overview.top_movers[0].symbol, "TURN");
        assert_eq!(overview.top_movers[0].volume, "900,000");
    }

    #[test]
    fn market_error_status_is_distinct_from_closed_market() {
        let overview = OverviewSnapshot::from_portfolio(None).with_market_error();

        assert!(overview.market_status.is_error());
        assert!(!MarketStatus::waiting().is_error());
    }

    #[test]
    fn market_recency_label_describes_data_state() {
        let waiting = OverviewSnapshot::from_portfolio(None);
        assert_eq!(waiting.market_recency_label(), "Waiting for API");

        let failed = OverviewSnapshot::from_portfolio(None).with_market_error();
        assert_eq!(failed.market_recency_label(), "Request failed");

        let loaded = OverviewSnapshot::from_portfolio(None).with_market_data(
            OverviewMarketData {
                market_status: MarketStatusUpdate {
                    status: "CLOSE".to_string(),
                    is_open: false,
                    as_of: None,
                    last_traded_date: "2026-06-25".to_string(),
                },
                indices: Vec::new(),
                gainers: Vec::new(),
                losers: Vec::new(),
                turnover: Vec::new(),
            },
            MoverTab::Gainers,
        );
        assert_eq!(loaded.market_recency_label(), "Last traded 2026-06-25");
    }

    #[test]
    fn mover_tabs_explain_their_sort_order() {
        assert_eq!(
            MoverTab::Gainers.description(),
            "Highest positive price moves"
        );
        assert_eq!(
            MoverTab::Losers.description(),
            "Largest negative price moves"
        );
        assert_eq!(MoverTab::Turnover.description(), "Highest traded value");
    }

    #[test]
    fn mover_tabs_have_specific_empty_messages() {
        assert_eq!(MoverTab::Gainers.empty_message(), "No gainers returned.");
        assert_eq!(MoverTab::Losers.empty_message(), "No losers returned.");
        assert_eq!(
            MoverTab::Turnover.empty_message(),
            "No turnover leaders returned."
        );
    }

    #[test]
    fn top_movers_empty_message_reflects_market_state() {
        let waiting = OverviewSnapshot::from_portfolio(None);
        assert_eq!(
            waiting.top_movers_empty_message(),
            "No market API data loaded."
        );

        let failed = OverviewSnapshot::from_portfolio(None).with_market_error();
        assert_eq!(
            failed.top_movers_empty_message(),
            "Market data unavailable."
        );

        let loaded = OverviewSnapshot::from_portfolio(None).with_market_data(
            OverviewMarketData {
                market_status: MarketStatusUpdate {
                    status: "CLOSE".to_string(),
                    is_open: false,
                    as_of: None,
                    last_traded_date: "2026-06-25".to_string(),
                },
                indices: Vec::new(),
                gainers: Vec::new(),
                losers: Vec::new(),
                turnover: Vec::new(),
            },
            MoverTab::Turnover,
        );
        assert_eq!(
            loaded.top_movers_empty_message(),
            "No turnover leaders returned."
        );
    }

    #[test]
    fn market_status_badge_label_is_human_readable() {
        assert_eq!(MarketStatus::waiting().badge_label(), "Waiting");
        assert_eq!(MarketStatus::api_error().badge_label(), "API Error");
        assert_eq!(
            MarketStatus {
                label: "Open",
                live: true,
            }
            .badge_label(),
            "Market Open"
        );
        assert_eq!(
            MarketStatus {
                label: "Closed",
                live: false,
            }
            .badge_label(),
            "Market Closed"
        );
    }

    #[test]
    fn market_empty_copy_is_actionable() {
        assert_eq!(
            MarketStatus::api_error().empty_title(),
            "Market data unavailable"
        );
        assert_eq!(
            MarketStatus::api_error().empty_detail(),
            "Refresh again, or check your API key and connection."
        );
        assert_eq!(
            MarketStatus::waiting().empty_title(),
            "No market API data loaded"
        );
        assert_eq!(
            MarketStatus::waiting().empty_detail(),
            "Set an API key in Profile & API, then refresh Overview."
        );
    }

    #[test]
    fn zero_market_moves_are_neutral() {
        let overview = OverviewSnapshot::from_portfolio(None).with_market_data(
            OverviewMarketData {
                market_status: MarketStatusUpdate {
                    status: "CLOSE".to_string(),
                    is_open: false,
                    as_of: None,
                    last_traded_date: "2026-06-25".to_string(),
                },
                indices: vec![MarketIndexUpdate {
                    label: "NEPSE".to_string(),
                    value: 2651.52,
                    change: 0.0,
                    change_pct: 0.0,
                }],
                gainers: vec![MarketMoverUpdate {
                    symbol: "FLAT".to_string(),
                    ltp: 100.0,
                    change: 0.0,
                    change_pct: 0.0,
                    volume: 1_000.0,
                    turnover: 100_000.0,
                }],
                losers: Vec::new(),
                turnover: Vec::new(),
            },
            MoverTab::Gainers,
        );

        assert_eq!(overview.market_pulse[0].direction, FinancialDirection::Flat);
        assert_eq!(overview.market_pulse[0].change, "0.00 (0.00%)");
        assert_eq!(overview.top_movers[0].direction, FinancialDirection::Flat);
        assert_eq!(overview.top_movers[0].change, "0.00 (0.00%)");
    }

    #[test]
    fn tiny_market_moves_that_display_as_zero_are_neutral() {
        let overview = OverviewSnapshot::from_portfolio(None).with_market_data(
            OverviewMarketData {
                market_status: MarketStatusUpdate {
                    status: "CLOSE".to_string(),
                    is_open: false,
                    as_of: None,
                    last_traded_date: "2026-06-25".to_string(),
                },
                indices: vec![MarketIndexUpdate {
                    label: "NEPSE".to_string(),
                    value: 2651.52,
                    change: 0.004,
                    change_pct: -0.004,
                }],
                gainers: vec![MarketMoverUpdate {
                    symbol: "FLAT".to_string(),
                    ltp: 100.0,
                    change: -0.004,
                    change_pct: 0.004,
                    volume: 1_000.0,
                    turnover: 100_000.0,
                }],
                losers: Vec::new(),
                turnover: Vec::new(),
            },
            MoverTab::Gainers,
        );

        assert_eq!(overview.market_pulse[0].direction, FinancialDirection::Flat);
        assert_eq!(overview.market_pulse[0].change, "0.00 (0.00%)");
        assert_eq!(overview.top_movers[0].direction, FinancialDirection::Flat);
        assert_eq!(overview.top_movers[0].change, "0.00 (0.00%)");
    }

    #[test]
    fn market_direction_uses_percent_when_change_displays_flat() {
        let overview = OverviewSnapshot::from_portfolio(None).with_market_data(
            OverviewMarketData {
                market_status: MarketStatusUpdate {
                    status: "CLOSE".to_string(),
                    is_open: false,
                    as_of: None,
                    last_traded_date: "2026-06-25".to_string(),
                },
                indices: vec![MarketIndexUpdate {
                    label: "NEPSE".to_string(),
                    value: 2651.52,
                    change: 0.004,
                    change_pct: 0.25,
                }],
                gainers: vec![MarketMoverUpdate {
                    symbol: "MOVE".to_string(),
                    ltp: 100.0,
                    change: -0.004,
                    change_pct: -0.25,
                    volume: 1_000.0,
                    turnover: 100_000.0,
                }],
                losers: Vec::new(),
                turnover: Vec::new(),
            },
            MoverTab::Gainers,
        );

        assert_eq!(overview.market_pulse[0].direction, FinancialDirection::Up);
        assert_eq!(overview.market_pulse[0].change, "0.00 (+0.25%)");
        assert_eq!(overview.top_movers[0].direction, FinancialDirection::Down);
        assert_eq!(overview.top_movers[0].change, "0.00 (-0.25%)");
    }
}
