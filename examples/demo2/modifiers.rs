use ratatui::{prelude::*, widgets::*};

pub fn render(area: Rect, buf: &mut Buffer) {
    Paragraph::new(Line::from(vec![
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
    ]))
    .render(area, buf);
}
