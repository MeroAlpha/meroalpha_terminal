use gpui::{IntoElement, ParentElement, Styled, div, px};
use gpui_component::{
    Disableable, Icon, IconName, Sizable, Theme,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputState},
};

pub fn render_top_bar(
    theme: &Theme,
    search_input: &gpui::Entity<InputState>,
    refreshing_overview: bool,
    on_refresh_overview: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
    on_notifications: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
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
                .text_color(theme.foreground)
                .text_size(px(24.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .whitespace_nowrap()
                .child("MeroAlpha Terminal"),
        )
        .child(
            h_flex()
                .gap_3()
                .flex_shrink_0()
                .child(
                    Input::new(search_input)
                        .w(px(260.))
                        .prefix(Icon::new(IconName::Search).xsmall()),
                )
                .child(
                    Button::new("terminal-refresh-overview")
                        .ghost()
                        .small()
                        .w(px(34.))
                        .h(px(34.))
                        .icon(IconName::Redo)
                        .tooltip(if refreshing_overview {
                            "Refreshing Overview"
                        } else {
                            "Refresh Overview"
                        })
                        .disabled(refreshing_overview)
                        .on_click(on_refresh_overview),
                )
                .child(
                    Button::new("terminal-notifications")
                        .ghost()
                        .small()
                        .w(px(34.))
                        .h(px(34.))
                        .icon(IconName::Bell)
                        .tooltip("Notifications")
                        .on_click(on_notifications),
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
