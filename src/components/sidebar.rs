use gpui::prelude::FluentBuilder;
use gpui::{IntoElement, ParentElement, Styled, div, px};
use gpui_component::{
    Icon, IconName, Sizable, Theme,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputState},
    v_flex,
};

/// Renders the application sidebar with logo, navigation, and user footer.
///
/// `active_route` is a string key matching the current page so the correct
/// nav item can be highlighted (e.g. `"portfolio"`).
pub fn render_sidebar(
    theme: &Theme,
    active_route: &str,
    profile_name: &str,
    api_key_configured: bool,
    settings_open: bool,
    profile_name_input: &gpui::Entity<InputState>,
    api_key_input: &gpui::Entity<InputState>,
    on_edit_settings: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
    api_key_visible: bool,
    on_toggle_api_key_visibility: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App)
    + 'static,
    on_save_settings: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
    on_cancel_settings: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    v_flex()
        .w(px(280.))
        .h_full()
        .flex_shrink_0()
        .justify_between()
        .bg(theme.sidebar)
        .border_r_1()
        .border_color(theme.sidebar_border)
        .p_6()
        .child(
            v_flex()
                .gap_10()
                .child(
                    // ── Logo ──────────────────────────────────────────────
                    h_flex()
                        .gap_3()
                        .child(
                            Icon::new(IconName::SquareTerminal).text_color(theme.sidebar_primary),
                        )
                        .child(
                            v_flex()
                                .child(
                                    div()
                                        .text_color(theme.sidebar_foreground)
                                        .text_size(px(18.))
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .child("NEPSE Terminal"),
                                )
                                .child(
                                    div()
                                        .text_color(theme.sidebar_primary)
                                        .text_size(px(10.))
                                        .child("MEROALPHA"),
                                ),
                        ),
                )
                .child(
                    // ── Navigation ────────────────────────────────────────
                    v_flex().gap_1().child(nav_item(
                        IconName::HardDrive,
                        "Portfolio",
                        active_route == "portfolio",
                        theme,
                    )),
                ),
        )
        .child(
            // ── User footer ───────────────────────────────────────────────
            v_flex()
                .gap_3()
                .border_t_1()
                .border_color(theme.sidebar_border)
                .pt_4()
                .child(profile_summary(theme, profile_name, api_key_configured))
                .when(!settings_open, |el| {
                    el.child(
                        Button::new("edit-local-profile")
                            .small()
                            .icon(IconName::Settings)
                            .label("Profile & API")
                            .on_click(on_edit_settings),
                    )
                })
                .when(settings_open, |el| {
                    el.child(
                        v_flex()
                            .gap_3()
                            .child(
                                v_flex()
                                    .gap_1()
                                    .child(field_label(theme, "Profile name"))
                                    .child(Input::new(profile_name_input).cleanable(true)),
                            )
                            .child(
                                v_flex()
                                    .gap_1()
                                    .child(field_label(theme, "MeroAlpha API key"))
                                    .child(
                                        Input::new(api_key_input).suffix(
                                            Button::new("toggle-api-key-visibility")
                                                .xsmall()
                                                .ghost()
                                                .icon(api_key_visibility_icon(api_key_visible))
                                                .on_click(on_toggle_api_key_visibility),
                                        ),
                                    ),
                            )
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(
                                        Button::new("save-local-settings")
                                            .primary()
                                            .small()
                                            .label("Save")
                                            .on_click(on_save_settings),
                                    )
                                    .child(
                                        Button::new("cancel-local-settings")
                                            .small()
                                            .label("Cancel")
                                            .on_click(on_cancel_settings),
                                    ),
                            ),
                    )
                }),
        )
}

fn profile_summary(
    theme: &Theme,
    profile_name: &str,
    api_key_configured: bool,
) -> impl IntoElement {
    h_flex()
        .gap_3()
        .child(
            div()
                .size(px(32.))
                .rounded_full()
                .bg(theme.sidebar_accent)
                .border_1()
                .border_color(theme.sidebar_border)
                .flex()
                .items_center()
                .justify_center()
                .child(
                    Icon::new(IconName::User)
                        .small()
                        .text_color(theme.sidebar_accent_foreground),
                ),
        )
        .child(
            v_flex()
                .flex_1()
                .child(
                    div()
                        .text_color(theme.sidebar_foreground)
                        .text_size(px(13.))
                        .truncate()
                        .child(profile_name.to_string()),
                )
                .child(
                    div()
                        .text_color(if api_key_configured {
                            theme.success
                        } else {
                            theme.muted_foreground
                        })
                        .text_size(px(11.))
                        .child(if api_key_configured {
                            "API key set"
                        } else {
                            "Local mode"
                        }),
                ),
        )
}

fn field_label(theme: &Theme, label: &'static str) -> impl IntoElement {
    div()
        .text_color(theme.muted_foreground)
        .text_size(px(11.))
        .child(label)
}

fn api_key_visibility_icon(visible: bool) -> IconName {
    if visible {
        IconName::Eye
    } else {
        IconName::EyeOff
    }
}

fn nav_item(icon: IconName, label: &'static str, active: bool, theme: &Theme) -> impl IntoElement {
    h_flex()
        .gap_3()
        .rounded(theme.radius)
        .px_4()
        .py_3()
        .bg(if active {
            theme.sidebar_accent
        } else {
            theme.sidebar
        })
        .border_l_2()
        .border_color(if active {
            theme.sidebar_primary
        } else {
            theme.sidebar
        })
        .child(Icon::new(icon).text_color(if active {
            theme.sidebar_primary
        } else {
            theme.muted_foreground
        }))
        .child(
            div()
                .flex_1()
                .text_color(if active {
                    theme.sidebar_accent_foreground
                } else {
                    theme.sidebar_foreground
                })
                .text_size(px(14.))
                .font_weight(if active {
                    gpui::FontWeight::MEDIUM
                } else {
                    gpui::FontWeight::NORMAL
                })
                .child(label),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_key_visibility_icon_matches_current_visibility() {
        assert!(matches!(api_key_visibility_icon(true), IconName::Eye));
        assert!(matches!(api_key_visibility_icon(false), IconName::EyeOff));
    }
}
