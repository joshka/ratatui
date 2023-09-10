use std::{
    io::{self, stdout, Stdout},
    time::Duration,
};

use anyhow::{Context, Result};
use crossterm::{
    event::{self, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{block::Title, *},
};

fn main() -> Result<()> {
    install_panic_hook();
    App::new()?.run()
}

struct App {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    should_quit: bool,
    selected_tab: usize,
}

impl App {
    fn new() -> Result<Self> {
        let terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        Ok(Self {
            terminal,
            should_quit: false,
            selected_tab: 0,
        })
    }

    fn run(&mut self) -> Result<()> {
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
                let widget = MainWidget {
                    selected_tab: self.selected_tab,
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
            }
            event::KeyCode::Right => {
                self.selected_tab = self.selected_tab.saturating_add(1).min(2);
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

struct MainWidget {
    selected_tab: usize,
}

impl Widget for MainWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(vec![
            "Ratatui Demo 2. Press ".dark_gray(),
            "q".gray(),
            " to quit".dark_gray(),
        ]);
        let block = Block::default().title(Title::from(title).alignment(Alignment::Center));
        let inner = block.inner(area);
        block.render(area, buf);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
            .split(inner);
        let tabs = Tabs::new(vec!["Tab1", "Tab2", "Tab3"])
            .style(Style::new().black().on_light_blue())
            .highlight_style(Style::new().white().on_light_red())
            .select(self.selected_tab);
        tabs.render(layout[0], buf);

        let data = (0..area.width * 2)
            .map(f64::from)
            .map(|x| (x / area.width as f64) * 10.0 - 5.0)
            .map(|x| {
                (
                    x,
                    x.powi(5) + 3.5 * x.powi(4) - 2.5 * x.powi(3) - 12.5 * x.powi(2)
                        + 1.5 * x
                        + 9.0,
                )
            })
            .collect::<Vec<_>>();
        let datasets = vec![Dataset::default()
            .name("data1")
            .marker(Marker::Braille)
            .style(Style::new().fg(Color::Red))
            .graph_type(GraphType::Line)
            .data(&data[..])];
        let chart = Chart::new(datasets)
            .x_axis(Axis::default().title("x").bounds([-4.0, 4.0]).labels(vec![
                "-4.0".into(),
                "0".into(),
                "4.0".into(),
            ]))
            .y_axis(Axis::default().title("y").bounds([-8.0, 8.0]).labels(vec![
                "-8".into(),
                "0".into(),
                "8".into(),
            ]));
        chart.render(layout[1], buf);
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

fn install_panic_hook() {
    better_panic::install();
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        hook(info);
        std::process::exit(1);
    }));
}
