mod app;

use gpui::{
    Action, App, AppContext, Menu, MenuItem, SharedString, TitlebarOptions, WindowBounds,
    WindowOptions, actions, px, size,
};
use gpui_component::{ActiveTheme as _, Root, Theme, ThemeMode, ThemeRegistry};
use serde::Deserialize;

use app::MeroAlphaTerminal;

const BUNDLED_THEMES: &[&str] = &[
    include_str!("../assets/themes/adventure.json"),
    include_str!("../assets/themes/alduin.json"),
    include_str!("../assets/themes/asciinema.json"),
    include_str!("../assets/themes/ayu.json"),
    include_str!("../assets/themes/catppuccin.json"),
    include_str!("../assets/themes/everforest.json"),
    include_str!("../assets/themes/fahrenheit.json"),
    include_str!("../assets/themes/flexoki.json"),
    include_str!("../assets/themes/gruvbox.json"),
    include_str!("../assets/themes/harper.json"),
    include_str!("../assets/themes/hybrid.json"),
    include_str!("../assets/themes/jellybeans.json"),
    include_str!("../assets/themes/kibble.json"),
    include_str!("../assets/themes/macos-classic.json"),
    include_str!("../assets/themes/matrix.json"),
    include_str!("../assets/themes/mellifluous.json"),
    include_str!("../assets/themes/molokai.json"),
    include_str!("../assets/themes/solarized.json"),
    include_str!("../assets/themes/spaceduck.json"),
    include_str!("../assets/themes/tokyonight.json"),
    include_str!("../assets/themes/twilight.json"),
];

actions!(meroalpha_terminal, [Quit]);

#[derive(Action, Clone, PartialEq, Deserialize)]
#[action(namespace = meroalpha_terminal, no_json)]
struct SwitchTheme(SharedString);

#[derive(Action, Clone, PartialEq, Deserialize)]
#[action(namespace = meroalpha_terminal, no_json)]
struct SwitchThemeMode(ThemeMode);

fn main() {
    gpui_platform::application()
        .with_assets(gpui_component_assets::Assets)
        .run(|cx: &mut App| {
            gpui_component::init(cx);
            load_bundled_themes(cx);
            Theme::change(ThemeMode::Dark, None, cx);
            install_menu_actions(cx);
            set_app_menus(cx);
            cx.activate(true);

            let window_options = terminal_window_options(cx);
            cx.spawn(async move |cx| {
                cx.open_window(window_options, |window, cx| {
                    let view = cx.new(|cx| MeroAlphaTerminal::new(window, cx));
                    cx.new(|cx| Root::new(view, window, cx))
                })
                .expect("failed to open MeroAlpha Terminal window");
            })
            .detach();
        });
}

fn terminal_window_options(cx: &App) -> WindowOptions {
    WindowOptions {
        titlebar: Some(TitlebarOptions {
            title: Some("MeroAlpha Terminal".into()),
            ..Default::default()
        }),
        window_bounds: Some(WindowBounds::centered(size(px(1280.), px(820.)), cx)),
        window_min_size: Some(size(px(800.), px(600.))),
        ..WindowOptions::default()
    }
}

fn load_bundled_themes(cx: &mut App) {
    let registry = ThemeRegistry::global_mut(cx);
    for theme in BUNDLED_THEMES {
        if let Err(error) = registry.load_themes_from_str(theme) {
            eprintln!("failed to load bundled theme: {error}");
        }
    }
}

fn install_menu_actions(cx: &mut App) {
    cx.on_action(|switch: &SwitchTheme, cx| {
        let theme_name = switch.0.clone();
        if let Some(theme_config) = ThemeRegistry::global(cx).themes().get(&theme_name).cloned() {
            Theme::global_mut(cx).apply_config(&theme_config);
        }
        set_app_menus(cx);
        cx.refresh_windows();
    });

    cx.on_action(|switch: &SwitchThemeMode, cx| {
        Theme::change(switch.0, None, cx);
        set_app_menus(cx);
        cx.refresh_windows();
    });

    cx.on_action(|_: &Quit, cx| cx.quit());
}

fn set_app_menus(cx: &mut App) {
    cx.set_menus(build_menus(cx));
}

fn build_menus(cx: &App) -> Vec<Menu> {
    vec![
        Menu {
            name: "MeroAlpha Terminal".into(),
            items: vec![
                MenuItem::Submenu(Menu {
                    name: "Appearance".into(),
                    items: vec![
                        MenuItem::action("Light", SwitchThemeMode(ThemeMode::Light))
                            .checked(!cx.theme().mode.is_dark()),
                        MenuItem::action("Dark", SwitchThemeMode(ThemeMode::Dark))
                            .checked(cx.theme().mode.is_dark()),
                    ],
                    disabled: false,
                }),
                theme_menu(cx),
                MenuItem::Separator,
                MenuItem::action("Quit", Quit),
            ],
            disabled: false,
        },
        Menu {
            name: "Edit".into(),
            items: vec![
                MenuItem::action("Undo", gpui_component::input::Undo),
                MenuItem::action("Redo", gpui_component::input::Redo),
                MenuItem::separator(),
                MenuItem::action("Cut", gpui_component::input::Cut),
                MenuItem::action("Copy", gpui_component::input::Copy),
                MenuItem::action("Paste", gpui_component::input::Paste),
                MenuItem::separator(),
                MenuItem::action("Delete", gpui_component::input::Delete),
                MenuItem::action(
                    "Delete Previous Word",
                    gpui_component::input::DeleteToPreviousWordStart,
                ),
                MenuItem::action(
                    "Delete Next Word",
                    gpui_component::input::DeleteToNextWordEnd,
                ),
                MenuItem::separator(),
                MenuItem::action("Find", gpui_component::input::Search),
                MenuItem::separator(),
                MenuItem::action("Select All", gpui_component::input::SelectAll),
            ],
            disabled: false,
        },
    ]
}

fn theme_menu(cx: &App) -> MenuItem {
    let current_name = cx.theme().theme_name();
    MenuItem::Submenu(Menu {
        name: "Theme".into(),
        items: ThemeRegistry::global(cx)
            .sorted_themes()
            .iter()
            .map(|theme| {
                MenuItem::action(theme.name.clone(), SwitchTheme(theme.name.clone()))
                    .checked(current_name == &theme.name)
            })
            .collect(),
        disabled: false,
    })
}
