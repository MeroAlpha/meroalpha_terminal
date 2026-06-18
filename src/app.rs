use gpui::{
    AppContext, Context, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb,
};
use gpui_component::{
    Icon, IconName, Sizable, h_flex, v_flex,
    button::{Button, ButtonVariants},
    input::{Input, InputState},
};

use meroalpha_terminal::portfolio::{
    HoldingImport, PortfolioPosition, PortfolioSnapshot, portfolio_snapshot,
};

const BACKGROUND: u32 = 0x09090B;
const SURFACE: u32 = 0x111113;
const SURFACE_CONTAINER: u32 = 0x1C1C1F;
const BORDER: u32 = 0x27272A;
const TEXT: u32 = 0xFAFAFA;
const TEXT_MUTED: u32 = 0xA1A1AA;
const EMERALD: u32 = 0x34D399;
const RED: u32 = 0xFCA5A5;

pub struct MeroAlphaTerminal {
    search: gpui::Entity<InputState>,
    portfolio: PortfolioSnapshot,
}

impl MeroAlphaTerminal {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Search tickers, sectors, or signals (Cmd+K)")
        });

        Self {
            search,
            portfolio: portfolio_snapshot(&seed_holdings()),
        }
    }

    fn render_sidebar(&self) -> impl IntoElement {
        v_flex()
            .w(px(320.))
            .h_full()
            .flex_shrink_0()
            .justify_between()
            .bg(rgb(SURFACE))
            .border_r_1()
            .border_color(rgb(BORDER))
            .p_6()
            .child(
                v_flex()
                    .gap_10()
                    .child(
                        h_flex()
                            .gap_3()
                            .child(Icon::new(IconName::SquareTerminal).text_color(rgb(EMERALD)))
                            .child(
                                v_flex()
                                    .child(
                                        div()
                                            .text_color(rgb(TEXT))
                                            .text_size(px(20.))
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .child("NEPSE Terminal"),
                                    )
                                    .child(
                                        div()
                                            .text_color(rgb(TEXT_MUTED))
                                            .text_size(px(11.))
                                            .child("MARKET LIVE"),
                                    ),
                            ),
                    )
                    .child(
                        v_flex()
                            .gap_1()
                            .child(nav_item(IconName::LayoutDashboard, "Overview", false))
                            .child(nav_item(IconName::ChartPie, "Market", false))
                            .child(nav_item(IconName::HardDrive, "Portfolio", true))
                            .child(nav_item(IconName::Network, "Broker Analysis", false))
                            .child(nav_item(IconName::Cpu, "Strategy Lab", false)),
                    ),
            )
            .child(
                h_flex()
                    .gap_3()
                    .border_t_1()
                    .border_color(rgb(BORDER))
                    .pt_4()
                    .child(
                        div()
                            .size(px(32.))
                            .rounded_full()
                            .bg(rgb(SURFACE_CONTAINER))
                            .border_1()
                            .border_color(rgb(BORDER)),
                    )
                    .child(
                        v_flex()
                            .child(
                                div()
                                    .text_color(rgb(TEXT))
                                    .text_size(px(13.))
                                    .child("USER_774"),
                            )
                            .child(
                                div()
                                    .text_color(rgb(EMERALD))
                                    .text_size(px(12.))
                                    .child("Pro Active"),
                            ),
                    ),
            )
    }

    fn render_topbar(&self) -> impl IntoElement {
        h_flex()
            .h(px(64.))
            .justify_between()
            .border_b_1()
            .border_color(rgb(BORDER))
            .bg(rgb(SURFACE))
            .px_6()
            .child(
                div()
                    .w(px(480.))
                    .child(Input::new(&self.search).prefix(Icon::new(IconName::Search).small())),
            )
            .child(
                h_flex()
                    .gap_4()
                    .child(Icon::new(IconName::Network).text_color(rgb(TEXT_MUTED)))
                    .child(Icon::new(IconName::Bell).text_color(rgb(TEXT_MUTED)))
                    .child(Icon::new(IconName::Settings).text_color(rgb(TEXT_MUTED))),
            )
    }

    fn render_portfolio_page(&self) -> impl IntoElement {
        v_flex()
            .size_full()
            .bg(rgb(BACKGROUND))
            .child(self.render_topbar())
            .child(
                h_flex()
                    .items_start()
                    .gap_5()
                    .p_6()
                    .overflow_hidden()
                    .child(
                        v_flex()
                            .flex_1()
                            .gap_5()
                            .child(self.render_page_header())
                            .child(self.render_kpis())
                            .child(self.render_performance_panel())
                            .child(self.render_holdings_table()),
                    )
                    .child(self.render_right_rail()),
            )
    }

    fn render_page_header(&self) -> impl IntoElement {
        h_flex()
            .justify_between()
            .items_end()
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_color(rgb(TEXT))
                            .text_size(px(30.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Portfolio Analysis"),
                    )
                    .child(
                        h_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_color(rgb(TEXT_MUTED))
                                    .text_size(px(14.))
                                    .child("Local valuation and risk assessment. Last synced:"),
                            )
                            .child(
                                div()
                                    .text_color(rgb(TEXT))
                                    .text_size(px(14.))
                                    .child("14:22:05 NPT"),
                            ),
                    ),
            )
            .child(
                h_flex()
                    .gap_3()
                    .child(Button::new("export").icon(IconName::Copy).label("Export Data"))
                    .child(
                        Button::new("import")
                            .primary()
                            .icon(IconName::Plus)
                            .label("Import CSV"),
                    ),
            )
    }

    fn render_kpis(&self) -> impl IntoElement {
        let top = self
            .portfolio
            .top_holding_symbol
            .clone()
            .unwrap_or_else(|| "NONE".to_string());

        h_flex()
            .gap_4()
            .child(kpi_card(
                "Total Value",
                format_money(self.portfolio.total_value),
                format!("{} Today", signed_money(12_450.0)),
                IconName::Building2,
                true,
            ))
            .child(kpi_card(
                "Unrealized P/L",
                signed_money(self.portfolio.unrealized_pl),
                format!("{:+.2}% All Time", self.portfolio.unrealized_pl_pct),
                IconName::ArrowUp,
                self.portfolio.unrealized_pl >= 0.0,
            ))
            .child(kpi_card(
                "Top Holding",
                top,
                format!(
                    "Value: {}",
                    self.portfolio
                        .positions
                        .first()
                        .map(|position| format_money(position.market_value))
                        .unwrap_or_else(|| "NPR 0.00".to_string())
                ),
                IconName::Star,
                true,
            ))
    }

    fn render_performance_panel(&self) -> impl IntoElement {
        panel()
            .h(px(360.))
            .child(
                h_flex()
                    .justify_between()
                    .mb_6()
                    .child(section_title("Performance vs Benchmark"))
                    .child(
                        h_flex()
                            .gap_1()
                            .rounded(px(6.))
                            .bg(rgb(SURFACE_CONTAINER))
                            .p_1()
                            .child(range_chip("1W", false))
                            .child(range_chip("1M", false))
                            .child(range_chip("3M", false))
                            .child(range_chip("YTD", true))
                            .child(range_chip("1Y", false))
                            .child(range_chip("ALL", false)),
                    ),
            )
            .child(
                div()
                    .relative()
                    .h(px(260.))
                    .border_l_1()
                    .border_b_1()
                    .border_color(rgb(BORDER))
                    .ml_8()
                    .child(chart_grid())
                    .child(chart_line()),
            )
    }

    fn render_holdings_table(&self) -> impl IntoElement {
        panel()
            .child(
                h_flex()
                    .justify_between()
                    .mb_5()
                    .child(section_title("Holdings Ledger"))
                    .child(
                        div()
                            .rounded(px(6.))
                            .border_1()
                            .border_color(rgb(BORDER))
                            .bg(rgb(SURFACE_CONTAINER))
                            .px_3()
                            .py_1()
                            .text_color(rgb(TEXT_MUTED))
                            .text_size(px(12.))
                            .child(format!("{} Positions", self.portfolio.positions.len())),
                    ),
            )
            .child(table_header())
            .children(
                self.portfolio
                    .positions
                    .iter()
                    .map(render_position_row)
                    .collect::<Vec<_>>(),
            )
    }

    fn render_right_rail(&self) -> impl IntoElement {
        v_flex()
            .w(px(392.))
            .flex_shrink_0()
            .gap_5()
            .child(
                panel()
                    .child(
                        h_flex()
                            .justify_between()
                            .mb_6()
                            .child(section_title("AI Intelligence"))
                            .child(
                                div()
                                    .text_color(rgb(TEXT_MUTED))
                                    .text_size(px(12.))
                                    .child("BETA v0.9"),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_5()
                            .mb_5()
                            .p_4()
                            .rounded(px(6.))
                            .border_1()
                            .border_color(rgb(BORDER))
                            .bg(rgb(BACKGROUND))
                            .child(risk_ring("24"))
                            .child(
                                v_flex()
                                    .gap_2()
                                    .child(div().text_color(rgb(TEXT)).text_size(px(18.)).child("Low Risk Profile"))
                                    .child(
                                        div()
                                            .text_color(rgb(TEXT_MUTED))
                                            .text_size(px(14.))
                                            .child("Beta: 0.85. Less volatile than market."),
                                    ),
                            ),
                    )
                    .child(insight("High Sector Concentration", "Commercial Banks constitute 48% of total equity. Consider diversifying into Hydro or Manufacturing.", false))
                    .child(insight("Optimal Holding Period", "NABIL has reached historical resistance levels. Statistical probability of near-term pullback is 68%.", true)),
            )
            .child(
                panel()
                    .child(section_title("Corporate Actions"))
                    .child(action_item("OCT 12", "NABIL", "Cash Div", "11.00%"))
                    .child(action_item("NOV 05", "GBIME", "Bonus", "8.50%")),
            )
            .child(
                panel()
                    .child(
                        div()
                            .text_color(rgb(TEXT_MUTED))
                            .text_size(px(12.))
                            .mb_4()
                            .child("API_RESPONSE: PORTFOLIO_SYNC"),
                    )
                    .child(
                        div()
                            .text_color(rgb(TEXT_MUTED))
                            .text_size(px(12.))
                            .child("{ \"status\": \"local\", \"positions\": 4 }"),
                    ),
            )
    }
}

impl Render for MeroAlphaTerminal {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .size_full()
            .bg(rgb(BACKGROUND))
            .child(self.render_sidebar())
            .child(self.render_portfolio_page())
    }
}

fn seed_holdings() -> Vec<HoldingImport> {
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
        HoldingImport {
            symbol: "SHIVM".to_string(),
            quantity: 1200.0,
            avg_cost: 540.00,
            ltp: 490.50,
        },
        HoldingImport {
            symbol: "NTC".to_string(),
            quantity: 850.0,
            avg_cost: 880.00,
            ltp: 910.00,
        },
    ]
}

fn nav_item(icon: IconName, label: &'static str, active: bool) -> impl IntoElement {
    h_flex()
        .gap_3()
        .rounded(px(6.))
        .px_4()
        .py_3()
        .bg(if active { rgb(SURFACE_CONTAINER) } else { rgb(SURFACE) })
        .border_r_2()
        .border_color(if active { rgb(EMERALD) } else { rgb(SURFACE) })
        .child(Icon::new(icon).text_color(if active { rgb(TEXT) } else { rgb(TEXT_MUTED) }))
        .child(
            div()
                .text_color(if active { rgb(TEXT) } else { rgb(TEXT_MUTED) })
                .text_size(px(15.))
                .child(label),
        )
}

fn panel() -> gpui::Div {
    div()
        .rounded(px(8.))
        .border_1()
        .border_color(rgb(BORDER))
        .bg(rgb(SURFACE))
        .p_5()
}

fn section_title(title: &'static str) -> impl IntoElement {
    div()
        .text_color(rgb(TEXT))
        .text_size(px(20.))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .child(title)
}

fn kpi_card(
    label: &'static str,
    value: impl Into<gpui::SharedString>,
    footer: impl Into<gpui::SharedString>,
    icon: IconName,
    positive: bool,
) -> impl IntoElement {
    panel()
        .h(px(128.))
        .flex_1()
        .justify_between()
        .child(
            h_flex()
                .justify_between()
                .child(div().text_color(rgb(TEXT_MUTED)).text_size(px(14.)).child(label))
                .child(Icon::new(icon).small().text_color(rgb(TEXT_MUTED))),
        )
        .child(
            v_flex()
                .gap_1()
                .child(
                    div()
                        .text_color(if positive { rgb(EMERALD) } else { rgb(RED) })
                        .text_size(px(24.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child(value.into()),
                )
                .child(
                    div()
                        .text_color(if positive { rgb(EMERALD) } else { rgb(RED) })
                        .text_size(px(13.))
                        .child(footer.into()),
                ),
        )
}

fn range_chip(label: &'static str, active: bool) -> impl IntoElement {
    div()
        .rounded(px(4.))
        .px_3()
        .py_1()
        .bg(if active { rgb(SURFACE) } else { rgb(SURFACE_CONTAINER) })
        .border_1()
        .border_color(if active { rgb(BORDER) } else { rgb(SURFACE_CONTAINER) })
        .text_color(if active { rgb(TEXT) } else { rgb(TEXT_MUTED) })
        .text_size(px(12.))
        .child(label)
}

fn chart_grid() -> impl IntoElement {
    v_flex()
        .absolute()
        .inset_0()
        .justify_between()
        .children((0..5).map(|_| {
            div()
                .h(px(1.))
                .w_full()
                .bg(rgb(BORDER))
                .opacity(0.55)
        }))
}

fn chart_line() -> impl IntoElement {
    h_flex()
        .absolute()
        .left(px(0.))
        .right(px(0.))
        .bottom(px(52.))
        .h(px(110.))
        .items_end()
        .justify_between()
        .px_8()
        .children([45., 70., 78., 118., 150., 178.].into_iter().map(|height| {
            div()
                .w(px(9.))
                .h(px(height))
                .rounded_full()
                .bg(rgb(EMERALD))
        }))
}

fn table_header() -> impl IntoElement {
    h_flex()
        .bg(rgb(SURFACE_CONTAINER))
        .border_b_1()
        .border_color(rgb(BORDER))
        .px_4()
        .py_3()
        .child(table_cell("TICKER", 1.2, TEXT_MUTED))
        .child(table_cell("QTY", 1.0, TEXT_MUTED))
        .child(table_cell("AVG COST", 1.0, TEXT_MUTED))
        .child(table_cell("LTP", 1.0, TEXT_MUTED))
        .child(table_cell("CUR VAL", 1.2, TEXT_MUTED))
        .child(table_cell("P/L %", 1.0, TEXT_MUTED))
}

fn render_position_row(position: &PortfolioPosition) -> impl IntoElement {
    h_flex()
        .border_b_1()
        .border_color(rgb(BORDER))
        .px_4()
        .py_4()
        .child(table_cell(position.symbol.clone(), 1.2, TEXT))
        .child(table_cell(format!("{:.0}", position.quantity), 1.0, TEXT))
        .child(table_cell(format!("{:.2}", position.avg_cost), 1.0, TEXT_MUTED))
        .child(table_cell(format!("{:.2}", position.ltp), 1.0, TEXT))
        .child(table_cell(format_number(position.market_value), 1.2, TEXT))
        .child(table_cell(
            format!("{:+.2}%", position.unrealized_pl_pct),
            1.0,
            if position.unrealized_pl >= 0.0 { EMERALD } else { RED },
        ))
}

fn table_cell(content: impl Into<gpui::SharedString>, grow: f32, color: u32) -> impl IntoElement {
    div()
        .flex_grow(grow)
        .flex_basis(px(100. * grow))
        .text_color(rgb(color))
        .text_size(px(13.))
        .child(content.into())
}

fn risk_ring(score: &'static str) -> impl IntoElement {
    div()
        .size(px(72.))
        .rounded_full()
        .border_4()
        .border_color(rgb(EMERALD))
        .flex()
        .items_center()
        .justify_center()
        .child(div().text_color(rgb(TEXT)).text_size(px(22.)).child(score))
}

fn insight(title: &'static str, body: &'static str, positive: bool) -> impl IntoElement {
    h_flex()
        .items_start()
        .gap_3()
        .mb_4()
        .child(
            div()
                .size(px(10.))
                .mt_1()
                .rounded_full()
                .bg(if positive { rgb(EMERALD) } else { rgb(RED) }),
        )
        .child(
            v_flex()
                .gap_1()
                .child(div().text_color(rgb(TEXT)).text_size(px(16.)).child(title))
                .child(div().text_color(rgb(TEXT_MUTED)).text_size(px(13.)).child(body)),
        )
}

fn action_item(date: &'static str, symbol: &'static str, label: &'static str, value: &'static str) -> impl IntoElement {
    h_flex()
        .justify_between()
        .border_t_1()
        .border_color(rgb(BORDER))
        .py_4()
        .child(
            h_flex()
                .gap_3()
                .child(
                    div()
                        .rounded(px(6.))
                        .bg(rgb(SURFACE_CONTAINER))
                        .px_3()
                        .py_2()
                        .text_color(rgb(TEXT))
                        .text_size(px(13.))
                        .child(date),
                )
                .child(
                    v_flex()
                        .child(div().text_color(rgb(TEXT)).text_size(px(16.)).child(symbol))
                        .child(div().text_color(rgb(TEXT_MUTED)).text_size(px(13.)).child(label)),
                ),
        )
        .child(div().text_color(rgb(EMERALD)).text_size(px(16.)).child(value))
}

fn format_money(value: f64) -> String {
    format!("NPR {}", format_number(value))
}

fn signed_money(value: f64) -> String {
    if value >= 0.0 {
        format!("+{}", format_money(value))
    } else {
        format!("-{}", format_money(value.abs()))
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
