use std::{io::Stdout, time::Duration};

use anyhow::{Context, Result};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::*;

use crate::{app_widget::AppWidget, tabs, tabs::Tab, tui};

pub struct App {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    should_quit: bool,
    tab_index: usize,
    selected_row: usize,
    tabs: Vec<Box<dyn Tab>>,
}

impl App {
    pub fn new() -> Result<Self> {
        let terminal = tui::create_terminal()?;
        Ok(Self {
            terminal,
            should_quit: false,
            tab_index: 0,
            selected_row: 0,
            tabs: vec![
                Box::new(tabs::AboutTab::new()),
                Box::new(tabs::EmailTab::new()),
                Box::new(tabs::TracerouteTab::new()),
                Box::new(tabs::BarsTab::new()),
            ],
        })
    }

    pub fn run(&mut self) -> Result<()> {
        tui::setup()?;
        while !self.should_quit {
            self.draw()?;
            self.handle_events()?;
        }
        tui::restore()?;
        Ok(())
    }

    fn draw(&mut self) -> Result<()> {
        self.terminal
            .draw(|frame| {
                let view = AppWidget::new(self.tab_index);
                let area = frame.size();
                frame.render_widget(view, area)
            })
            .context("terminal.draw")?;
        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        match tui::next_event(Duration::from_millis(16))? {
            Some(Event::Key(key)) => self.handle_key_event(key),
            _ => Ok(()),
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Tab => {
                self.tab_index = self.tab_index.saturating_add(1).min(self.tabs.len() - 1);
            }
            KeyCode::BackTab => {
                self.tab_index = self.tab_index.saturating_sub(1);
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.selected_row = 0;
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.selected_row = 0;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_row = self.selected_row.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected_row = self.selected_row.saturating_add(1);
            }
            _ => {}
        };
        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = tui::restore();
    }
}

pub fn install_panic_hook() {
    better_panic::install();
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = tui::restore();
        hook(info);
        std::process::exit(1);
    }));
}
