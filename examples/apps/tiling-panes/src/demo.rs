//! A scripted tour of the demo, for recording videos. Set `TILING_PANES_DEMO=1` to play it.
//!
//! The script sends the same mouse and key events a person would, so dragging, hovering and
//! menus behave exactly as they do by hand. Targets are worked out from the panes' current
//! positions, so the tour works at any terminal size.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::layout::{Position, Rect};

use crate::{App, Hit};

/// A point to aim the mouse at, found from the panes and clickable areas of the last render.
#[derive(Debug, Clone, Copy)]
enum Target {
    /// The tab strip of the pane in this slot of the layout, counting from the top left.
    Tabs(usize),
    /// Near one side of the pane in this slot, where a drop splits it.
    Edge(usize, Side),
    /// The middle of the pane in this slot, where a drop swaps the two panes.
    Centre(usize),
    /// A menu bar title.
    MenuTitle(usize),
    /// An item in the open menu.
    MenuItem(usize),
}

#[derive(Debug, Clone, Copy)]
enum Side {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy)]
enum Step {
    Wait(u64),
    Key(KeyCode),
    /// Glides the mouse to the target.
    Move(Target),
    /// Presses on the first target, drags to the second and lets go.
    Drag(Target, Target),
    Click(Target),
    RightClick(Target),
}

const SCRIPT: &[Step] = &[
    Step::Wait(1200),
    Step::Move(Target::Centre(0)),
    Step::Wait(400),
    // Move the notes pane to the right of the processes pane.
    Step::Drag(Target::Tabs(0), Target::Edge(1, Side::Right)),
    Step::Wait(900),
    // Swap two panes by dropping one on the other's centre.
    Step::Drag(Target::Tabs(1), Target::Centre(0)),
    Step::Wait(900),
    // Split the focused pane twice, then move the new pane under another.
    Step::Key(KeyCode::Char('n')),
    Step::Wait(700),
    Step::Key(KeyCode::Char('v')),
    Step::Wait(700),
    Step::Drag(Target::Tabs(2), Target::Edge(0, Side::Bottom)),
    Step::Wait(900),
    Step::Drag(Target::Tabs(3), Target::Edge(1, Side::Left)),
    Step::Wait(900),
    Step::Drag(Target::Tabs(0), Target::Edge(3, Side::Top)),
    Step::Wait(900),
    // The menu bar, with hover moving down the items.
    Step::Click(Target::MenuTitle(0)),
    Step::Wait(400),
    Step::Move(Target::MenuItem(0)),
    Step::Wait(300),
    Step::Move(Target::MenuItem(1)),
    Step::Wait(300),
    Step::Move(Target::MenuItem(2)),
    Step::Wait(500),
    Step::Move(Target::MenuTitle(1)),
    Step::Wait(700),
    Step::Key(KeyCode::Esc),
    // The context menu and the about dialog.
    Step::RightClick(Target::Centre(2)),
    Step::Wait(1200),
    Step::Key(KeyCode::Esc),
    Step::Key(KeyCode::Char('?')),
    Step::Wait(2200),
    Step::Key(KeyCode::Enter),
    Step::Wait(500),
    // Shade alternate cells to show the lines sit between cells, then compare with box drawing.
    Step::Key(KeyCode::Char('g')),
    Step::Wait(2200),
    Step::Key(KeyCode::Char('g')),
    Step::Wait(500),
    Step::Key(KeyCode::Char('b')),
    Step::Wait(2200),
    Step::Key(KeyCode::Char('b')),
    Step::Wait(800),
    // Close the panes the tour added, and finish.
    Step::Click(Target::Centre(4)),
    Step::Key(KeyCode::Char('x')),
    Step::Wait(500),
    Step::Click(Target::Centre(3)),
    Step::Key(KeyCode::Char('x')),
    Step::Wait(1500),
    Step::Key(KeyCode::Char('q')),
];

/// How long a drag or glide takes, and how many mouse events it sends.
const GLIDE: Duration = Duration::from_millis(700);
const GLIDE_STEPS: u32 = 24;

/// Plays the script, turning each step into timed events when it starts.
pub struct Tour {
    steps: VecDeque<Step>,
    events: VecDeque<(Instant, Event)>,
    mouse: Position,
}

impl Tour {
    pub fn new() -> Self {
        Self {
            steps: SCRIPT.iter().copied().collect(),
            events: VecDeque::new(),
            mouse: Position::new(0, 0),
        }
    }

    /// Returns the next event if it's due, or how long until it is.
    pub fn next(&mut self, app: &App) -> Result<Event, Duration> {
        let now = Instant::now();
        while self.events.is_empty() {
            let Some(step) = self.steps.pop_front() else {
                return Err(Duration::from_secs(60));
            };
            self.plan(step, app, now);
        }
        let (due, _) = self.events[0];
        if due > now {
            return Err(due - now);
        }
        Ok(self
            .events
            .pop_front()
            .map_or(Event::FocusGained, |(_, event)| event))
    }

    fn plan(&mut self, step: Step, app: &App, now: Instant) {
        match step {
            Step::Wait(millis) => {
                self.events
                    .push_back((now + Duration::from_millis(millis), Event::FocusGained));
            }
            Step::Key(code) => {
                let key = KeyEvent::new(code, KeyModifiers::NONE);
                self.events.push_back((now, Event::Key(key)));
            }
            Step::Move(target) => {
                let to = target.position(app);
                self.glide(now, to, MouseEventKind::Moved);
            }
            Step::Drag(from, to) => {
                let from = from.position(app);
                let to = to.position(app);
                self.glide(now, from, MouseEventKind::Moved);
                let start = now + GLIDE + Duration::from_millis(150);
                self.push_mouse(start, MouseEventKind::Down(MouseButton::Left), from);
                self.glide(start, to, MouseEventKind::Drag(MouseButton::Left));
                let end = start + GLIDE + Duration::from_millis(250);
                self.push_mouse(end, MouseEventKind::Up(MouseButton::Left), to);
            }
            Step::Click(target) => {
                let to = target.position(app);
                self.glide(now, to, MouseEventKind::Moved);
                let at = now + GLIDE + Duration::from_millis(150);
                self.push_mouse(at, MouseEventKind::Down(MouseButton::Left), to);
                self.push_mouse(at, MouseEventKind::Up(MouseButton::Left), to);
            }
            Step::RightClick(target) => {
                let to = target.position(app);
                self.glide(now, to, MouseEventKind::Moved);
                let at = now + GLIDE + Duration::from_millis(150);
                self.push_mouse(at, MouseEventKind::Down(MouseButton::Right), to);
            }
        }
    }

    /// Mouse events along a straight line from the current position to `to`.
    fn glide(&mut self, start: Instant, to: Position, kind: MouseEventKind) {
        let from = self.mouse;
        for step in 1..=GLIDE_STEPS {
            let t = f64::from(step) / f64::from(GLIDE_STEPS);
            // Ease in and out, so the pointer looks like a hand moved it.
            let t = t * t * (3.0 - 2.0 * t);
            let lerp = |a: u16, b: u16| (f64::from(a) + (f64::from(b) - f64::from(a)) * t).round();
            let position = Position::new(lerp(from.x, to.x) as u16, lerp(from.y, to.y) as u16);
            self.push_mouse(start + GLIDE * step / GLIDE_STEPS, kind, position);
        }
    }

    fn push_mouse(&mut self, at: Instant, kind: MouseEventKind, position: Position) {
        self.mouse = position;
        let mouse = MouseEvent {
            kind,
            column: position.x,
            row: position.y,
            modifiers: KeyModifiers::NONE,
        };
        self.events.push_back((at, Event::Mouse(mouse)));
    }
}

impl Target {
    fn position(self, app: &App) -> Position {
        let pane = |slot: usize| {
            app.areas
                .get(slot % app.areas.len().max(1))
                .map_or_else(Rect::default, |(_, area)| *area)
        };
        let middle = |area: Rect| Position::new(area.x + area.width / 2, area.y + area.height / 2);
        match self {
            Self::Tabs(slot) => {
                let area = pane(slot);
                Position::new(area.x + 4, area.y)
            }
            Self::Centre(slot) => middle(pane(slot)),
            Self::Edge(slot, side) => {
                let area = pane(slot);
                let centre = middle(area);
                match side {
                    Side::Left => Position::new(area.x + area.width / 10, centre.y),
                    Side::Right => Position::new(area.right() - 1 - area.width / 10, centre.y),
                    Side::Top => Position::new(centre.x, area.y + 1 + area.height / 10),
                    Side::Bottom => Position::new(centre.x, area.bottom() - 1 - area.height / 10),
                }
            }
            Self::MenuTitle(index) => app
                .hits
                .iter()
                .find(|(_, hit)| *hit == Hit::MenuTitle(index))
                .map_or_else(Position::default, |(area, _)| middle(*area)),
            Self::MenuItem(index) => app
                .hits
                .iter()
                .filter(|(_, hit)| matches!(hit, Hit::Item(_)))
                .nth(index)
                .map_or_else(Position::default, |(area, _)| middle(*area)),
        }
    }
}
