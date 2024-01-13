use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use ratatui::{layout::Flex, prelude::*};
use tui_big_text::{BigTextBuilder, PixelSize};
use unicode_width::UnicodeWidthStr;

use crate::{Root, Term};

#[derive(Debug)]
pub struct App {
    term: Term,
    context: AppContext,
    mode: Mode,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Mode {
    #[default]
    Normal,
    Destroy,
    Quit,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AppContext {
    pub tab_index: usize,
    pub row_index: usize,
}

impl App {
    fn new() -> Result<Self> {
        Ok(Self {
            term: Term::start()?,
            context: AppContext::default(),
            mode: Mode::Normal,
        })
    }

    pub fn run() -> Result<()> {
        install_panic_hook();
        let mut app = Self::new()?;
        while !app.should_quit() {
            app.draw()?;
            app.handle_events()?;
        }
        Term::stop()?;
        Ok(())
    }

    fn draw(&mut self) -> Result<()> {
        self.term
            .draw(|frame| {
                frame.render_widget(Root::new(&self.context), frame.size());
                if self.mode == Mode::Destroy && frame.count() > 60 {
                    render_destroy_mode(frame);
                }
            })
            .context("terminal.draw")?;
        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        match Term::next_event(Duration::from_millis(16))? {
            Some(Event::Key(key)) => self.handle_key_event(key),
            Some(Event::Resize(width, height)) => {
                Ok(self.term.resize(Rect::new(0, 0, width, height))?)
            }
            _ => Ok(()),
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }

        let context = &mut self.context;
        const TAB_COUNT: usize = 5;
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.mode = Mode::Quit;
            }
            KeyCode::Tab | KeyCode::BackTab if key.modifiers.contains(KeyModifiers::SHIFT) => {
                let tab_index = context.tab_index + TAB_COUNT; // to wrap around properly
                context.tab_index = tab_index.saturating_sub(1) % TAB_COUNT;
                context.row_index = 0;
            }
            KeyCode::Tab | KeyCode::BackTab => {
                context.tab_index = context.tab_index.saturating_add(1) % TAB_COUNT;
                context.row_index = 0;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                context.row_index = context.row_index.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                context.row_index = context.row_index.saturating_add(1);
            }
            KeyCode::Char('d') => {
                self.mode = Mode::Destroy;
            }
            _ => {}
        };
        Ok(())
    }

    fn should_quit(&self) -> bool {
        self.mode == Mode::Quit
    }
}

fn render_destroy_mode(frame: &mut Frame<'_>) {
    // need a seeded rng to make the frame render the same moved pixels each time
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(10);
    let area = frame.size();

    // move a bunch of pixels down one row - pick a random pixel and move it down
    // this is a very inefficient way to do this, but it's just for fun

    let frame_count = frame.count();
    let buf = frame.buffer_mut();

    let speed_multiplier = 10; // higher is more pixels per frame
    for _ in 0..(frame_count * speed_multiplier) {
        let x = rng.gen_range(0..area.width);
        let y = rng.gen_range(0..area.height);
        let new_y = y.saturating_add(1).min(area.bottom() - 1);

        // copy the cell down one row
        let source_cell = buf.get_mut(x, y).clone();
        let dest_cell = buf.get_mut(x, new_y);
        *dest_cell = source_cell;
    }

    // draw some text fading in and out from black to red and back
    let sub_frame = (frame.count() % 120) as u8;
    if sub_frame < 20 {
        // ramp brightness 0..255..0
        let text_brightness = if sub_frame < 8 {
            sub_frame.saturating_mul(32)
        } else {
            16_u8.saturating_sub(sub_frame).saturating_mul(32)
        };
        // a shade of red
        let color = Color::Rgb(text_brightness, 0, 0);

        let big_text = BigTextBuilder::default()
            .lines(vec!["  OBEY  ".into(), "THE RAT".into()])
            .pixel_size(PixelSize::Full)
            .style(Style::new().fg(color))
            .build()
            .unwrap();
        let width = "THE RAT".width() as u16 * 8;
        let height = 2 * 8;

        // a centered layout
        let horizontal = Layout::horizontal([width]).flex(Flex::Center);
        let vertical = Layout::vertical([height]).flex(Flex::Center);
        let [area] = area.split(&vertical);
        let [area] = area.split(&horizontal);

        frame.render_widget(big_text, area);
    }
}

pub fn install_panic_hook() {
    better_panic::install();
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = Term::stop();
        hook(info);
        std::process::exit(1);
    }));
}
