use gpui::{IntoElement, ParentElement, Styled, div, px};
use gpui_component::{Theme, h_flex, scroll::ScrollableElement, v_flex};

use crate::portfolio::PortfolioSnapshot;

use crate::components::theme::{
    elevated_panel, format_money, section_heading, tone_for_signed_value,
};

/// Renders the right-rail panel: portfolio allocation summary.
///
/// Renders a real allocation breakdown derived from the live portfolio snapshot.
pub fn render_right_rail(portfolio: &PortfolioSnapshot, theme: &Theme) -> impl IntoElement {
    v_flex()
        .w(px(340.))
        .max_w(px(420.))
        .h_full()
        .flex_shrink_0()
        .gap_5()
        .overflow_y_scrollbar()
        .child(allocation_panel(portfolio, theme))
        .child(summary_panel(portfolio, theme))
}

fn allocation_panel(portfolio: &PortfolioSnapshot, theme: &Theme) -> impl IntoElement {
    elevated_panel(theme)
        .child(div().mb_6().child(section_heading(
            theme,
            "Allocation",
            format!("{} positions", portfolio.positions.len()),
        )))
        .child(
            v_flex()
                .gap_4()
                .max_h(px(280.))
                .overflow_y_scrollbar()
                .children(portfolio.positions.iter().map(|position| {
                    let tone = tone_for_signed_value(theme, position.unrealized_pl);

                    v_flex()
                        .gap_2()
                        .child(
                            h_flex()
                                .justify_between()
                                .child(
                                    div()
                                        .text_color(theme.foreground)
                                        .text_size(px(13.))
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .child(position.symbol.clone()),
                                )
                                .child(
                                    h_flex()
                                        .gap_3()
                                        .child(
                                            div().text_color(tone.accent).text_size(px(12.)).child(
                                                format!("{:+.2}%", position.unrealized_pl_pct),
                                            ),
                                        )
                                        .child(
                                            div()
                                                .text_color(theme.muted_foreground)
                                                .text_size(px(12.))
                                                .child(format!("{:.1}%", position.weight_pct)),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .h(px(5.))
                                .w_full()
                                .rounded_full()
                                .bg(theme.progress_bar)
                                .child(div().h_full().rounded_full().bg(tone.accent).w(
                                    gpui::DefiniteLength::Fraction(
                                        (position.weight_pct / 100.0).clamp(0.0, 1.0) as f32,
                                    ),
                                )),
                        )
                })),
        )
}

fn summary_panel(portfolio: &PortfolioSnapshot, theme: &Theme) -> impl IntoElement {
    let positive = portfolio.unrealized_pl >= 0.0;
    let pnl_tone = tone_for_signed_value(theme, portfolio.unrealized_pl);

    elevated_panel(theme)
        .child(
            h_flex()
                .mb_5()
                .child(section_heading(theme, "Summary", "local")),
        )
        .child(summary_row(
            theme,
            "Total Cost",
            format_money(portfolio.total_cost),
            theme.muted_foreground,
        ))
        .child(summary_row(
            theme,
            "Market Value",
            format_money(portfolio.total_value),
            theme.foreground,
        ))
        .child(summary_row(
            theme,
            "Unrealised P/L",
            format!(
                "{} ({:+.2}%)",
                if positive {
                    format!("+{}", format_money(portfolio.unrealized_pl))
                } else {
                    format!("-{}", format_money(portfolio.unrealized_pl.abs()))
                },
                portfolio.unrealized_pl_pct
            ),
            pnl_tone.accent,
        ))
}

fn summary_row(
    theme: &Theme,
    label: &'static str,
    value: impl Into<gpui::SharedString>,
    value_color: gpui::Hsla,
) -> impl IntoElement {
    h_flex()
        .justify_between()
        .border_t_1()
        .border_color(theme.border)
        .py_3()
        .gap_4()
        .child(
            div()
                .flex_shrink_0()
                .text_color(theme.muted_foreground)
                .text_size(px(12.))
                .child(label),
        )
        .child(
            div()
                .flex_1()
                .text_align(gpui::TextAlign::Right)
                .text_color(value_color)
                .text_size(px(13.))
                .font_weight(gpui::FontWeight::MEDIUM)
                .whitespace_nowrap()
                .child(value.into()),
        )
}
