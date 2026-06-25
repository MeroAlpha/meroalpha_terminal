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
            Self::Turnover => "TURNOVER",
            Self::Gainers | Self::Losers => "VOLUME",
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
    pub detail: String,
    pub live: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MarketPulseCard {
    pub label: String,
    pub value: String,
    pub change: String,
    pub positive: bool,
    pub sparkline: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PortfolioPerformance {
    pub total_value_label: String,
    pub change_label: String,
    pub change_pct_label: String,
    pub risk_label: &'static str,
    pub risk_score_label: &'static str,
    pub positive: bool,
    pub is_empty: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MarketMover {
    pub symbol: String,
    pub ltp: String,
    pub change: String,
    pub volume: String,
    pub positive: bool,
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

    pub fn with_market_error(mut self) -> Self {
        self.market_status = MarketStatus::api_error();
        self.last_updated = "Request failed".to_string();
        self
    }
}

impl MarketStatus {
    fn waiting() -> Self {
        Self {
            label: "Waiting",
            detail: "Awaiting market data".to_string(),
            live: false,
        }
    }

    fn from_update(update: &MarketStatusUpdate) -> Self {
        Self {
            label: if update.is_open { "Open" } else { "Closed" },
            detail: format!("Last traded {}", update.last_traded_date),
            live: update.is_open,
        }
    }

    fn api_error() -> Self {
        Self {
            label: "API Error",
            detail: "See terminal logs".to_string(),
            live: false,
        }
    }
}

impl MarketPulseCard {
    fn from_update(update: &MarketIndexUpdate) -> Self {
        Self {
            label: update.label.clone(),
            value: format_number(update.value),
            change: format!("{:+.2} ({:+.2}%)", update.change, update.change_pct),
            positive: update.change >= 0.0,
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
            change: format!("{:+.2} ({:+.2}%)", update.change, update.change_pct),
            volume: format_whole_number(metric),
            positive: update.change >= 0.0,
        }
    }
}

impl PortfolioPerformance {
    fn from_portfolio(portfolio: Option<&PortfolioSnapshot>) -> Self {
        match portfolio {
            Some(portfolio) => Self {
                total_value_label: format_npr(portfolio.total_value),
                change_label: format_signed_npr(portfolio.unrealized_pl),
                change_pct_label: format!("{:+.2}%", portfolio.unrealized_pl_pct),
                risk_label: "Local Portfolio",
                risk_score_label: allocation_risk_label(portfolio.positions.len()),
                positive: portfolio.unrealized_pl >= 0.0,
                is_empty: false,
            },
            None => Self {
                total_value_label: "Import CSV".to_string(),
                change_label: "Local portfolio inactive".to_string(),
                change_pct_label: "0.00%".to_string(),
                risk_label: "Awaiting holdings",
                risk_score_label: "0 positions",
                positive: true,
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
    if value >= 0.0 {
        format!("+{}", format_npr(value))
    } else {
        format!("-{}", format_npr(value.abs()))
    }
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
    fn overview_snapshot_is_stable_without_imported_holdings() {
        let overview = OverviewSnapshot::from_portfolio(None);

        assert_eq!(overview.market_pulse.len(), 0);
        assert_eq!(overview.top_movers.len(), 0);
        assert_eq!(overview.portfolio.total_value_label, "Import CSV");
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
        assert_eq!(overview.market_status.detail, "Last traded 2026-06-25");
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
        assert_eq!(overview.mover_metric_label, "TURNOVER");
        assert_eq!(overview.top_movers[0].symbol, "TURN");
        assert_eq!(overview.top_movers[0].volume, "900,000");
    }
}
