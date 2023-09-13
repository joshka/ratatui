use std::{
    io::{self, stdout, Stdout},
    time::Duration,
};

use crossterm::{
    event::{self, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;

use crate::{main_view::MainView, *};

pub struct App {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    should_quit: bool,
    selected_tab: usize,
    selected_row: usize,
}

impl App {
    pub fn new() -> Result<Self> {
        let terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        Ok(Self {
            terminal,
            should_quit: false,
            selected_tab: 0,
            selected_row: 0,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        setup_terminal()?;
        while !self.should_quit {
            self.draw()?;
            self.handle_events()?;
        }
        restore_terminal()
    }

    fn draw(&mut self) -> Result<()> {
        self.terminal
            .draw(|frame| {
                let area = frame.size();
                let widget = MainView {
                    selected_tab: self.selected_tab,
                    selected_row: self.selected_row,
                };
                frame.render_widget(widget, area);
            })
            .context("terminal.draw")?;
        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        if !event::poll(Duration::from_millis(16))? {
            return Ok(());
        }
        match event::read()? {
            event::Event::Key(key) => self.handle_key_event(key),
            _ => Ok(()),
        }
    }

    fn handle_key_event(&mut self, key: event::KeyEvent) -> std::result::Result<(), anyhow::Error> {
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }
        match key.code {
            event::KeyCode::Char('q') => {
                self.should_quit = true;
            }
            event::KeyCode::Left => {
                self.selected_tab = self.selected_tab.saturating_sub(1);
                self.selected_row = 0;
            }
            event::KeyCode::Right => {
                self.selected_tab = self.selected_tab.saturating_add(1);
                self.selected_row = 0;
            }
            event::KeyCode::Up => {
                self.selected_row = self.selected_row.saturating_sub(1);
            }
            event::KeyCode::Down => {
                self.selected_row = self.selected_row.saturating_add(1);
            }
            _ => {}
        };
        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = restore_terminal();
    }
}

fn setup_terminal() -> Result<()> {
    enable_raw_mode().context("enable raw mode")?;
    stdout()
        .execute(EnterAlternateScreen)
        .context("enter alternate screen")?;
    Ok(())
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode().context("disable raw mode")?;
    stdout()
        .execute(LeaveAlternateScreen)
        .context("leave alternate screen")?;
    Ok(())
}

pub fn install_panic_hook() {
    better_panic::install();
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        hook(info);
        std::process::exit(1);
    }));
}
