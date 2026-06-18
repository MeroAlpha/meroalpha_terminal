use gpui::{IntoElement, ParentElement, Styled, div, px};
use gpui_component::{IconName, Theme, h_flex, v_flex};

use crate::portfolio::PortfolioSnapshot;

use crate::components::theme::{
    elevated_panel, format_money, icon_badge, signed_money, tone_for_signed_value,
};

/// Renders the three KPI summary cards at the top of the portfolio page.
pub fn render_kpis(portfolio: &PortfolioSnapshot, theme: &Theme) -> impl IntoElement {
    let top = portfolio
        .top_holding_symbol
        .clone()
        .unwrap_or_else(|| "—".to_string());

    let top_value = portfolio
        .positions
        .first()
        .map(|p| format_money(p.market_value))
        .unwrap_or_else(|| "NPR 0.00".to_string());

    h_flex()
        .gap_4()
        .flex_wrap()
        .child(kpi_card(
            "Total Value",
            format_money(portfolio.total_value),
            format!("Cost basis: {}", format_money(portfolio.total_cost)),
            IconName::Building2,
            true,
            theme,
        ))
        .child(kpi_card(
            "Unrealized P/L",
            signed_money(portfolio.unrealized_pl),
            format!("{:+.2}% All Time", portfolio.unrealized_pl_pct),
            IconName::ArrowUp,
            portfolio.unrealized_pl >= 0.0,
            theme,
        ))
        .child(kpi_card(
            "Top Holding",
            top,
            format!("Value: {}", top_value),
            IconName::Star,
            true,
            theme,
        ))
}

fn kpi_card(
    label: &'static str,
    value: impl Into<gpui::SharedString>,
    footer: impl Into<gpui::SharedString>,
    icon: IconName,
    positive: bool,
    theme: &Theme,
) -> impl IntoElement {
    let footer = footer.into();
    let tone = if positive {
        tone_for_signed_value(theme, 1.0)
    } else {
        tone_for_signed_value(theme, -1.0)
    };

    elevated_panel(theme)
        .h(px(132.))
        .min_w(px(260.))
        .flex_1()
        .justify_between()
        .child(
            h_flex()
                .justify_between()
                .items_start()
                .child(
                    div()
                        .text_color(theme.muted_foreground)
                        .text_size(px(12.))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .child(label),
                )
                .child(icon_badge(icon, tone)),
        )
        .child(
            v_flex()
                .gap_2()
                .child(
                    div()
                        .text_color(tone.accent)
                        .text_size(px(25.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .whitespace_nowrap()
                        .child(value.into()),
                )
                .child(
                    div()
                        .text_color(if positive {
                            theme.muted_foreground
                        } else {
                            tone.accent
                        })
                        .text_size(px(13.))
                        .child(footer),
                )
                .child(div().w(px(42.)).h(px(2.)).rounded_full().bg(if positive {
                    theme.muted
                } else {
                    tone.accent
                })),
        )
}
