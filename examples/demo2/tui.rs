use std::{
    io::{self, stdout, Stdout},
    rc::Rc,
    time::Duration,
};

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use itertools::Itertools;
use ratatui::prelude::*;

pub fn create_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    // this size is to match the size of the terminal when running the demo
    // using vhs in a 1280x640 sized window (github social preview size)
    let options = TerminalOptions {
        viewport: Viewport::Fixed(Rect::new(0, 0, 81, 18)),
        // viewport: Viewport::Fullscreen,
    };
    let terminal = Terminal::with_options(CrosstermBackend::new(io::stdout()), options)?;
    Ok(terminal)
}

pub fn setup() -> Result<()> {
    enable_raw_mode().context("enable raw mode")?;
    stdout()
        .execute(EnterAlternateScreen)
        .context("enter alternate screen")?;
    Ok(())
}

pub fn restore() -> Result<()> {
    disable_raw_mode().context("disable raw mode")?;
    stdout()
        .execute(LeaveAlternateScreen)
        .context("leave alternate screen")?;
    Ok(())
}

pub fn next_event(timeout: Duration) -> io::Result<Option<Event>> {
    if !event::poll(timeout)? {
        return Ok(None);
    }
    let event = event::read()?;
    Ok(Some(event))
}

/// helper method to split an area into multiple sub-areas
pub fn layout(area: Rect, direction: Direction, heights: Vec<u16>) -> Rc<[Rect]> {
    let constraints = heights
        .iter()
        .map(|&h| {
            if h > 0 {
                Constraint::Length(h)
            } else {
                Constraint::Min(0)
            }
        })
        .collect_vec();
    Layout::default()
        .direction(direction)
        .constraints(constraints)
        .split(area)
}
