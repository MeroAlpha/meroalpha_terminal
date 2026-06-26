use gpui::{
    AnyElement, AppContext, Context, IntoElement, ParentElement, PathPromptOptions, Render, Styled,
    WeakEntity, Window, div, px,
};
use gpui_component::{
    ActiveTheme as _, Disableable, Icon, IconName, Root, Sizable, WindowExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::InputState,
    notification::Notification,
    scroll::ScrollableElement,
    v_flex,
};

use meroalpha_terminal::{
    db::open_app_db,
    meroalpha_api::{MeroAlphaClient, OverviewMarketData, PriceFailure, PriceUpdate},
    portfolio::{
        AppSettings, PortfolioRepository, PortfolioSnapshot, SqlitePortfolioRepository,
        local_ltp_for_unavailable_price, parse_holdings_csv, portfolio_snapshot,
    },
};

use meroalpha_terminal::components::{
    empty_state::render_empty_state,
    holdings_table::render_holdings_table,
    kpi_cards::render_kpis,
    overview::render_overview_page,
    right_rail::{render_right_rail, render_right_rail_responsive},
    sidebar::{SidebarNavItem, render_sidebar},
    theme::{StatusKind, status_tone},
    top_bar::render_top_bar,
};
use meroalpha_terminal::overview::{MoverTab, OverviewSnapshot};

// ── App state ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppNotificationKind {
    Info,
    Success,
    Warning,
    Error,
}

impl AppNotificationKind {
    fn status_kind(self) -> StatusKind {
        match self {
            Self::Info => StatusKind::Info,
            Self::Success => StatusKind::Success,
            Self::Warning => StatusKind::Warning,
            Self::Error => StatusKind::Error,
        }
    }

    fn icon(self) -> IconName {
        match self {
            Self::Info => IconName::Info,
            Self::Success => IconName::CircleCheck,
            Self::Warning => IconName::TriangleAlert,
            Self::Error => IconName::CircleX,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppRoute {
    Overview,
    Portfolio,
}

impl AppRoute {
    fn key(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Portfolio => "portfolio",
        }
    }

    fn sidebar_routes() -> &'static [Self] {
        &[Self::Overview, Self::Portfolio]
    }
}

pub struct MeroAlphaTerminal {
    /// SQLite-backed repository, always valid after new() returns.
    repository: SqlitePortfolioRepository,
    settings: AppSettings,
    profile_name_input: gpui::Entity<InputState>,
    api_key_input: gpui::Entity<InputState>,
    settings_open: bool,
    api_key_visible: bool,
    active_route: AppRoute,
    /// None until the user imports a CSV for the first time.
    snapshot: Option<PortfolioSnapshot>,
    price_refresh_error: Option<String>,
    price_refresh_status: Option<String>,
    refreshing_prices: bool,
    overview_market_data: Option<OverviewMarketData>,
    overview_refresh_error: Option<String>,
    refreshing_overview: bool,
    overview_mover_tab: MoverTab,
}

impl MeroAlphaTerminal {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Open (or create) the on-disk SQLite database.
        let repository = open_app_db().expect("open portfolio database");
        let settings = repository.load_settings();
        let profile_name_input =
            cx.new(|cx| InputState::new(window, cx).default_value(settings.profile_name.clone()));
        let api_key_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("MeroAlpha API key")
                .masked(true)
                .default_value(settings.meroalpha_api_key.clone().unwrap_or_default())
        });
        // Load whatever was persisted from a previous session.
        let snapshot = {
            let s = repository.load_snapshot();
            if s.positions.is_empty() {
                None
            } else {
                Some(s)
            }
        };

        let this = Self {
            repository,
            settings,
            profile_name_input,
            api_key_input,
            settings_open: false,
            api_key_visible: false,
            active_route: AppRoute::Overview,
            snapshot,
            price_refresh_error: None,
            price_refresh_status: None,
            refreshing_prices: false,
            overview_market_data: None,
            overview_refresh_error: None,
            refreshing_overview: false,
            overview_mover_tab: MoverTab::Gainers,
        };

        if configured_api_key(&this.settings).is_some() {
            cx.defer_in(window, |this, window, cx| {
                this.refresh_overview_market_data(window, cx, false);
            });
        }

        this
    }

    fn push_notification(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
        kind: AppNotificationKind,
        title: &'static str,
        message: impl Into<gpui::SharedString>,
    ) {
        let message = message.into();
        let theme = cx.theme().clone();
        let tone = status_tone(&theme, kind.status_kind());
        let content: gpui::SharedString = format!("{title}: {message}").into();
        let notification = Notification::new()
            .message(content)
            .icon(Icon::new(kind.icon()).small().text_color(tone.accent))
            .w(px(360.));

        window.push_notification(notification, cx);
    }

    // ── CSV import ────────────────────────────────────────────────────────────

    /// Opens the native file picker; on selection parses and persists the CSV.
    fn open_csv_picker(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Select MeroShare CSV export".into()),
        });

        cx.spawn_in(
            window,
            async move |this: WeakEntity<MeroAlphaTerminal>, cx| {
                // Await the platform file-picker result.
                // Shape: Result<Result<Option<Vec<PathBuf>>, Error>, Canceled>
                let paths = match receiver.await {
                    Ok(Ok(Some(paths))) => paths,
                    // Cancelled or any error → do nothing.
                    _ => return,
                };

                let path = match paths.into_iter().next() {
                    Some(p) => p,
                    None => return,
                };

                let _ = this.update_in(cx, |this, window, cx| {
                    this.push_notification(
                        window,
                        cx,
                        AppNotificationKind::Info,
                        "Importing holdings",
                        "Reading the selected CSV file.",
                    );
                });

                // Read the CSV file (potentially large) off the main thread.
                let read_result: std::io::Result<String> = cx
                    .background_executor()
                    .spawn(async move { std::fs::read_to_string(path) })
                    .await;

                let content = match read_result {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = this.update_in(cx, |this, window, cx| {
                            let message = format!("Cannot read file: {e}");
                            this.push_notification(
                                window,
                                cx,
                                AppNotificationKind::Error,
                                "Import failed",
                                message,
                            );
                            cx.notify();
                        });
                        return;
                    }
                };

                // Parse and persist on the main thread.
                let _ = this.update_in(cx, |this, window, cx| {
                    match parse_holdings_csv(&content) {
                        Ok(holdings) => {
                            let position_count = holdings.len();
                            this.repository.replace_holdings(holdings.clone());
                            this.snapshot = Some(portfolio_snapshot(&holdings));
                            this.push_notification(
                                window,
                                cx,
                                AppNotificationKind::Success,
                                "Holdings imported",
                                format!("Imported {position_count} holdings from CSV."),
                            );
                        }
                        Err(e) => {
                            let message = format!("CSV parse error: {:?}", e);
                            this.push_notification(
                                window,
                                cx,
                                AppNotificationKind::Error,
                                "Import failed",
                                message,
                            );
                        }
                    }
                    cx.notify();
                });
            },
        )
        .detach();
    }

    // ── MeroAlpha price refresh ───────────────────────────────────────────────

    fn refresh_prices(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(api_key) = configured_api_key(&self.settings) else {
            let message = "Set your MeroAlpha API key first.";
            self.price_refresh_error = Some(message.to_string());
            self.push_notification(
                window,
                cx,
                AppNotificationKind::Warning,
                "Price refresh",
                message,
            );
            cx.notify();
            return;
        };

        let symbols = self
            .snapshot
            .as_ref()
            .map(|snapshot| {
                snapshot
                    .positions
                    .iter()
                    .map(|position| position.symbol.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if symbols.is_empty() {
            let message = "Import holdings before refreshing prices.";
            self.price_refresh_error = Some(message.to_string());
            self.push_notification(
                window,
                cx,
                AppNotificationKind::Warning,
                "Price refresh",
                message,
            );
            cx.notify();
            return;
        }

        self.refreshing_prices = true;
        self.price_refresh_error = None;
        self.price_refresh_status = Some(format!("Refreshing {} prices...", symbols.len()));
        self.push_notification(
            window,
            cx,
            AppNotificationKind::Info,
            "Fetching prices",
            format!("Fetching latest prices for {} holdings.", symbols.len()),
        );
        cx.notify();

        cx.spawn_in(
            window,
            async move |this: WeakEntity<MeroAlphaTerminal>, cx| {
                let fetch_result = cx
                    .background_executor()
                    .spawn(async move {
                        let client = MeroAlphaClient::new(api_key);
                        let mut updates = Vec::new();
                        let mut failures = Vec::new();

                        for result in client.latest_prices(&symbols) {
                            match result {
                                Ok(update) => updates.push(update),
                                Err(error) => failures.push(error),
                            }
                        }

                        (updates, failures)
                    })
                    .await;

                let _ = this.update_in(cx, |this, window, cx| {
                    this.apply_price_refresh(fetch_result.0, fetch_result.1);
                    let message = this
                        .price_refresh_error
                        .as_ref()
                        .or(this.price_refresh_status.as_ref())
                        .cloned();
                    if let Some(message) = message {
                        let kind = this
                            .price_refresh_error
                            .as_ref()
                            .map(|_| AppNotificationKind::Error)
                            .unwrap_or_else(|| notification_kind_for_status(&message));
                        this.push_notification(window, cx, kind, "Price refresh", message);
                    }
                    cx.notify();
                });
            },
        )
        .detach();
    }

    fn apply_price_refresh(&mut self, updates: Vec<PriceUpdate>, failures: Vec<PriceFailure>) {
        let current_snapshot = self.snapshot.clone();
        let mut unavailable_count = 0;
        let mut local_floor_count = 0;
        let mut hard_failures = Vec::new();
        let mut storage_errors = Vec::new();

        for update in &updates {
            if let Err(error) = self.repository.update_ltp(&update.symbol, update.ltp) {
                storage_errors.push(format!("Could not save {}: {:?}", update.symbol, error));
            }
        }

        for failure in failures {
            if failure.error.is_unavailable_price() {
                unavailable_count += 1;
                let current_ltp = current_snapshot
                    .as_ref()
                    .and_then(|snapshot| {
                        snapshot
                            .positions
                            .iter()
                            .find(|position| position.symbol == failure.symbol)
                    })
                    .map(|position| position.ltp)
                    .unwrap_or(0.0);
                let local_ltp = local_ltp_for_unavailable_price(current_ltp);

                if local_ltp != current_ltp {
                    local_floor_count += 1;
                    if let Err(error) = self.repository.update_ltp(&failure.symbol, local_ltp) {
                        storage_errors
                            .push(format!("Could not save {}: {:?}", failure.symbol, error));
                    }
                }
            } else {
                hard_failures.push(failure);
            }
        }

        self.snapshot = Some(self.repository.load_snapshot());
        self.refreshing_prices = false;

        if !storage_errors.is_empty() {
            self.price_refresh_status = None;
            self.price_refresh_error = Some(storage_errors.join("; "));
        } else if updates.is_empty() && unavailable_count == 0 {
            self.price_refresh_status = None;
            self.price_refresh_error = Some(format!(
                "No prices updated. {}",
                hard_failures
                    .first()
                    .map(|failure| format!("{}: {:?}", failure.symbol, failure.error))
                    .unwrap_or_else(|| "No response data.".to_string())
            ));
        } else if hard_failures.is_empty() {
            self.price_refresh_error = None;
            self.price_refresh_status = Some(price_refresh_status(
                updates.len(),
                unavailable_count,
                local_floor_count,
            ));
        } else {
            self.price_refresh_status = Some(format!(
                "Updated {} prices; kept local valuation for {} unavailable symbols; {} symbols failed.",
                updates.len(),
                unavailable_count,
                hard_failures.len()
            ));
        }
    }

    // ── Overview market data ─────────────────────────────────────────────────

    fn refresh_overview_market_data(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        notify_user: bool,
    ) {
        let Some(api_key) = configured_api_key(&self.settings) else {
            let message = "Set your MeroAlpha API key first.";
            eprintln!("[overview-refresh] skipped: {message}");
            self.overview_market_data = None;
            self.overview_refresh_error = Some(message.to_string());
            if notify_user {
                self.push_notification(
                    window,
                    cx,
                    AppNotificationKind::Warning,
                    "Overview refresh",
                    message,
                );
            }
            cx.notify();
            return;
        };

        eprintln!(
            "[overview-refresh] start notify_user={notify_user} api_key=set existing_market_data={}",
            self.overview_market_data.is_some()
        );
        self.refreshing_overview = true;
        self.overview_refresh_error = None;
        if notify_user {
            self.push_notification(
                window,
                cx,
                AppNotificationKind::Info,
                "Overview refresh",
                "Fetching market overview from MeroAlpha Data API.",
            );
        }
        cx.notify();

        cx.spawn_in(
            window,
            async move |this: WeakEntity<MeroAlphaTerminal>, cx| {
                let fetch_result = cx
                    .background_executor()
                    .spawn(async move {
                        let client = MeroAlphaClient::new(api_key);
                        eprintln!(
                            "[overview-refresh] client base_url={}",
                            client.base_url()
                        );
                        client.overview_market_data()
                    })
                    .await;

                let _ = this.update_in(cx, |this, window, cx| {
                    this.refreshing_overview = false;
                    match fetch_result {
                        Ok(market_data) => {
                            let index_count = market_data.indices.len();
                            let mover_count = market_data
                                .gainers
                                .len()
                                .max(market_data.losers.len())
                                .max(market_data.turnover.len());
                            eprintln!(
                                "[overview-refresh] success indices={index_count} movers={mover_count}"
                            );
                            this.overview_market_data = Some(market_data);
                            this.overview_refresh_error = None;
                            if notify_user {
                                this.push_notification(
                                    window,
                                    cx,
                                    AppNotificationKind::Success,
                                    "Overview refreshed",
                                    "MeroAlpha Data API overview data loaded.",
                                );
                            }
                        }
                        Err(error) => {
                            let message = format!("Could not load Overview: {:?}", error);
                            eprintln!("[overview-refresh] error {message}");
                            this.overview_market_data = None;
                            this.overview_refresh_error = Some(message.clone());
                            if notify_user {
                                this.push_notification(
                                    window,
                                    cx,
                                    AppNotificationKind::Error,
                                    "Overview refresh failed",
                                    message,
                                );
                            }
                        }
                    }
                    cx.notify();
                });
            },
        )
        .detach();
    }

    // ── Local settings ────────────────────────────────────────────────────────

    fn open_settings_editor(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.profile_name_input.update(cx, |input, cx| {
            input.set_value(self.settings.profile_name.clone(), window, cx);
        });
        self.api_key_input.update(cx, |input, cx| {
            input.set_value(
                self.settings.meroalpha_api_key.clone().unwrap_or_default(),
                window,
                cx,
            );
            input.set_masked(true, window, cx);
        });
        self.settings_open = true;
        self.api_key_visible = false;
        cx.notify();
    }

    fn cancel_settings_editor(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.profile_name_input.update(cx, |input, cx| {
            input.set_value(self.settings.profile_name.clone(), window, cx);
        });
        self.api_key_input.update(cx, |input, cx| {
            input.set_value(
                self.settings.meroalpha_api_key.clone().unwrap_or_default(),
                window,
                cx,
            );
            input.set_masked(true, window, cx);
        });
        self.settings_open = false;
        self.api_key_visible = false;
        cx.notify();
    }

    fn toggle_api_key_visibility(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.api_key_visible = !self.api_key_visible;
        self.api_key_input.update(cx, |input, cx| {
            input.set_masked(!self.api_key_visible, window, cx);
        });
        cx.notify();
    }

    fn save_settings_editor(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let profile_name = self.profile_name_input.read(cx).value().to_string();
        let api_key = self.api_key_input.read(cx).value().to_string();
        let next = AppSettings {
            profile_name,
            meroalpha_api_key: Some(api_key),
        };

        match self.repository.save_settings(&next) {
            Ok(()) => {
                self.settings = self.repository.load_settings();
                self.settings_open = false;
                self.push_notification(
                    window,
                    cx,
                    AppNotificationKind::Success,
                    "Settings saved",
                    "Profile settings were saved locally.",
                );
            }
            Err(error) => {
                let message = format!("{error:?}");
                self.push_notification(
                    window,
                    cx,
                    AppNotificationKind::Error,
                    "Settings save failed",
                    message,
                );
            }
        }
        cx.notify();
    }

    // ── Render helpers ────────────────────────────────────────────────────────

    fn render_page_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let api_key_configured = configured_api_key(&self.settings).is_some();
        let refresh_prices_disabled =
            price_refresh_disabled(api_key_configured, self.refreshing_prices);

        h_flex()
            .justify_between()
            .items_end()
            .gap_4()
            .flex_wrap()
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_color(cx.theme().foreground)
                            .text_size(px(28.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Portfolio"),
                    )
                    .child(
                        div()
                            .text_color(cx.theme().muted_foreground)
                            .text_size(px(14.))
                            .child("MeroShare holdings — local valuation"),
                    ),
            )
            .child(
                h_flex()
                    .gap_3()
                    .child(
                        Button::new("refresh-prices")
                            .small()
                            .w(px(36.))
                            .h(px(36.))
                            .icon(IconName::Redo)
                            .tooltip(price_refresh_tooltip(
                                api_key_configured,
                                self.refreshing_prices,
                            ))
                            .disabled(refresh_prices_disabled)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.refresh_prices(window, cx);
                            })),
                    )
                    .child(
                        Button::new("import-csv-header")
                            .primary()
                            .small()
                            .w(px(36.))
                            .h(px(36.))
                            .icon(IconName::Plus)
                            .tooltip("Import CSV")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.open_csv_picker(window, cx);
                            })),
                    ),
            )
    }

    fn nav_item(
        &self,
        route: AppRoute,
        icon: IconName,
        label: &'static str,
        cx: &mut Context<Self>,
    ) -> SidebarNavItem {
        SidebarNavItem {
            id: route.key(),
            icon,
            label,
            active: self.active_route == route,
            on_click: Box::new(cx.listener(move |this, _, _window, cx| {
                this.active_route = route;
                cx.notify();
            })),
        }
    }

    fn render_portfolio_page(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let snapshot = self.snapshot.as_ref().unwrap();
        let theme = cx.theme().clone();
        let window_width = window.bounds().size.width;
        let is_stacked = window_width < px(1180.);

        v_flex().size_full().bg(theme.background).child(
            if is_stacked {
                v_flex()
                    .size_full()
                    .items_start()
                    .gap_4()
                    .p_5()
                    .overflow_y_scrollbar()
                    .child(self.render_page_header(cx))
                    .child(render_kpis(snapshot, &theme))
                    .child(render_holdings_table(snapshot, &theme))
                    .child(render_right_rail_responsive(
                        snapshot,
                        &theme,
                        true,
                        window_width,
                    ))
                    .into_any_element()
            } else {
                h_flex()
                    .size_full()
                    .items_start()
                    .gap_4()
                    .p_5()
                    .child(
                        v_flex()
                            .flex_1()
                            .h_full()
                            .gap_4()
                            .overflow_y_scrollbar()
                            .child(self.render_page_header(cx))
                            .child(render_kpis(snapshot, &theme))
                            .child(render_holdings_table(snapshot, &theme)),
                    )
                    .child(render_right_rail(snapshot, &theme))
                    .into_any_element()
            }
        )
    }

    fn render_empty_page(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        v_flex()
            .size_full()
            .bg(theme.background)
            .child(render_empty_state(
                &theme,
                cx.listener(|this, _, window, cx| {
                    this.open_csv_picker(window, cx);
                }),
            ))
    }

    fn render_overview_route(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let overview = overview_snapshot_for_route(
            self.snapshot.as_ref(),
            self.overview_market_data.clone(),
            self.overview_refresh_error.is_some(),
            self.overview_mover_tab,
        );
        render_overview_page(
            overview,
            theme,
            window,
            cx.listener(|this, _, _, cx| {
                this.overview_mover_tab = MoverTab::Gainers;
                cx.notify();
            }),
            cx.listener(|this, _, _, cx| {
                this.overview_mover_tab = MoverTab::Losers;
                cx.notify();
            }),
            cx.listener(|this, _, _, cx| {
                this.overview_mover_tab = MoverTab::Turnover;
                cx.notify();
            }),
        )
    }

    fn render_active_route(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        match self.active_route {
            AppRoute::Overview => div()
                .flex_1()
                .h_full()
                .overflow_y_scrollbar()
                .child(self.render_overview_route(window, cx))
                .into_any_element(),
            AppRoute::Portfolio => {
                let content = if self.snapshot.is_some() {
                    self.render_portfolio_page(window, cx).into_any_element()
                } else {
                    self.render_empty_page(cx).into_any_element()
                };

                div()
                    .flex_1()
                    .h_full()
                    .overflow_hidden()
                    .child(content)
                    .into_any_element()
            }
        }
    }
}

fn price_refresh_status(updated: usize, unavailable: usize, local_floor_count: usize) -> String {
    let mut parts = Vec::new();

    if updated > 0 {
        parts.push(format!("Updated {updated} prices from MeroAlpha."));
    } else {
        parts.push("No upstream prices updated.".to_string());
    }

    if unavailable > 0 {
        parts.push(format!(
            "Kept local valuation for {unavailable} unavailable symbols."
        ));
    }

    if local_floor_count > 0 {
        parts.push(format!(
            "Applied NPR 1.00 local floor to {local_floor_count} symbols."
        ));
    }

    parts.join(" ")
}

fn price_refresh_status_kind(message: &str) -> StatusKind {
    if message.contains("No upstream") || message.contains("Kept local valuation") {
        StatusKind::Warning
    } else {
        StatusKind::Success
    }
}

fn notification_kind_for_status(message: &str) -> AppNotificationKind {
    if price_refresh_status_kind(message) == StatusKind::Warning {
        AppNotificationKind::Warning
    } else {
        AppNotificationKind::Success
    }
}

fn configured_api_key(settings: &AppSettings) -> Option<String> {
    settings
        .meroalpha_api_key
        .as_deref()
        .map(str::trim)
        .filter(|key| !key.is_empty())
        .map(ToString::to_string)
}

fn price_refresh_disabled(api_key_configured: bool, refreshing: bool) -> bool {
    !api_key_configured || refreshing
}

fn price_refresh_tooltip(api_key_configured: bool, refreshing: bool) -> &'static str {
    if !api_key_configured {
        "Set API key to refresh prices"
    } else if refreshing {
        "Refreshing prices"
    } else {
        "Refresh Prices"
    }
}

fn overview_snapshot_for_route(
    portfolio: Option<&PortfolioSnapshot>,
    market_data: Option<OverviewMarketData>,
    refresh_failed: bool,
    selected_tab: MoverTab,
) -> OverviewSnapshot {
    let overview =
        OverviewSnapshot::from_portfolio(portfolio).with_selected_mover_tab(selected_tab);
    if refresh_failed {
        overview.with_market_error()
    } else if let Some(market_data) = market_data {
        overview.with_market_data(market_data, selected_tab)
    } else {
        overview
    }
}

// ── Render ────────────────────────────────────────────────────────────────────

impl Render for MeroAlphaTerminal {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let notification_layer = Root::render_notification_layer(window, cx);
        let nav_items = AppRoute::sidebar_routes()
            .iter()
            .map(|route| {
                let (icon, label) = match route {
                    AppRoute::Overview => (IconName::LayoutDashboard, "Overview"),
                    AppRoute::Portfolio => (IconName::HardDrive, "Portfolio"),
                };
                self.nav_item(*route, icon, label, cx)
            })
            .collect::<Vec<_>>();

        h_flex()
            .size_full()
            .bg(theme.background)
            .child(render_sidebar(
                &theme,
                nav_items,
                &self.settings.profile_name,
                configured_api_key(&self.settings).is_some(),
                self.settings_open,
                &self.profile_name_input,
                &self.api_key_input,
                cx.listener(|this, _, window, cx| {
                    this.open_settings_editor(window, cx);
                }),
                self.api_key_visible,
                cx.listener(|this, _, window, cx| {
                    this.toggle_api_key_visibility(window, cx);
                }),
                cx.listener(|this, _, window, cx| {
                    this.save_settings_editor(window, cx);
                }),
                cx.listener(|this, _, window, cx| {
                    this.cancel_settings_editor(window, cx);
                }),
            ))
            .child(
                v_flex()
                    .flex_1()
                    .h_full()
                    .min_w(px(0.))
                    .child(render_top_bar(
                        &theme,
                        configured_api_key(&self.settings).is_some(),
                        self.refreshing_overview,
                        cx.listener(|this, _, window, cx| {
                            this.refresh_overview_market_data(window, cx, true);
                        }),
                        cx.listener(|this, _, window, cx| {
                            this.open_settings_editor(window, cx);
                        }),
                    ))
                    .child(self.render_active_route(window, cx)),
            )
            .children(notification_layer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use meroalpha_terminal::meroalpha_api::{MarketStatusUpdate, OverviewMarketData};

    #[test]
    fn maps_price_refresh_status_to_notification_kind() {
        assert_eq!(
            notification_kind_for_status("Updated 4 prices from MeroAlpha."),
            AppNotificationKind::Success
        );
        assert_eq!(
            notification_kind_for_status(
                "No upstream prices updated. Kept local valuation for 2 unavailable symbols."
            ),
            AppNotificationKind::Warning
        );
    }

    #[test]
    fn app_route_keys_are_stable_for_sidebar_selection() {
        assert_eq!(AppRoute::Overview.key(), "overview");
        assert_eq!(AppRoute::Portfolio.key(), "portfolio");
    }

    #[test]
    fn sidebar_routes_only_include_working_pages() {
        assert_eq!(
            AppRoute::sidebar_routes(),
            &[AppRoute::Overview, AppRoute::Portfolio]
        );
    }

    #[test]
    fn configured_api_key_ignores_blank_keys() {
        assert_eq!(
            configured_api_key(&AppSettings {
                profile_name: "Test".to_string(),
                meroalpha_api_key: Some("  ma_key  ".to_string()),
            }),
            Some("ma_key".to_string())
        );
        assert_eq!(
            configured_api_key(&AppSettings {
                profile_name: "Test".to_string(),
                meroalpha_api_key: Some("   ".to_string()),
            }),
            None
        );
        assert_eq!(
            configured_api_key(&AppSettings {
                profile_name: "Test".to_string(),
                meroalpha_api_key: None,
            }),
            None
        );
    }

    #[test]
    fn price_refresh_button_requires_api_key_and_idle_state() {
        assert!(price_refresh_disabled(false, false));
        assert_eq!(
            price_refresh_tooltip(false, false),
            "Set API key to refresh prices"
        );

        assert!(price_refresh_disabled(true, true));
        assert_eq!(price_refresh_tooltip(true, true), "Refreshing prices");

        assert!(!price_refresh_disabled(true, false));
        assert_eq!(price_refresh_tooltip(true, false), "Refresh Prices");
    }

    #[test]
    fn overview_error_wins_over_stale_market_data() {
        let overview = overview_snapshot_for_route(
            None,
            Some(OverviewMarketData {
                market_status: MarketStatusUpdate {
                    status: "CLOSE".to_string(),
                    is_open: false,
                    as_of: None,
                    last_traded_date: "2026-06-25".to_string(),
                },
                indices: Vec::new(),
                gainers: Vec::new(),
                losers: Vec::new(),
                turnover: Vec::new(),
            }),
            true,
            MoverTab::Gainers,
        );

        assert!(overview.market_status.is_error());
        assert_eq!(overview.market_recency_label(), "Request failed");
        assert_eq!(overview.selected_mover_tab, MoverTab::Gainers);
    }

    #[test]
    fn overview_route_preserves_selected_tab_without_market_rows() {
        let waiting = overview_snapshot_for_route(None, None, false, MoverTab::Turnover);
        assert_eq!(waiting.selected_mover_tab, MoverTab::Turnover);
        assert_eq!(waiting.mover_metric_label, "TURNOVER (NPR)");

        let failed = overview_snapshot_for_route(None, None, true, MoverTab::Losers);
        assert_eq!(failed.selected_mover_tab, MoverTab::Losers);
        assert_eq!(failed.mover_metric_label, "VOLUME");
    }
}
