use std::{io::Stdout, time::Duration};

use anyhow::{Context, Result};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use itertools::Itertools;
use ratatui::prelude::*;

use crate::{
    app_widget::AppWidget,
    tabs,
    tabs::{AboutTab, EmailTab, MiscWidgetsTab, RecipeTab, Tab, TracerouteTab},
    tui,
};

pub struct App {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    should_quit: bool,
    tab_index: usize,
    selected_index: usize,
    tabs: Vec<Box<dyn Tab>>,
}

impl App {
    pub fn new() -> Result<Self> {
        let terminal = tui::create_terminal()?;
        Ok(Self {
            terminal,
            should_quit: false,
            tab_index: 0,
            selected_index: 0,
            tabs: vec![
                Box::new(tabs::AboutTab::new()),
                Box::new(tabs::EmailTab::new(0)),
                Box::new(tabs::TracerouteTab::new(0)),
                Box::new(tabs::MiscWidgetsTab::new()),
                Box::new(tabs::RecipeTab::new(0)),
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
                let titles = self.tabs.iter().map(|tab| tab.title()).collect_vec();
                let tab: Box<dyn Tab> = {
                    // This is a bit of a hack to get around the borrow checker.
                    // which works because we know that the tabs are all static.
                    match self.tab_index {
                        0 => Box::new(AboutTab::new()),
                        1 => Box::new(EmailTab::new(self.selected_index)),
                        2 => Box::new(TracerouteTab::new(self.selected_index)),
                        3 => Box::new(MiscWidgetsTab::new()),
                        4 => Box::new(RecipeTab::new(self.selected_index)),
                        _ => unreachable!(),
                    }
                };
                let view = AppWidget::new(tab, self.tab_index, titles);
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
            KeyCode::Tab | KeyCode::BackTab if key.modifiers.contains(KeyModifiers::SHIFT) => {
                let tab_index = self.tab_index + self.tabs.len(); // to wrap around properly
                self.tab_index = tab_index.saturating_sub(1) % self.tabs.len();
                self.selected_index = 0;
            }
            KeyCode::Tab | KeyCode::BackTab => {
                self.tab_index = self.tab_index.saturating_add(1) % self.tabs.len();
                self.selected_index = 0;
            }
            KeyCode::Left | KeyCode::Char('h') => {}
            KeyCode::Right | KeyCode::Char('l') => {}
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_index = self.selected_index.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected_index = self.selected_index.saturating_add(1);
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
