// ── Shared helpers ────────────────────────────────────────────────────────────
use gpui::prelude::FluentBuilder;
use gpui::{Hsla, IntoElement, ParentElement, Styled, div, px, transparent_white};
use gpui_component::{
    Colorize, Icon, IconName, Sizable, Theme, alert::Alert, badge::Badge, h_flex,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tone {
    pub accent: Hsla,
    pub foreground: Hsla,
    pub surface: Hsla,
    pub border: Hsla,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    Success,
    Warning,
    Error,
    Info,
}

pub fn tone_for_signed_value(theme: &Theme, value: f64) -> Tone {
    if value >= 0.0 {
        Tone {
            accent: theme.success,
            foreground: theme.success_foreground,
            surface: theme.success.mix_oklab(transparent_white(), 0.04),
            border: theme.success.mix_oklab(transparent_white(), 0.3),
        }
    } else {
        Tone {
            accent: theme.danger,
            foreground: theme.danger_foreground,
            surface: theme.danger.mix_oklab(transparent_white(), 0.04),
            border: theme.danger.mix_oklab(transparent_white(), 0.3),
        }
    }
}

pub fn status_tone(theme: &Theme, kind: StatusKind) -> Tone {
    match kind {
        StatusKind::Success => Tone {
            accent: theme.success,
            foreground: theme.success_foreground,
            surface: theme.success.mix_oklab(transparent_white(), 0.04),
            border: theme.success.mix_oklab(transparent_white(), 0.3),
        },
        StatusKind::Warning => Tone {
            accent: theme.warning,
            foreground: theme.warning_foreground,
            surface: theme.warning.mix_oklab(transparent_white(), 0.04),
            border: theme.warning.mix_oklab(transparent_white(), 0.3),
        },
        StatusKind::Error => Tone {
            accent: theme.danger,
            foreground: theme.danger_foreground,
            surface: theme.danger.mix_oklab(transparent_white(), 0.04),
            border: theme.danger.mix_oklab(transparent_white(), 0.3),
        },
        StatusKind::Info => Tone {
            accent: theme.info,
            foreground: theme.info_foreground,
            surface: theme.info.mix_oklab(transparent_white(), 0.04),
            border: theme.info.mix_oklab(transparent_white(), 0.3),
        },
    }
}

/// A standard card / panel surface with a border.
pub fn panel(theme: &Theme) -> gpui::Div {
    div()
        .rounded(theme.radius)
        .border_1()
        .border_color(theme.border)
        .bg(theme.background)
        .p_5()
}

pub fn elevated_panel(theme: &Theme) -> gpui::Div {
    panel(theme)
        .bg(theme.secondary)
        .border_color(theme.border)
        .shadow_xs()
}

/// A bold section heading.
pub fn section_title(theme: &Theme, title: &'static str) -> impl IntoElement {
    div()
        .text_color(theme.foreground)
        .text_size(px(18.))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .child(title)
}

pub fn section_heading(
    theme: &Theme,
    title: impl Into<gpui::SharedString>,
    detail: impl Into<gpui::SharedString>,
) -> impl IntoElement {
    h_flex()
        .flex_1()
        .min_w(px(0.))
        .gap_3()
        .justify_between()
        .items_center()
        .child(
            div()
                .flex_1()
                .min_w(px(0.))
                .text_color(theme.foreground)
                .text_size(px(18.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .truncate()
                .child(title.into()),
        )
        .child(
            div()
                .flex_shrink_0()
                .child(Badge::new().child(detail.into()).color(theme.muted)),
        )
}

pub fn icon_badge(icon: IconName, tone: Tone) -> impl IntoElement {
    div()
        .size(px(28.))
        .rounded(px(7.))
        .border_1()
        .border_color(tone.border)
        .bg(tone.surface)
        .flex()
        .items_center()
        .justify_center()
        .child(Icon::new(icon).small().text_color(tone.accent))
}

pub fn status_strip(
    kind: StatusKind,
    label: impl Into<gpui::SharedString>,
    message: impl Into<gpui::SharedString>,
) -> impl IntoElement {
    let id = label.into();
    let message = message.into();
    match kind {
        StatusKind::Success => Alert::success(id.clone(), message).title(id),
        StatusKind::Warning => Alert::warning(id.clone(), message).title(id),
        StatusKind::Error => Alert::error(id.clone(), message).title(id),
        StatusKind::Info => Alert::info(id.clone(), message).title(id),
    }
}

/// A table cell with configurable flex-grow and text colour.
pub fn table_cell(
    content: impl Into<gpui::SharedString>,
    grow: f32,
    color: Hsla,
    align_right: bool,
) -> impl IntoElement {
    div()
        .flex_grow(grow)
        .flex_basis(px(100. * grow))
        .min_w(px(0.))
        .text_color(color)
        .text_size(px(13.))
        .whitespace_nowrap()
        .truncate()
        .when(align_right, |el| {
            el.text_align(gpui::TextAlign::Right).justify_end()
        })
        .child(content.into())
}

// ── Number formatting ─────────────────────────────────────────────────────────

pub fn format_money(value: f64) -> String {
    format!("NPR {}", format_number(value))
}

pub fn signed_money(value: f64) -> String {
    if value >= 0.0 {
        format!("+{}", format_money(value))
    } else {
        format!("-{}", format_money(value.abs()))
    }
}

pub fn format_number(value: f64) -> String {
    let rounded = format!("{:.2}", value.abs());
    let (whole, decimal) = rounded.split_once('.').unwrap_or((rounded.as_str(), "00"));
    let mut out = String::new();
    for (ix, ch) in whole.chars().rev().enumerate() {
        if ix > 0 && ix % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    let whole = out.chars().rev().collect::<String>();
    if value < 0.0 {
        format!("-{}.{}", whole, decimal)
    } else {
        format!("{}.{}", whole, decimal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::rgb;
    use gpui_component::ThemeColor;

    #[test]
    fn tone_for_signed_value_maps_financial_direction() {
        let theme = test_theme();
        assert_eq!(tone_for_signed_value(&theme, 12.0).accent, theme.success);
        assert_eq!(tone_for_signed_value(&theme, 0.0).accent, theme.success);
        assert_eq!(tone_for_signed_value(&theme, -0.01).accent, theme.danger);
    }

    #[test]
    fn status_tone_uses_distinct_surface_and_border_colors() {
        let theme = test_theme();
        let success = status_tone(&theme, StatusKind::Success);
        let warning = status_tone(&theme, StatusKind::Warning);
        let error = status_tone(&theme, StatusKind::Error);

        assert_ne!(success.surface, warning.surface);
        assert_ne!(warning.surface, error.surface);
        assert_ne!(success.border, error.border);
    }

    fn test_theme() -> Theme {
        Theme::from(&ThemeColor {
            success: rgb(0x22C55E).into(),
            success_foreground: rgb(0x052E16).into(),
            warning: rgb(0xF59E0B).into(),
            warning_foreground: rgb(0x451A03).into(),
            danger: rgb(0xEF4444).into(),
            danger_foreground: rgb(0x450A0A).into(),
            info: rgb(0x3B82F6).into(),
            info_foreground: rgb(0x172554).into(),
            ..ThemeColor::default()
        })
    }
}
