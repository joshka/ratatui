use anyhow::Result;

mod app;
mod colors;
mod tabs;
mod text;
mod tui;
mod view;

fn main() -> Result<()> {
    app::install_panic_hook();
    app::App::new()?.run()
}
