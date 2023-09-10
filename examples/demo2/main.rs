use anyhow::Result;

mod app;
mod app_widget;
mod colors;
mod styles;
mod tabs;
mod tui;

fn main() -> Result<()> {
    app::install_panic_hook();
    app::App::new()?.run()
}
