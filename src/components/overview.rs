use gpui::{DefiniteLength, IntoElement, ParentElement, Styled, div, px, InteractiveElement as _, prelude::FluentBuilder as _};
use gpui_component::{
    Icon, IconName, Selectable, Sizable, Theme, Colorize as _,
    button::{Button, ButtonVariants},
    h_flex,
    scroll::ScrollableElement,
    v_flex,
};

use crate::components::theme::{
    elevated_panel, section_heading, table_cell, tone_for_signed_value,
};
use crate::overview::{
    FinancialDirection, MarketMover, MarketPulseCard, MarketStatus, MoverTab, OverviewSnapshot,
    PortfolioPerformance,
};

pub fn render_overview_page(
    snapshot: OverviewSnapshot,
    theme: Theme,
    window: &mut gpui::Window,
    on_gainers: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
    on_losers: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
    on_turnover: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    let window_width = window.bounds().size.width;
    let is_stacked = window_width < px(1180.);

    v_flex()
        .w_full()
        .bg(theme.background)
        .p_5()
        .gap_4()
        .child(
            if is_stacked {
                v_flex()
                    .gap_4()
                    .w_full()
                    .child(market_pulse_panel(&snapshot, &theme, true))
                    .child(portfolio_performance_panel(&snapshot, &theme, true))
            } else {
                h_flex()
                    .gap_4()
                    .items_stretch()
                    .child(market_pulse_panel(&snapshot, &theme, false))
                    .child(portfolio_performance_panel(&snapshot, &theme, false))
            }
        )
        .child(top_movers_panel(
            &snapshot,
            &theme,
            on_gainers,
            on_losers,
            on_turnover,
        ))
}

fn market_pulse_panel(snapshot: &OverviewSnapshot, theme: &Theme, is_stacked: bool) -> impl IntoElement {
    elevated_panel(theme)
        .flex_1()
        .when(!is_stacked, |el| el.min_w(px(560.)))
        .min_h(px(272.))
        .child(
            h_flex()
                .mb_5()
                .gap_3()
                .justify_between()
                .items_center()
                .child(
                    h_flex()
                        .flex_1()
                        .min_w(px(0.))
                        .gap_3()
                        .items_center()
                        .child(
                            div()
                                .flex_shrink_0()
                                .text_color(theme.foreground)
                                .text_size(px(18.))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .child("Market Pulse"),
                        )
                        .child(
                            div()
                                .min_w(px(0.))
                                .text_color(theme.muted_foreground)
                                .text_size(px(13.))
                                .truncate()
                                .child(snapshot.market_recency_label()),
                        ),
                )
                .child(market_status_badge(&snapshot.market_status, theme)),
        )
        .child(h_flex().gap_4().items_stretch().flex_wrap().children(
            if snapshot.market_pulse.is_empty() {
                vec![market_empty_state(&snapshot.market_status, theme).into_any_element()]
            } else {
                snapshot
                    .market_pulse
                    .iter()
                    .map(|card| market_pulse_card(card, theme).into_any_element())
                    .collect::<Vec<_>>()
            },
        ))
}

fn market_status_badge(status: &MarketStatus, theme: &Theme) -> impl IntoElement {
    let accent = if status.is_error() {
        theme.danger
    } else if status.live {
        theme.success
    } else {
        theme.warning
    };

    h_flex()
        .gap_2()
        .h(px(30.))
        .px_3()
        .flex_shrink_0()
        .rounded(theme.radius)
        .border_1()
        .border_color(accent)
        .bg(accent.opacity(0.12))
        .child(div().size(px(7.)).rounded_full().bg(accent))
        .child(
            div()
                .text_color(accent)
                .text_size(px(11.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .child(status.badge_label()),
        )
}

fn market_pulse_card(card: &MarketPulseCard, theme: &Theme) -> impl IntoElement {
    let tone = tone_for_direction(theme, card.direction);

    v_flex()
        .flex_1()
        .min_w(px(128.))
        .h(px(184.))
        .justify_between()
        .rounded(theme.radius)
        .border_1()
        .border_color(theme.border)
        .bg(theme.background)
        .p_3()
        .hover(|style| style.bg(theme.secondary).border_color(tone.accent))
        .child(
            h_flex()
                .size_full()
                .gap_3()
                .child(
                    // Interactive direction vertical highlight line
                    div()
                        .w(px(3.))
                        .h_full()
                        .rounded_full()
                        .bg(tone.accent)
                )
                .child(
                    v_flex()
                        .flex_1()
                        .h_full()
                        .justify_between()
                        .child(
                            h_flex()
                                .justify_between()
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w(px(0.))
                                        .text_color(theme.muted_foreground)
                                        .text_size(px(11.))
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .truncate()
                                        .child(card.label.clone()),
                                )
                                .child(
                                    Icon::new(financial_direction_icon(card.direction))
                                        .xsmall()
                                        .text_color(tone.accent),
                                ),
                        )
                        .child(
                            v_flex()
                                .gap_1()
                                .child(
                                    div()
                                        .text_color(theme.foreground)
                                        .text_size(px(20.))
                                        .child(card.value.clone()),
                                )
                                .child(
                                    div()
                                        .text_color(tone.accent)
                                        .text_size(px(12.))
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .child(card.change.clone()),
                                ),
                        )
                        .child(sparkline(&card.sparkline, tone.accent, theme))
                )
        )
}

fn portfolio_performance_panel(snapshot: &OverviewSnapshot, theme: &Theme, is_stacked: bool) -> impl IntoElement {
    let performance = &snapshot.portfolio;
    let tone = tone_for_direction(theme, performance.direction);
    let accent = tone.accent;

    elevated_panel(theme)
        .when(is_stacked, |el| el.flex_1().min_w(px(280.)))
        .when(!is_stacked, |el| el.w(px(320.)).flex_shrink_0())
        .min_h(px(272.))
        .justify_between()
        .bg(theme.secondary.mix_oklab(accent, 0.04)) // Dynamic subtle overlay matching direction
        .child(
            div()
                .mb_5()
                .child(section_heading(theme, "Portfolio Performance", "local")),
        )
        .child(
            v_flex()
                .gap_2()
                .child(
                    div()
                        .text_color(theme.muted_foreground)
                        .text_size(px(11.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child("TOTAL VALUE"),
                )
                .child(
                    div()
                        .min_w(px(0.))
                        .text_color(theme.foreground)
                        .text_size(px(if performance.is_empty { 24. } else { 28. }))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .truncate()
                        .child(performance.total_value_label.clone()),
                )
                .child(
                    div()
                        .pt_3()
                        .text_color(theme.muted_foreground)
                        .text_size(px(11.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child(if performance.is_empty {
                            "STATUS"
                        } else {
                            "UNREALIZED P/L"
                        }),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .min_w(px(0.))
                        .child(
                            Icon::new(portfolio_direction_icon(performance))
                                .xsmall()
                                .text_color(accent),
                        )
                        .child(
                            div()
                                .min_w(px(0.))
                                .text_color(accent)
                                .text_size(px(17.))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .truncate()
                                .child(portfolio_change_label(performance)),
                        ),
                )
                .child(div().my_3().h(px(1.)).w_full().bg(theme.border))
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            div()
                                .text_color(theme.muted_foreground)
                                .text_size(px(11.))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .child("RISK VIEW"),
                        )
                        .child(
                            h_flex()
                                .flex_1()
                                .min_w(px(0.))
                                .gap_2()
                                .px_2()
                                .py_1()
                                .rounded(theme.radius)
                                .bg(theme.muted)
                                .child(div().size(px(7.)).rounded_full().bg(accent))
                                .child(
                                    div()
                                        .min_w(px(0.))
                                        .text_color(theme.foreground)
                                        .text_size(px(12.))
                                        .truncate()
                                        .child(format!(
                                            "{} - {}",
                                            performance.risk_label, performance.risk_score_label
                                        )),
                                ),
                        ),
                ),
        )
        .child(sparkline(
            &[0.34, 0.42, 0.40, 0.58, 0.50, 0.70],
            accent,
            theme,
        ))
}

fn top_movers_panel(
    snapshot: &OverviewSnapshot,
    theme: &Theme,
    on_gainers: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
    on_losers: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
    on_turnover: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    elevated_panel(theme)
        .w_full()
        .child(
            h_flex()
                .mb_4()
                .gap_3()
                .justify_between()
                .items_center()
                .child(section_heading(
                    theme,
                    "Top Movers",
                    snapshot.selected_mover_tab.label(),
                ))
                .child(
                    h_flex()
                        .flex_shrink_0()
                        .gap_1()
                        .rounded(theme.radius)
                        .border_1()
                        .border_color(theme.border)
                        .bg(theme.background)
                        .p_1()
                        .child(mover_tab_button(
                            "top-movers-gainers",
                            MoverTab::Gainers,
                            snapshot.selected_mover_tab == MoverTab::Gainers,
                            on_gainers,
                        ))
                        .child(mover_tab_button(
                            "top-movers-losers",
                            MoverTab::Losers,
                            snapshot.selected_mover_tab == MoverTab::Losers,
                            on_losers,
                        ))
                        .child(mover_tab_button(
                            "top-movers-turnover",
                            MoverTab::Turnover,
                            snapshot.selected_mover_tab == MoverTab::Turnover,
                            on_turnover,
                        )),
                ),
        )
        .child(
            div().overflow_x_scrollbar().child(
                v_flex()
                    .min_w(px(680.))
                    .child(movers_header(theme, snapshot.mover_metric_label))
                    .children(if snapshot.top_movers.is_empty() {
                        vec![
                            movers_empty_state(theme, snapshot.top_movers_empty_message())
                                .into_any_element(),
                        ]
                    } else {
                        snapshot
                            .top_movers
                            .iter()
                            .map(|mover| mover_row(mover, theme).into_any_element())
                            .collect::<Vec<_>>()
                    }),
            ),
        )
}

fn mover_tab_button(
    id: &'static str,
    tab: MoverTab,
    active: bool,
    on_click: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    let button = Button::new(id)
        .xsmall()
        .label(tab.label())
        .selected(active)
        .tooltip(tab.description())
        .on_click(on_click);
    if active { button } else { button.ghost() }
}

fn market_empty_state(status: &MarketStatus, theme: &Theme) -> impl IntoElement {
    let request_failed = status.is_error();
    let accent = if request_failed {
        theme.danger
    } else {
        theme.muted_foreground
    };

    v_flex()
        .flex_1()
        .min_w(px(280.))
        .h(px(184.))
        .items_center()
        .justify_center()
        .rounded(theme.radius)
        .border_1()
        .border_color(theme.border)
        .bg(theme.background)
        .gap_2()
        .child(
            Icon::new(market_empty_icon(status))
                .small()
                .text_color(accent),
        )
        .child(
            div()
                .max_w(px(260.))
                .text_color(theme.foreground)
                .text_size(px(14.))
                .text_align(gpui::TextAlign::Center)
                .font_weight(gpui::FontWeight::MEDIUM)
                .truncate()
                .child(status.empty_title()),
        )
        .child(
            div()
                .max_w(px(320.))
                .text_color(theme.muted_foreground)
                .text_size(px(12.))
                .text_align(gpui::TextAlign::Center)
                .truncate()
                .child(status.empty_detail()),
        )
}

fn market_empty_icon(status: &MarketStatus) -> IconName {
    if status.is_error() {
        IconName::TriangleAlert
    } else {
        IconName::Info
    }
}

fn movers_empty_state(theme: &Theme, message: &'static str) -> impl IntoElement {
    h_flex()
        .border_b_1()
        .border_color(theme.table_row_border)
        .px_4()
        .py_5()
        .child(
            div()
                .flex_1()
                .min_w(px(0.))
                .text_color(theme.muted_foreground)
                .text_size(px(13.))
                .truncate()
                .child(message),
        )
}

fn movers_header(theme: &Theme, metric_label: &'static str) -> impl IntoElement {
    h_flex()
        .rounded_t(px(6.))
        .bg(theme.table_head)
        .border_b_1()
        .border_color(theme.table_row_border)
        .px_4()
        .py_3()
        .child(table_cell(
            "SYMBOL",
            1.0,
            theme.table_head_foreground,
            false,
        ))
        .child(table_cell("LTP", 1.0, theme.table_head_foreground, true))
        .child(table_cell("CHANGE", 1.4, theme.table_head_foreground, true))
        .child(table_cell(
            metric_label,
            1.0,
            theme.table_head_foreground,
            true,
        ))
}

fn mover_row(mover: &MarketMover, theme: &Theme) -> impl IntoElement {
    let tone = tone_for_direction(theme, mover.direction);

    h_flex()
        .border_b_1()
        .border_color(theme.table_row_border)
        .px_4()
        .py_3()
        .child(table_cell(
            mover.symbol.clone(),
            1.0,
            theme.foreground,
            false,
        ))
        .child(table_cell(mover.ltp.clone(), 1.0, theme.foreground, true))
        .child(table_cell(mover.change.clone(), 1.4, tone.accent, true))
        .child(table_cell(
            mover.volume.clone(),
            1.0,
            theme.foreground,
            true,
        ))
}

fn tone_for_direction(
    theme: &Theme,
    direction: FinancialDirection,
) -> crate::components::theme::Tone {
    match direction {
        FinancialDirection::Up => tone_for_signed_value(theme, 1.0),
        FinancialDirection::Down => tone_for_signed_value(theme, -1.0),
        FinancialDirection::Flat => crate::components::theme::Tone {
            accent: theme.muted_foreground,
            foreground: theme.muted_foreground,
            surface: theme.muted,
            border: theme.border,
        },
    }
}

fn financial_direction_icon(direction: FinancialDirection) -> IconName {
    match direction {
        FinancialDirection::Up => IconName::ArrowUp,
        FinancialDirection::Down => IconName::ArrowDown,
        FinancialDirection::Flat => IconName::Minus,
    }
}

fn portfolio_direction_icon(performance: &PortfolioPerformance) -> IconName {
    if performance.is_empty {
        IconName::Info
    } else {
        financial_direction_icon(performance.direction)
    }
}

fn portfolio_change_label(performance: &PortfolioPerformance) -> String {
    if performance.is_empty {
        performance.change_label.clone()
    } else {
        format!(
            "{} ({})",
            performance.change_label, performance.change_pct_label
        )
    }
}

fn sparkline(values: &[f32], color: gpui::Hsla, theme: &Theme) -> impl IntoElement {
    h_flex()
        .h(px(34.))
        .gap_1()
        .items_end()
        .children(values.iter().map(|value| {
            div()
                .flex_1()
                .h(DefiniteLength::Fraction(value.clamp(0.1, 1.0)))
                .rounded(px(2.))
                .bg(color)
                .border_1()
                .border_color(theme.background)
        }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn financial_direction_icon_matches_sign() {
        assert!(matches!(
            financial_direction_icon(FinancialDirection::Up),
            IconName::ArrowUp
        ));
        assert!(matches!(
            financial_direction_icon(FinancialDirection::Down),
            IconName::ArrowDown
        ));
        assert!(matches!(
            financial_direction_icon(FinancialDirection::Flat),
            IconName::Minus
        ));
    }

    #[test]
    fn portfolio_direction_icon_is_neutral_when_empty() {
        let empty = PortfolioPerformance {
            total_value_label: "No holdings".to_string(),
            change_label: "Portfolio inactive".to_string(),
            change_pct_label: String::new(),
            risk_label: "Awaiting holdings",
            risk_score_label: "0 positions",
            direction: FinancialDirection::Flat,
            is_empty: true,
        };
        let positive = PortfolioPerformance {
            direction: FinancialDirection::Up,
            is_empty: false,
            ..empty.clone()
        };
        let negative = PortfolioPerformance {
            direction: FinancialDirection::Down,
            is_empty: false,
            ..empty.clone()
        };

        assert!(matches!(portfolio_direction_icon(&empty), IconName::Info));
        assert!(matches!(
            portfolio_direction_icon(&positive),
            IconName::ArrowUp
        ));
        assert!(matches!(
            portfolio_direction_icon(&negative),
            IconName::ArrowDown
        ));
    }

    #[test]
    fn portfolio_change_label_omits_percent_when_empty() {
        let empty = PortfolioPerformance {
            total_value_label: "No holdings".to_string(),
            change_label: "Portfolio inactive".to_string(),
            change_pct_label: String::new(),
            risk_label: "Awaiting holdings",
            risk_score_label: "0 positions",
            direction: FinancialDirection::Flat,
            is_empty: true,
        };
        let loaded = PortfolioPerformance {
            change_label: "+NPR 10.00".to_string(),
            change_pct_label: "+1.00%".to_string(),
            is_empty: false,
            ..empty.clone()
        };

        assert_eq!(portfolio_change_label(&empty), "Portfolio inactive");
        assert_eq!(portfolio_change_label(&loaded), "+NPR 10.00 (+1.00%)");
    }

    #[test]
    fn market_empty_icon_distinguishes_api_error() {
        let error = MarketStatus {
            label: "API Error",
            live: false,
        };
        let waiting = MarketStatus {
            label: "Waiting",
            live: false,
        };

        assert!(matches!(market_empty_icon(&error), IconName::TriangleAlert));
        assert!(matches!(market_empty_icon(&waiting), IconName::Info));
    }
}
