use gpui::{IntoElement, ParentElement, Styled, div, px};
use gpui_component::{
    Icon, IconName, Sizable, Theme,
    button::{Button, ButtonVariants},
    v_flex,
};

/// Renders the empty-state view shown when no holdings have been imported yet.
///
/// `on_import` is an `on_click` handler produced by `cx.listener(...)` in the
/// parent view; it triggers the native file picker.
pub fn render_empty_state(
    theme: &Theme,
    on_import: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    v_flex()
        .size_full()
        .items_center()
        .justify_center()
        .bg(theme.background)
        .gap_5()
        .child(
            v_flex()
                .w(px(420.))
                .items_center()
                .gap_5()
                .rounded(theme.radius)
                .border_1()
                .border_color(theme.border)
                .bg(theme.secondary)
                .p_8()
                .child(
                    div()
                        .size(px(52.))
                        .rounded(theme.radius_lg)
                        .bg(theme.background)
                        .border_1()
                        .border_color(theme.border)
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(Icon::new(IconName::Inbox).large().text_color(theme.primary)),
                )
                .child(
                    div()
                        .text_color(theme.foreground)
                        .text_size(px(22.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child("Import your portfolio"),
                )
                .child(
                    div()
                        .text_color(theme.muted_foreground)
                        .text_size(px(14.))
                        .text_align(gpui::TextAlign::Center)
                        .child("Export your holdings from MeroShare and upload the CSV file."),
                )
                .child(
                    div()
                        .text_color(theme.muted_foreground)
                        .text_size(px(12.))
                        .child("Portfolio → My Shares → Export (CSV)"),
                )
                .child(
                    Button::new("import-csv-empty")
                        .primary()
                        .icon(IconName::Plus)
                        .label("Import MeroShare CSV")
                        .on_click(on_import),
                ),
        )
}
