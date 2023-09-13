use std::{
    io::{self, stdout, Stdout},
    rc::Rc,
    time::Duration,
};

use anyhow::{Context, Result};
use crossterm::{
    event::{self, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use itertools::Itertools;
use palette::{
    convert::{FromColorUnclamped, IntoColorUnclamped},
    Okhsv, Srgb,
};
use ratatui::{
    prelude::*,
    widgets::{StatefulWidget, *},
};

fn main() -> Result<()> {
    install_panic_hook();
    App::new()?.run()
}

struct App {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    should_quit: bool,
    selected_tab: usize,
    selected_row: usize,
}

impl App {
    fn new() -> Result<Self> {
        let terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        Ok(Self {
            terminal,
            should_quit: false,
            selected_tab: 0,
            selected_row: 0,
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
                self.selected_tab = self.selected_tab.saturating_add(1).min(2);
                self.selected_row = 0;
            }
            event::KeyCode::Up => {
                self.selected_row = self.selected_row.saturating_sub(1);
            }
            event::KeyCode::Down => {
                self.selected_row = self.selected_row.saturating_add(1).min(10);
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

fn render_title(title: &str, area: Rect, buf: &mut Buffer) {
    Paragraph::new(title)
        .white()
        .bold()
        .underlined()
        .render(Rect { height: 1, ..area }, buf);
}

/// helper method to split an area into multiple sub-areas
fn layout(area: Rect, direction: Direction, heights: Vec<u16>) -> Rc<[Rect]> {
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

struct MainView {
    selected_tab: usize,
    selected_row: usize,
}

impl Widget for MainView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let areas = layout(area, Direction::Vertical, vec![1, 0, 1]);

        self.render_title_bar(areas[0], buf);
        match self.selected_tab {
            0 => self.render_tab1(areas[1], buf),
            1 => render_title("Tab2", areas[1], buf),
            2 => render_title("Tab3", areas[1], buf),
            _ => unreachable!(),
        }
        self.render_bottom_bar(areas[2], buf);
    }
}

impl MainView {
    fn render_title_bar(&self, area: Rect, buf: &mut Buffer) {
        let areas = layout(area, Direction::Horizontal, vec![12, 0]);
        Paragraph::new(Line::from(vec![
            // █ characters here are a hack around the VHS bug that swallows style resets for
            // whitespace characters
            "█".blue().not_bold(), // vhs bug hack
            "Ratatui".into(),
            "█".blue().not_bold(),
            "████".light_blue().not_bold(),
        ]))
        .italic()
        .bold()
        .gray()
        .on_blue()
        .render(areas[0], buf);

        Tabs::new(vec![" Tab1 ", " Tab2 ", " Tab3 "])
            .style(Style::new().blue().on_light_blue())
            .highlight_style(Style::new().bold().light_blue().on_blue())
            .select(self.selected_tab)
            .render(areas[1], buf);
    }

    fn render_bottom_bar(&self, area: Rect, buf: &mut Buffer) {
        let key_style = Style::new().bold().on_blue();
        Paragraph::new(Line::from(vec![
            // █ characters here are a hack around the VHS bug that swallows style resets for
            // whitespace characters
            "█".into(),
            "Q".set_style(key_style),
            "█ Quit █".into(),
            "←".set_style(key_style),
            "█ Previous Tab █".into(),
            "→".set_style(key_style),
            "█ Next Tab █".into(),
            "↑".set_style(key_style),
            "█ Previous Row █".into(),
            "↓".set_style(key_style),
            "█ Next Row".into(),
        ]))
        .blue()
        .on_light_blue()
        .render(area, buf);
    }

    fn render_tab1(&self, area: Rect, buf: &mut Buffer) {
        let areas = layout(area, Direction::Vertical, vec![6, 1, 2, 1, 0]);
        ColorsExample.render(areas[0], buf);
        ModifiersExample.render(areas[2], buf);
        TableExample::new(self.selected_row).render(areas[4], buf);
    }
}

struct ColorsExample;

impl Widget for ColorsExample {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let areas = layout(area, Direction::Vertical, vec![1, 0]);
        let top_areas = layout(areas[1], Direction::Horizontal, vec![8, 1, 36, 1, 0]);

        render_title("Colors", areas[0], buf);
        Self::render_16_colors(top_areas[0], buf);
        Self::render_256_colors(top_areas[2], buf);
        Self::render_rgb_colors(top_areas[4], buf);
    }
}

impl ColorsExample {
    fn render_16_colors(area: Rect, buf: &mut Buffer) {
        let areas = layout(area, Direction::Vertical, vec![1, 0]);
        Paragraph::new("16 color").render(areas[0], buf);
        let sym = "██";
        Paragraph::new(vec![
            Line::from(vec![sym.black(), sym.red(), sym.green(), sym.yellow()]),
            Line::from(vec![sym.blue(), sym.magenta(), sym.cyan(), sym.gray()]),
            Line::from(vec![
                sym.dark_gray(),
                sym.light_red(),
                sym.light_green(),
                sym.light_yellow(),
            ]),
            Line::from(vec![
                sym.light_blue(),
                sym.light_magenta(),
                sym.light_cyan(),
                sym.white(),
            ]),
        ])
        .render(areas[1], buf);
    }

    fn render_256_colors(area: Rect, buf: &mut Buffer) {
        let layout = layout(area, Direction::Vertical, vec![1, 0]);
        Paragraph::new("256 colors (Indexed RGB)").render(layout[0], buf);
        let area = layout[1];
        for (xi, x) in (16..52).zip(area.left()..area.right()) {
            for (yi, y) in (0..3).zip(area.top()..area.bottom()) {
                let fg = Color::Indexed(yi * 72 + xi);
                let bg = Color::Indexed(yi * 72 + xi + 36);
                buf.get_mut(x, y).set_char('▀').set_fg(fg).set_bg(bg);
            }
            let fg = Color::Indexed(xi.saturating_add(216));
            buf.get_mut(x, area.bottom() - 1).set_char('█').set_fg(fg);
        }
    }

    fn render_rgb_colors(area: Rect, buf: &mut Buffer) {
        let layout = layout(area, Direction::Vertical, vec![1, 0]);
        Paragraph::new("24bit RGB (Truecolor)").render(layout[0], buf);
        let area = layout[1];
        for (xi, x) in (area.left()..area.right()).enumerate() {
            for (yi, y) in (area.top()..area.bottom()).enumerate() {
                let hue = xi as f32 * 360.0 / area.width as f32;
                let value_fg = (yi as f32 + 0.5) / (area.height as f32);
                let value_bg = (yi as f32 + 1.0) / (area.height as f32);
                let fg = Okhsv::<f32>::new(hue, Okhsv::max_saturation(), value_fg);
                let fg: Srgb<f32> = fg.into_color_unclamped();
                let fg: Srgb<u8> = fg.into_format();
                let fg = Color::Rgb(fg.red, fg.green, fg.blue);
                let bg = Okhsv::new(hue, Okhsv::max_saturation(), value_bg);
                let bg = Srgb::<f32>::from_color_unclamped(bg);
                let bg: Srgb<u8> = bg.into_format();
                let bg = Color::Rgb(bg.red, bg.green, bg.blue);
                buf.get_mut(x, y).set_char('▀').set_fg(fg).set_bg(bg);
            }
        }
    }
}

struct ModifiersExample;

impl Widget for ModifiersExample {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = layout(area, Direction::Vertical, vec![1, 1]);
        render_title("Modifiers", layout[0], buf);
        let area = layout[1];
        let text = Line::from(vec![
            "Bold".bold(),
            " ".into(),
            "Dim".dim(),
            " ".into(),
            "Italic".italic(),
            " ".into(),
            "Underlined".underlined(),
            " ".into(),
            "SlowBlink".slow_blink(),
            " ".into(),
            "RapidBlink".rapid_blink(),
            " ".into(),
            "Reversed".reversed(),
            " ".into(),
            "Hidden".hidden(),
            " ".into(),
            "CrossedOut".crossed_out(),
        ]);
        Paragraph::new(text).render(area, buf);
    }
}

struct TableExample {
    selected_row: usize,
}

impl TableExample {
    fn new(selected_row: usize) -> Self {
        Self { selected_row }
    }
}

impl Widget for TableExample {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = layout(area, Direction::Vertical, vec![1, 0]);
        render_title("Soup Ingredients", layout[0], buf);
        let mut state = TableState::default().with_selected(Some(self.selected_row));

        // https://www.realsimple.com/food-recipes/browse-all-recipes/ratatouille
        StatefulWidget::render(
            Table::new(vec![
                Row::new(vec!["01", "4 tbsp", "olive oil", ""]),
                Row::new(vec!["02", "1", "onion", "thinly sliced"]),
                Row::new(vec!["03", "4", "cloves garlic", "peeled and sliced"]),
                Row::new(vec!["04", "1", "small bay leaf", ""]),
                Row::new(vec!["05", "1", "small eggplant", "cut into 1/2 inch cubes"]),
                Row::new(vec![
                    "06",
                    "1",
                    "small zucchini",
                    "halved lengthwise and cut into thin slices",
                ]),
                Row::new(vec!["07", "1", "red bell pepper", "cut into slivers"]),
                Row::new(vec!["08", "4", "plum tomatoes", "coarsely chopped"]),
                Row::new(vec!["09", "1 tsp", "kosher salt", ""]),
                Row::new(vec!["10", "1/4 cup", "shredded fresh basil leaves", ""]),
                Row::new(vec!["11", "", "freshly ground black pepper", ""]),
            ])
            .header(
                Row::new(vec!["Item", "Qty", "Ingredient", "Notes"])
                    .style(Style::new().black().on_light_blue()),
            )
            .widths(&[
                Constraint::Length(4),
                Constraint::Length(7),
                Constraint::Length(30),
                Constraint::Length(450),
            ])
            .highlight_style(Style::new().black().on_light_yellow()),
            layout[1],
            buf,
            &mut state,
        );
    }
}

// fn render_chart(area: Rect, layout: std::rc::Rc<[Rect]>, buf: &mut Buffer) {
//     let data = (0..area.width * 2)
//         .map(f64::from)
//         .map(|x| (x / area.width as f64) * 10.0 - 5.0)
//         .map(|x| {
//             (
//                 x,
//                 x.powi(5) + 3.5 * x.powi(4) - 2.5 * x.powi(3) - 12.5 * x.powi(2) + 1.5 * x + 9.0,
//             )
//         })
//         .collect::<Vec<_>>();
//     let datasets = vec![Dataset::default()
//         .name("data1")
//         .marker(Marker::Braille)
//         .style(Style::new().fg(Color::Red))
//         .graph_type(GraphType::Line)
//         .data(&data[..])];
//     let chart = Chart::new(datasets)
//         .x_axis(Axis::default().title("x").bounds([-4.0, 4.0]).labels(vec![
//             "-4.0".into(),
//             "0".into(),
//             "4.0".into(),
//         ]))
//         .y_axis(Axis::default().title("y").bounds([-8.0, 8.0]).labels(vec![
//             "-8".into(),
//             "0".into(),
//             "8".into(),
//         ]));
//     chart.render(layout[1], buf);
// }

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
