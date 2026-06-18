use gpui::{IntoElement, ParentElement, Styled, div, px};
use gpui_component::{Theme, h_flex, scroll::ScrollableElement};

use crate::portfolio::{PortfolioPosition, PortfolioSnapshot};

use crate::components::theme::{
    elevated_panel, format_number, section_heading, table_cell, tone_for_signed_value,
};

/// Renders the full holdings ledger panel (header + rows).
pub fn render_holdings_table(portfolio: &PortfolioSnapshot, theme: &Theme) -> impl IntoElement {
    elevated_panel(theme)
        .min_w(px(720.))
        .child(div().mb_5().child(section_heading(
            theme,
            "Holdings Ledger",
            format!("{} positions", portfolio.positions.len()),
        )))
        .child(
            div().overflow_x_scrollbar().child(
                gpui_component::v_flex()
                    .min_w(px(760.))
                    .child(table_header(theme))
                    .children(
                        portfolio
                            .positions
                            .iter()
                            .map(|position| render_position_row(position, theme))
                            .collect::<Vec<_>>(),
                    ),
            ),
        )
}

fn table_header(theme: &Theme) -> impl IntoElement {
    h_flex()
        .rounded_t(px(6.))
        .bg(theme.table_head)
        .border_b_1()
        .border_color(theme.table_row_border)
        .px_4()
        .py_2()
        .child(table_cell(
            "TICKER",
            1.4,
            theme.table_head_foreground,
            false,
        ))
        .child(table_cell("QTY", 1.0, theme.table_head_foreground, true))
        .child(table_cell(
            "AVG COST",
            1.0,
            theme.table_head_foreground,
            true,
        ))
        .child(table_cell("LTP", 1.0, theme.table_head_foreground, true))
        .child(table_cell(
            "MARKET VALUE",
            1.4,
            theme.table_head_foreground,
            true,
        ))
        .child(table_cell("WEIGHT", 0.8, theme.table_head_foreground, true))
        .child(table_cell("P/L %", 1.0, theme.table_head_foreground, true))
}

fn render_position_row(position: &PortfolioPosition, theme: &Theme) -> impl IntoElement {
    let tone = tone_for_signed_value(theme, position.unrealized_pl);

    h_flex()
        .border_b_1()
        .border_color(theme.table_row_border)
        .px_4()
        .py_4()
        .child(table_cell(
            position.symbol.clone(),
            1.4,
            theme.foreground,
            false,
        ))
        .child(table_cell(
            format!("{:.0}", position.quantity),
            1.0,
            theme.foreground,
            true,
        ))
        .child(table_cell(
            format!("{:.2}", position.avg_cost),
            1.0,
            theme.muted_foreground,
            true,
        ))
        .child(table_cell(
            format!("{:.2}", position.ltp),
            1.0,
            theme.foreground,
            true,
        ))
        .child(table_cell(
            format_number(position.market_value),
            1.4,
            theme.foreground,
            true,
        ))
        .child(table_cell(
            format!("{:.1}%", position.weight_pct),
            0.8,
            theme.muted_foreground,
            true,
        ))
        .child(table_cell(
            format!("{:+.2}%", position.unrealized_pl_pct),
            1.0,
            tone.accent,
            true,
        ))
}
