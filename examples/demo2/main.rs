//! # [Ratatui] Demo2 example
//!
//! The latest version of this example is available in the [examples] folder in the repository.
//!
//! Please note that the examples are designed to be run against the `main` branch of the Github
//! repository. This means that you may not be able to compile with the latest release version on
//! crates.io, or the one that you have installed locally.
//!
//! See the [examples readme] for more information on finding examples that match the version of the
//! library you are using.
//!
//! [Ratatui]: https://github.com/ratatui-org/ratatui
//! [examples]: https://github.com/ratatui-org/ratatui/blob/main/examples
//! [examples readme]: https://github.com/ratatui-org/ratatui/blob/main/examples/README.md

use anyhow::Result;
pub use app::*;
pub use colors::*;
pub use root::*;
pub use term::*;
pub use theme::*;

mod app;
mod big_text;
mod colors;
mod root;
mod tabs;
mod term;
mod theme;

fn main() -> Result<()> {
    App::run()
}
