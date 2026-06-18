mod app;

use gpui::{App, AppContext, WindowOptions};
use gpui_component::Root;

use app::MeroAlphaTerminal;

fn main() {
    gpui_platform::application()
        .with_assets(gpui_component_assets::Assets)
        .run(|cx: &mut App| {
            gpui_component::init(cx);

            cx.spawn(async move |cx| {
                cx.open_window(WindowOptions::default(), |window, cx| {
                    let view = cx.new(|cx| MeroAlphaTerminal::new(window, cx));
                    cx.new(|cx| Root::new(view, window, cx))
                })
                .expect("failed to open MeroAlpha Terminal window");
            })
            .detach();
        });
}
