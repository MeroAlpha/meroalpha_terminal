use gpui::{IntoElement, ParentElement, Styled, div, px};
use gpui_component::{
    Disableable, IconName, Sizable, Theme,
    button::{Button, ButtonVariants},
    h_flex,
};

pub fn render_top_bar(
    theme: &Theme,
    api_key_configured: bool,
    refreshing_overview: bool,
    on_refresh_overview: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
    on_settings: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    h_flex()
        .h(px(58.))
        .px_6()
        .gap_4()
        .justify_between()
        .border_b_1()
        .border_color(theme.border)
        .bg(theme.background)
        .child(
            div()
                .min_w(px(0.))
                .text_color(theme.foreground)
                .text_size(px(24.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .whitespace_nowrap()
                .truncate()
                .child("MeroAlpha Terminal"),
        )
        .child(
            h_flex()
                .gap_3()
                .flex_shrink_0()
                .child(
                    Button::new("terminal-refresh-overview")
                        .ghost()
                        .small()
                        .w(px(34.))
                        .h(px(34.))
                        .icon(IconName::Redo)
                        .tooltip(if !api_key_configured {
                            "Set API key to refresh"
                        } else if refreshing_overview {
                            "Refreshing Overview"
                        } else {
                            "Refresh Overview"
                        })
                        .disabled(refreshing_overview || !api_key_configured)
                        .on_click(on_refresh_overview),
                )
                .child(
                    Button::new("terminal-settings")
                        .ghost()
                        .small()
                        .w(px(34.))
                        .h(px(34.))
                        .icon(IconName::Settings)
                        .tooltip("Settings")
                        .on_click(on_settings),
                ),
        )
}
