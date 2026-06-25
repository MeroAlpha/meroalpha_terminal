use gpui::{DefiniteLength, IntoElement, ParentElement, Styled, div, px};
use gpui_component::{
    Icon, IconName, Sizable, Theme,
    button::{Button, ButtonVariants},
    h_flex,
    scroll::ScrollableElement,
    v_flex,
};

use crate::components::theme::{
    elevated_panel, section_heading, table_cell, tone_for_signed_value,
};
use crate::overview::{MarketMover, MarketPulseCard, MarketStatus, MoverTab, OverviewSnapshot};

pub fn render_overview_page(
    snapshot: OverviewSnapshot,
    theme: Theme,
    on_gainers: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
    on_losers: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
    on_turnover: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    v_flex()
        .size_full()
        .min_w(px(840.))
        .bg(theme.background)
        .overflow_scrollbar()
        .p_5()
        .gap_4()
        .child(
            h_flex()
                .gap_4()
                .items_stretch()
                .flex_wrap()
                .child(market_pulse_panel(&snapshot, &theme))
                .child(portfolio_performance_panel(&snapshot, &theme)),
        )
        .child(top_movers_panel(
            &snapshot,
            &theme,
            on_gainers,
            on_losers,
            on_turnover,
        ))
}

fn market_pulse_panel(snapshot: &OverviewSnapshot, theme: &Theme) -> impl IntoElement {
    elevated_panel(theme)
        .flex_1()
        .min_w(px(560.))
        .min_h(px(272.))
        .child(
            h_flex()
                .mb_5()
                .gap_3()
                .justify_between()
                .items_center()
                .child(
                    h_flex()
                        .gap_3()
                        .items_center()
                        .child(
                            div()
                                .text_color(theme.foreground)
                                .text_size(px(18.))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .child("Market Pulse"),
                        )
                        .child(
                            div()
                                .text_color(theme.muted_foreground)
                                .text_size(px(13.))
                                .child(format!("Updated {}", snapshot.last_updated)),
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
    let accent = if status.live {
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
                .child(format!("{} Status", status.label)),
        )
        .child(
            div()
                .text_color(theme.muted_foreground)
                .text_size(px(11.))
                .child(status.detail.clone()),
        )
}

fn market_pulse_card(card: &MarketPulseCard, theme: &Theme) -> impl IntoElement {
    let tone = tone_for_signed_value(theme, if card.positive { 1.0 } else { -1.0 });

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
        .child(
            h_flex()
                .justify_between()
                .child(
                    div()
                        .text_color(theme.muted_foreground)
                        .text_size(px(11.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child(card.label.clone()),
                )
                .child(
                    Icon::new(if card.positive {
                        IconName::ArrowUp
                    } else {
                        IconName::TriangleAlert
                    })
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
}

fn portfolio_performance_panel(snapshot: &OverviewSnapshot, theme: &Theme) -> impl IntoElement {
    let performance = &snapshot.portfolio;
    let tone = tone_for_signed_value(theme, if performance.positive { 1.0 } else { -1.0 });

    elevated_panel(theme)
        .w(px(320.))
        .min_h(px(272.))
        .flex_shrink_0()
        .justify_between()
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
                        .text_color(theme.foreground)
                        .text_size(px(if performance.is_empty { 24. } else { 28. }))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
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
                        .child(
                            Icon::new(IconName::ArrowUp)
                                .xsmall()
                                .text_color(tone.accent),
                        )
                        .child(
                            div()
                                .text_color(tone.accent)
                                .text_size(px(17.))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .child(format!(
                                    "{} ({})",
                                    performance.change_label, performance.change_pct_label
                                )),
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
                                .gap_2()
                                .px_2()
                                .py_1()
                                .rounded(theme.radius)
                                .bg(theme.muted)
                                .child(div().size(px(7.)).rounded_full().bg(tone.accent))
                                .child(
                                    div().text_color(theme.foreground).text_size(px(12.)).child(
                                        format!(
                                            "{} - {}",
                                            performance.risk_label, performance.risk_score_label
                                        ),
                                    ),
                                ),
                        ),
                ),
        )
        .child(sparkline(
            &[0.34, 0.42, 0.40, 0.58, 0.50, 0.70],
            tone.accent,
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
        .flex_1()
        .min_w(px(720.))
        .child(
            h_flex()
                .mb_4()
                .justify_between()
                .child(section_heading(
                    theme,
                    "Top Movers (Live)",
                    snapshot.selected_mover_tab.label(),
                ))
                .child(
                    h_flex()
                        .gap_2()
                        .child(mover_tab_button(
                            "top-movers-gainers",
                            "Gainers",
                            snapshot.selected_mover_tab == MoverTab::Gainers,
                            on_gainers,
                        ))
                        .child(mover_tab_button(
                            "top-movers-losers",
                            "Losers",
                            snapshot.selected_mover_tab == MoverTab::Losers,
                            on_losers,
                        ))
                        .child(mover_tab_button(
                            "top-movers-turnover",
                            "Turnover",
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
                        vec![movers_empty_state(theme).into_any_element()]
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
    label: &'static str,
    active: bool,
    on_click: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    let button = Button::new(id).xsmall().label(label).on_click(on_click);
    if active { button } else { button.ghost() }
}

fn market_empty_state(status: &MarketStatus, theme: &Theme) -> impl IntoElement {
    let request_failed = status.label == "API Error";

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
            Icon::new(IconName::Info)
                .small()
                .text_color(theme.muted_foreground),
        )
        .child(
            div()
                .text_color(theme.foreground)
                .text_size(px(14.))
                .font_weight(gpui::FontWeight::MEDIUM)
                .child(if request_failed {
                    "Market API request failed"
                } else {
                    "No market API data loaded"
                }),
        )
        .child(
            div()
                .text_color(theme.muted_foreground)
                .text_size(px(12.))
                .child(if request_failed {
                    "See terminal logs for base URL, route, status, and response body."
                } else {
                    "Set an API key and refresh Overview."
                }),
        )
}

fn movers_empty_state(theme: &Theme) -> impl IntoElement {
    h_flex()
        .border_b_1()
        .border_color(theme.table_row_border)
        .px_4()
        .py_5()
        .child(
            div()
                .flex_1()
                .text_color(theme.muted_foreground)
                .text_size(px(13.))
                .child("No mover rows returned from the MeroAlpha Data API."),
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
        .child(table_cell("ACTION", 0.7, theme.table_head_foreground, true))
}

fn mover_row(mover: &MarketMover, theme: &Theme) -> impl IntoElement {
    let tone = tone_for_signed_value(theme, if mover.positive { 1.0 } else { -1.0 });

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
        .child(
            div()
                .flex_grow(0.7)
                .flex_basis(px(70.))
                .flex()
                .justify_end()
                .child(Icon::new(IconName::Plus).xsmall().text_color(tone.accent)),
        )
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
