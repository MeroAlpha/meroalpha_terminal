use gpui::prelude::FluentBuilder;
use gpui::{IntoElement, ParentElement, Styled, div, px, InteractiveElement as _};
use gpui_component::{
    Icon, IconName, Selectable, Sizable, Theme,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputState},
    v_flex,
};

type ClickHandler = Box<dyn Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static>;

pub struct SidebarNavItem {
    pub id: &'static str,
    pub icon: IconName,
    pub label: &'static str,
    pub active: bool,
    pub on_click: ClickHandler,
}

/// Renders the application sidebar with logo, navigation, and user footer.
#[allow(clippy::too_many_arguments)]
pub fn render_sidebar(
    theme: &Theme,
    nav_items: Vec<SidebarNavItem>,
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
                                        .child("MeroAlpha Terminal"),
                                )
                                .child(
                                    div()
                                        .text_color(theme.sidebar_primary)
                                        .text_size(px(10.))
                                        .child("NEPSE MARKET"),
                                ),
                        ),
                )
                .child(
                    // ── Navigation ────────────────────────────────────────
                    v_flex()
                        .gap_1()
                        .children(nav_items.into_iter().map(|item| nav_item(item, theme))),
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
                                                .tooltip(api_key_visibility_tooltip(
                                                    api_key_visible,
                                                ))
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

fn api_key_visibility_tooltip(visible: bool) -> &'static str {
    if visible {
        "Hide API key"
    } else {
        "Show API key"
    }
}

fn nav_item(item: SidebarNavItem, theme: &Theme) -> impl IntoElement {
    let active = item.active;

    Button::new(item.id)
        .ghost()
        .selected(active)
        .w_full()
        .justify_start()
        .on_click(item.on_click)
        .child(
            h_flex()
                .w_full()
                .gap_3()
                .rounded(theme.radius)
                .px_4()
                .py_3()
                .bg(if active {
                    theme.sidebar_accent
                } else {
                    theme.sidebar
                })
                .when(!active, |el| {
                    el.hover(|style| style.bg(theme.sidebar_accent.opacity(0.4)))
                })
                .border_l_2()
                .border_color(if active {
                    theme.sidebar_primary
                } else {
                    theme.sidebar
                })
                .child(Icon::new(item.icon).text_color(if active {
                    theme.sidebar_primary
                } else {
                    theme.sidebar_foreground
                }))
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.))
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
                        .truncate()
                        .child(item.label),
                ),
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

    #[test]
    fn api_key_visibility_tooltip_matches_current_visibility() {
        assert_eq!(api_key_visibility_tooltip(true), "Hide API key");
        assert_eq!(api_key_visibility_tooltip(false), "Show API key");
    }
}
