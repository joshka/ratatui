use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use ratatui::{buffer::Buffer, layout::Flex, prelude::*, widgets::Widget};
use unicode_width::UnicodeWidthStr;

use crate::{
    big_text::{BigTextBuilder, PixelSize},
    Root, Term,
};

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
                if self.mode == Mode::Destroy {
                    destroy(frame);
                }
            })
            .context("terminal.draw")?;
        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        // https://superuser.com/questions/1449366/do-60-fps-gifs-actually-exist-or-is-the-maximum-50-fps
        const GIF_FRAME_RATE: f64 = 50.0;
        match Term::next_event(Duration::from_secs_f64(1.0 / GIF_FRAME_RATE))? {
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

/// delay the start of the animation so it doesn't start immediately
const DELAY: usize = 300;
/// higher is means more pixels per frame are modified in the animation
const SPEED_MULTIPLIER: usize = 100;
/// delay the start of the text animation so it doesn't start immediately after the initial delay
const TEXT_DELAY: usize = 120;

/// Destroy mode activated by pressing `d`
fn destroy(frame: &mut Frame<'_>) {
    let frame_count = frame.count().saturating_sub(DELAY);
    if frame_count == 0 {
        return;
    }

    let area = frame.size();
    let buf = frame.buffer_mut();
    let mask_buf = &mut Buffer::empty(area);
    text(frame_count, area, mask_buf);
    drip(frame_count, area, buf, mask_buf);
}

/// draw some text fading in and out from black to red and back
fn text(frame_count: usize, area: Rect, buf: &mut Buffer) {
    // let sub_frame = frame_count.saturating_sub(TEXT_DELAY);
    // if sub_frame == 0 {
    //     return;
    // }
    // // ramp red component brightness up and down 0..256..128
    // let red = if sub_frame < 256 {
    //     sub_frame
    // } else {
    //     512_usize.saturating_sub(sub_frame).clamp(128, 255)
    // };
    // let color = Color::Rgb(red as u8, 0, 0); // a shade of red

    let line1 = "RATATUI";
    let big_text = BigTextBuilder::default()
        .lines([line1.into()])
        .pixel_size(PixelSize::Full)
        // .style(Style::new().fg(color))
        .build()
        .unwrap();

    // the font size is 8x8 for each character and we have 1 line
    let area = centered_rect(area, line1.width() as u16 * 8, 1 * 8);
    big_text.render(area, buf);
}

/// Move a bunch of random pixels down one row.
///
/// Each pick some random pixels and move them each down one row. This is a very inefficient way to
/// do this, but it works well enough for this demo.
fn drip(frame_count: usize, area: Rect, buf: &mut Buffer, mask_buf: &mut Buffer) {
    // a seeded rng as we have to move the same random pixels each frame
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(10);
    let pixel_count = frame_count * SPEED_MULTIPLIER;
    for _ in 0..pixel_count {
        let src_x = rng.gen_range(0..area.width);
        let src_y = rng.gen_range(0..area.height - 1);
        let mask_cell = mask_buf.get_mut(src_x, src_y);
        let source_cell = buf.get_mut(src_x, src_y);
        // skip this pixel if it's in the mask
        let cloned_cell = source_cell.clone();
        if mask_cell.symbol() != " ".to_string() {
            source_cell.reset();
            source_cell
                .set_symbol(mask_cell.symbol())
                .set_fg(Color::Rgb(255, 128, 64));
            continue;
        }
        let (dest_x, dest_y) = if rng.gen_ratio(1, 100) {
            // move the pixel to a random location about 1% of the time
            (
                rng.gen_range(src_x.saturating_sub(5)..src_x.saturating_add(5))
                    .clamp(area.left(), area.right() - 1),
                area.top(),
            )
        } else {
            // otherwise move the pixel down a row
            (src_x, src_y.saturating_add(1).min(area.bottom() - 1))
        };
        let dest_cell = buf.get_mut(dest_x, dest_y);
        let mask_cell = mask_buf.get_mut(dest_x, dest_y);
        if mask_cell.symbol() != " ".to_string() {
            continue;
        }
        // copy the cell to the new location
        *dest_cell = cloned_cell;
    }
}

/// a centered rect of the given size
fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let horizontal = Layout::horizontal([width]).flex(Flex::Center);
    let vertical = Layout::vertical([height]).flex(Flex::Center);
    let [area] = area.split(&vertical);
    let [area] = area.split(&horizontal);
    area
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
