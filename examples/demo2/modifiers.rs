use crate::main_view::{layout, render_title};
use ratatui::{prelude::*, widgets::*};

pub fn render(area: Rect, buf: &mut Buffer) {
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
        "█".reset().reversed(),
        "Hidden".hidden(),
        " ".into(),
        "CrossedOut".crossed_out(),
    ]);
    Paragraph::new(text).render(area, buf);
}
