use std::{error::Error, io};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use itertools::Itertools;
use ratatui::{
    layout::{Constraint::*, Flex},
    prelude::*,
    widgets::*,
};

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let res = run_app(&mut terminal);

    // restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let mut selection = ExampleSelection::Stretch;
    loop {
        terminal.draw(|f| f.render_widget(selection, f.size()))?;

        if let Event::Key(key) = event::read()? {
            use KeyCode::*;
            match key.code {
                Char('q') => break Ok(()),
                Char('j') | Char('l') | Down | Right => {
                    selection = selection.next();
                }
                Char('k') | Char('h') | Up | Left => {
                    selection = selection.previous();
                }
                _ => (),
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum ExampleSelection {
    Stretch,
    StretchLast,
    Start,
    Center,
    End,
    SpaceAround,
    SpaceBetween,
}

impl ExampleSelection {
    fn previous(&self) -> Self {
        use ExampleSelection::*;
        match *self {
            Stretch => Stretch,
            StretchLast => Stretch,
            Start => StretchLast,
            Center => Start,
            End => Center,
            SpaceAround => End,
            SpaceBetween => SpaceAround,
        }
    }

    fn next(&self) -> Self {
        use ExampleSelection::*;
        match *self {
            Stretch => StretchLast,
            StretchLast => Start,
            Start => Center,
            Center => End,
            End => SpaceAround,
            SpaceAround => SpaceBetween,
            SpaceBetween => SpaceBetween,
        }
    }

    fn selected(&self) -> usize {
        use ExampleSelection::*;
        match self {
            Stretch => 0,
            StretchLast => 1,
            Start => 2,
            Center => 3,
            End => 4,
            SpaceAround => 5,
            SpaceBetween => 6,
        }
    }
}

impl Widget for ExampleSelection {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [tabs, area] = area.split(&Layout::vertical([Fixed(3), Proportional(0)]));

        self.render_tabs(tabs, buf);

        match self {
            ExampleSelection::Stretch => self.render_example(area, buf, Flex::Stretch),
            ExampleSelection::StretchLast => self.render_example(area, buf, Flex::StretchLast),
            ExampleSelection::Start => self.render_example(area, buf, Flex::Start),
            ExampleSelection::Center => self.render_example(area, buf, Flex::Center),
            ExampleSelection::End => self.render_example(area, buf, Flex::End),
            ExampleSelection::SpaceAround => self.render_example(area, buf, Flex::SpaceAround),
            ExampleSelection::SpaceBetween => self.render_example(area, buf, Flex::SpaceBetween),
        }
    }
}

impl ExampleSelection {
    fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        Tabs::new(
            [
                ExampleSelection::Stretch,
                ExampleSelection::StretchLast,
                ExampleSelection::Start,
                ExampleSelection::Center,
                ExampleSelection::End,
                ExampleSelection::SpaceAround,
                ExampleSelection::SpaceBetween,
            ]
            .iter()
            .map(|e| format!("{:?}", e)),
        )
        .block(Block::bordered().title("Flex Layouts"))
        .highlight_style(Style::default().yellow())
        .select(self.selected())
        .padding("  ", "  ")
        .render(area, buf);
    }

    fn render_example(&self, area: Rect, buf: &mut Buffer, flex: Flex) {
        let [example1, example2, example3, example4, example5, example6, _] =
            area.split(&Layout::vertical([Fixed(8); 7]));

        Example::new([Length(20), Length(10)])
            .flex(flex)
            .render(example1, buf);
        Example::new([Length(20), Fixed(10)])
            .flex(flex)
            .render(example2, buf);
        Example::new([Proportional(1), Proportional(1), Length(40), Fixed(20)])
            .flex(flex)
            .render(example3, buf);
        Example::new([Min(20), Length(40), Fixed(20)])
            .flex(flex)
            .render(example4, buf);
        Example::new([Min(20), Proportional(0), Length(40), Fixed(20)])
            .flex(flex)
            .render(example5, buf);
        Example::new([
            Min(20),
            Proportional(0),
            Percentage(10),
            Length(40),
            Fixed(20),
        ])
        .flex(flex)
        .render(example6, buf);
    }
}

struct Example {
    constraints: Vec<Constraint>,
    flex: Flex,
}

impl Example {
    fn new<C>(constraints: C) -> Self
    where
        C: Into<Vec<Constraint>>,
    {
        Self {
            constraints: constraints.into(),
            flex: Flex::default(),
        }
    }

    fn flex(mut self, flex: Flex) -> Self {
        self.flex = flex;
        self
    }
}

impl Widget for Example {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [title, legend, area] = area.split(&Layout::vertical([Ratio(1, 3); 3]));
        let blocks = Layout::horizontal(&self.constraints)
            .flex(self.flex)
            .split(area);

        self.heading().render(title, buf);

        self.legend(legend.width as usize).render(legend, buf);

        for (i, (block, _constraint)) in blocks.iter().zip(&self.constraints).enumerate() {
            let text = format!("{} px", block.width);
            let fg = Color::Indexed(i as u8 + 1);
            self.illustration(text, fg).render(*block, buf);
        }
    }
}

impl Example {
    fn heading(&self) -> Paragraph {
        // Renders the following
        //
        // Fixed(40), Proportional(0)
        let spans = self.constraints.iter().enumerate().map(|(i, c)| {
            let color = Color::Indexed(i as u8 + 1);
            Span::styled(format!("{:?}", c), color)
        });
        let heading =
            Line::from(Itertools::intersperse(spans, Span::raw(", ")).collect::<Vec<Span>>());
        Paragraph::new(heading).block(Block::default().padding(Padding::vertical(1)))
    }

    fn legend(&self, width: usize) -> Paragraph {
        // a bar like `<----- 80 px ----->`
        let width_label = format!("{} px", width);
        let width_bar = format!(
            "<{width_label:-^width$}>",
            width = width - width_label.len() / 2
        );
        Paragraph::new(width_bar.dark_gray()).alignment(Alignment::Center)
    }

    fn illustration(&self, text: String, fg: Color) -> Paragraph {
        Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(Block::bordered().style(Style::default().fg(fg)))
    }
}
