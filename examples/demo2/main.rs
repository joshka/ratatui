use anyhow::{Context, Result};

mod app;
mod bars;
mod chart;
mod colors;
mod email;
mod gauges;
mod main_view;
mod modifiers;
mod recipe;
mod text;
mod traceroute;

fn main() -> Result<()> {
    app::install_panic_hook();
    app::App::new()?.run()
}
