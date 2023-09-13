use anyhow::{Context, Result};

mod app;
mod chart;
mod colors;
mod gauges;
mod main_view;
mod modifiers;
mod table;
mod text;

fn main() -> Result<()> {
    app::install_panic_hook();
    app::App::new()?.run()
}
