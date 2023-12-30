use std::io::stdout;

use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    widgets::{Block, Paragraph, Widget},
    Terminal, TerminalOptions, Viewport,
};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut terminal = Terminal::with_options(
        CrosstermBackend::new(stdout()),
        TerminalOptions {
            viewport: Viewport::Inline(4),
        },
    )?;

    terminal.insert_before(2, |buf| {
        let reusable_paragraph = Paragraph::new("Hello");
        reusable_paragraph.render(Rect::new(0, 0, 30, 1), buf);
        reusable_paragraph.render(Rect::new(0, 1, 30, 1), buf);
    })?;

    Ok(())
}
