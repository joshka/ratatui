/// A Ratatui example of a tiling, multiplexer-style app shell drawn with the cell border
/// protocol.
///
/// Panes can be rearranged by dragging them with the mouse: drop a pane onto an edge of
/// another pane to split it there, or onto its centre to swap the two. There is a menu bar, a
/// context menu on right click, and an about dialog.
///
/// On terminals that support the cell border protocol, every line in the UI is drawn on the
/// edges of cells, so it takes up no columns or rows. That allows things box-drawing
/// characters can't do: tabs that join the pane below them, one-row buttons, one-cell
/// checkboxes, underline-only inputs, table column rules without separator columns, and a
/// focus ring that wins every shared edge. Elsewhere it falls back to box-drawing characters.
///
/// This example runs with the Ratatui library code in the branch that you are currently
/// reading. See the [`latest`] branch for the code which works with the most recent Ratatui
/// release.
///
/// [`latest`]: https://github.com/ratatui/ratatui/tree/latest
use std::collections::HashMap;
use std::io::{Read, Write};

use color_eyre::Result;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseButton, MouseEvent,
    MouseEventKind,
};
use crossterm::execute;
use ratatui::buffer::{BorderJoints, BorderSegments, Buffer, CellBorder, LineStyle};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin, Position, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Widget, Wrap};
use ratatui::{DefaultTerminal, Frame};

const BG: Color = Color::Rgb(8, 10, 15);
const PANEL_FOCUS: Color = Color::Rgb(20, 24, 34);
const RAISED: Color = Color::Rgb(30, 36, 50);
const TEXT: Color = Color::Rgb(232, 236, 242);
const MUTED: Color = Color::Rgb(122, 134, 154);
const LINE: Color = Color::Rgb(52, 60, 78);
const SHADOW: Color = Color::Rgb(0, 0, 0);
const TEAL: Color = Color::Rgb(0, 224, 184);
const RED: Color = Color::Rgb(255, 72, 120);
const GOLD: Color = Color::Rgb(255, 190, 30);
const BLUE: Color = Color::Rgb(64, 140, 255);

/// Stroke widths in 1/256 of the cell width.
const THIN: u8 = 20;
const THICK: u8 = 32;
/// Wide enough for a double line to read as two lines.
const WIDE: u8 = 56;

const LIPSUM: [&str; 4] = [
    "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut \
     labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco \
     laboris nisi ut aliquip ex ea commodo consequat.",
    "Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla \
     pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt \
     mollit anim id est laborum.",
    "Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque \
     laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi \
     architecto beatae vitae dicta sunt explicabo.",
    "Nemo enim ipsam voluptatem quia voluptas sit aspernatur aut odit aut fugit, sed quia \
     consequuntur magni dolores eos qui ratione voluptatem sequi nesciunt.",
];

const PROCESSES: [(u32, &str, f32, u32); 10] = [
    (412, "ghostty", 12.4, 312),
    (977, "cargo", 41.0, 905),
    (1203, "rust-analyzer", 18.7, 1650),
    (88, "launchd", 0.3, 24),
    (5120, "zig", 22.9, 740),
    (2311, "jj", 1.2, 48),
    (6402, "tiling-panes", 3.1, 18),
    (733, "WindowServer", 7.8, 410),
    (1500, "mds_stores", 0.9, 96),
    (3021, "fish", 0.1, 9),
];

const LOG_MESSAGES: [(&str, &str); 8] = [
    ("INFO", "listening on 127.0.0.1:8080"),
    ("INFO", "GET /api/panes 200 3ms"),
    ("WARN", "slow query: layout tree took 41ms"),
    ("INFO", "pane 2 moved beside pane 0"),
    ("ERROR", "failed to attach: session not found"),
    ("INFO", "GET /api/themes 200 1ms"),
    ("DEBUG", "border pen cache hit"),
    ("WARN", "retrying connection (2/5)"),
];

const MENUS: [(&str, &[&[Action]]); 3] = [
    (
        "Pane",
        &[&[Action::SplitRight, Action::SplitDown], &[Action::Close]],
    ),
    ("View", &[&[Action::ToggleBorders]]),
    ("Help", &[&[Action::About], &[Action::Quit]]),
];

const CONTEXT_MENU: &[&[Action]] = &[&[Action::SplitRight, Action::SplitDown], &[Action::Close]];

fn main() -> Result<()> {
    color_eyre::install()?;
    ratatui::run(|terminal| {
        let features = if std::env::var_os("NO_CELL_BORDERS").is_some() {
            None
        } else {
            detect_cell_borders()?
        };
        let mut app = App::new(features.is_some());
        // Only fill outside rounded corners where the terminal says it can.
        app.borders.round_outside =
            features.is_some_and(|features| features.contains(&"outside".into()));
        app.run(terminal)
    })
}

/// Asks the terminal whether it supports the cell border protocol.
///
/// Sends the capability query followed by Primary Device Attributes. Terminals answer in order, so
/// a DA1 reply without a border reply before it means no support. Must run in raw mode before
/// anything else reads from stdin.
fn detect_cell_borders() -> Result<Option<Vec<String>>> {
    let mut stdout = std::io::stdout();
    stdout.write_all(b"\x1bP1+bcell-border;q\x1b\\\x1b[c")?;
    stdout.flush()?;
    let mut reply = Vec::new();
    let mut byte = [0];
    // The DA1 reply is `CSI ? ... c`, and is always the last thing the terminal sends.
    while std::io::stdin().read(&mut byte)? == 1 {
        reply.push(byte[0]);
        if byte[0] == b'c' && reply.windows(3).any(|w| w == b"\x1b[?") {
            break;
        }
    }
    // The reply is `cell-border;ok;VERSION;FEATURE,FEATURE,...`.
    let reply = String::from_utf8_lossy(&reply);
    let Some((_, rest)) = reply.split_once("cell-border;ok;") else {
        return Ok(None);
    };
    let features = rest.split(['\x1b', ';']).nth(1).unwrap_or_default();
    Ok(Some(features.split(',').map(str::to_string).collect()))
}

/// What a pane shows. Each kind has its own accent colour, used for its focus ring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Notes,
    Processes,
    Logs,
    Settings,
}

impl Kind {
    const fn of(pane: usize) -> Self {
        match pane % 4 {
            0 => Self::Notes,
            1 => Self::Processes,
            2 => Self::Logs,
            _ => Self::Settings,
        }
    }

    const fn accent(self) -> Color {
        match self {
            Self::Notes => TEAL,
            Self::Processes => BLUE,
            Self::Logs => GOLD,
            Self::Settings => RED,
        }
    }

    /// The pane background: near black with a hint of the accent, a little lighter when focused.
    fn panel(self, focused: bool) -> Color {
        mix(self.accent(), BG, if focused { 0.16 } else { 0.1 })
    }

    const fn tabs(self) -> [&'static str; 2] {
        match self {
            Self::Notes => ["notes.md", "todo.md"],
            Self::Processes => ["top", "services"],
            Self::Logs => ["app.log", "system.log"],
            Self::Settings => ["general", "keys"],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    SplitRight,
    SplitDown,
    Close,
    ToggleBorders,
    About,
    Quit,
}

impl Action {
    const fn label(self) -> &'static str {
        match self {
            Self::SplitRight => "Split right",
            Self::SplitDown => "Split down",
            Self::Close => "Close pane",
            Self::ToggleBorders => "Toggle cell borders",
            Self::About => "About",
            Self::Quit => "Quit",
        }
    }

    const fn key(self) -> &'static str {
        match self {
            Self::SplitRight => "n",
            Self::SplitDown => "v",
            Self::Close => "x",
            Self::ToggleBorders => "b",
            Self::About => "?",
            Self::Quit => "q",
        }
    }
}

/// Something clickable, recorded during render for hit testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Hit {
    MenuTitle(usize),
    Item(Action),
    Tab(usize, usize),
    Checkbox,
    DialogOk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Overlay {
    None,
    Menu(usize),
    Context(Position),
    About,
}

/// A layout tree: panes are leaves, and splits divide their area evenly between children, with a
/// gap between them.
#[derive(Debug, Clone)]
enum Node {
    Pane(usize),
    Split(Direction, Vec<Self>),
}

/// Where a dragged pane lands relative to the pane under the mouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Drop {
    Left,
    Right,
    Top,
    Bottom,
    Swap,
}

struct Drag {
    pane: usize,
    mouse: Position,
    moved: bool,
    target: Option<(usize, Drop)>,
}

/// Whether to draw with the cell border protocol, and which of its optional features to use.
struct Borders {
    enabled: bool,
    /// Whether rounded corners fill the part outside their arc. Needs the terminal's `outside`
    /// feature.
    round_outside: bool,
}

struct App {
    root: Node,
    next_id: usize,
    focus: usize,
    tabs: HashMap<usize, usize>,
    drag: Option<Drag>,
    overlay: Overlay,
    hover: Position,
    borders: Borders,
    /// Shades alternate cells, to show that the lines sit on cell edges and take no cells.
    show_grid: bool,
    /// Pane areas from the last render, used for hit testing.
    areas: Vec<(usize, Rect)>,
    /// Clickable areas from the last render, topmost last.
    hits: Vec<(Rect, Hit)>,
    should_exit: bool,
}

impl App {
    fn new(cell_borders: bool) -> Self {
        let root = Node::Split(
            Direction::Horizontal,
            vec![
                Node::Split(Direction::Vertical, vec![Node::Pane(0), Node::Pane(2)]),
                Node::Split(Direction::Vertical, vec![Node::Pane(1), Node::Pane(3)]),
            ],
        );
        Self {
            root,
            next_id: 4,
            focus: 0,
            tabs: HashMap::new(),
            drag: None,
            overlay: Overlay::None,
            hover: Position::default(),
            borders: Borders {
                enabled: cell_borders,
                round_outside: false,
            },
            show_grid: false,
            areas: Vec::new(),
            hits: Vec::new(),
            should_exit: false,
        }
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        execute!(std::io::stdout(), EnableMouseCapture)?;
        while !self.should_exit {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_event(&event::read()?);
        }
        execute!(std::io::stdout(), DisableMouseCapture)?;
        Ok(())
    }

    fn handle_event(&mut self, event: &Event) {
        match event {
            Event::Key(key) if key.is_press() => self.on_key(key.code),
            Event::Mouse(mouse) => self.on_mouse(*mouse),
            _ => {}
        }
    }

    fn on_key(&mut self, code: KeyCode) {
        let action = match code {
            KeyCode::Esc if self.overlay != Overlay::None => {
                self.overlay = Overlay::None;
                return;
            }
            KeyCode::Enter if self.overlay == Overlay::About => {
                self.overlay = Overlay::None;
                return;
            }
            KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
            KeyCode::Char('n') => Action::SplitRight,
            KeyCode::Char('v') => Action::SplitDown,
            KeyCode::Char('x') => Action::Close,
            KeyCode::Char('b') => Action::ToggleBorders,
            KeyCode::Char('o') => {
                self.borders.round_outside = !self.borders.round_outside;
                return;
            }
            KeyCode::Char('g') => {
                self.show_grid = !self.show_grid;
                return;
            }
            KeyCode::Char('?') => Action::About,
            _ => return,
        };
        self.run_action(action);
    }

    fn run_action(&mut self, action: Action) {
        self.overlay = Overlay::None;
        match action {
            Action::SplitRight => self.split_focused(Drop::Right),
            Action::SplitDown => self.split_focused(Drop::Bottom),
            Action::Close => self.close_focused(),
            Action::ToggleBorders => self.borders.enabled = !self.borders.enabled,
            Action::About => self.overlay = Overlay::About,
            Action::Quit => self.should_exit = true,
        }
    }

    fn on_mouse(&mut self, mouse: MouseEvent) {
        let position = Position::new(mouse.column, mouse.row);
        self.hover = position;
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => self.on_press(position),
            MouseEventKind::Down(MouseButton::Right) => {
                if let Some((pane, _)) = self.pane_at(position) {
                    self.focus = pane;
                    self.overlay = Overlay::Context(position);
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => self.on_drag(position),
            MouseEventKind::Up(MouseButton::Left) => self.on_release(),
            MouseEventKind::Moved => {
                // Sliding along the menu bar while a menu is open switches menus.
                if let (Overlay::Menu(_), Some(Hit::MenuTitle(index))) =
                    (self.overlay, self.hit_at(position))
                {
                    self.overlay = Overlay::Menu(index);
                }
            }
            _ => {}
        }
    }

    fn on_press(&mut self, position: Position) {
        match (self.overlay, self.hit_at(position)) {
            (Overlay::About, Some(Hit::DialogOk)) => self.overlay = Overlay::None,
            (Overlay::About, _) => {}
            (Overlay::Menu(open), Some(Hit::MenuTitle(index))) if open == index => {
                self.overlay = Overlay::None;
            }
            (_, Some(Hit::MenuTitle(index))) => self.overlay = Overlay::Menu(index),
            (_, Some(Hit::Item(action))) => self.run_action(action),
            (Overlay::Menu(_) | Overlay::Context(_), _) => self.overlay = Overlay::None,
            (Overlay::None, hit) => {
                match hit {
                    Some(Hit::Tab(pane, tab)) => {
                        self.tabs.insert(pane, tab);
                    }
                    Some(Hit::Checkbox) => self.borders.enabled = !self.borders.enabled,
                    _ => {}
                }
                self.start_drag(position);
            }
        }
    }

    fn start_drag(&mut self, position: Position) {
        if let Some((pane, _)) = self.pane_at(position) {
            self.focus = pane;
            self.drag = Some(Drag {
                pane,
                mouse: position,
                moved: false,
                target: None,
            });
        }
    }

    fn on_drag(&mut self, position: Position) {
        let target = self.drop_target(position);
        if let Some(drag) = &mut self.drag {
            drag.moved |= drag.mouse != position;
            drag.mouse = position;
            drag.target = target.filter(|(pane, _)| *pane != drag.pane);
        }
    }

    fn on_release(&mut self) {
        if let Some(Drag {
            pane,
            moved: true,
            target: Some((target, drop)),
            ..
        }) = self.drag.take()
        {
            self.move_pane(pane, target, drop);
        }
    }

    fn hit_at(&self, position: Position) -> Option<Hit> {
        self.hits
            .iter()
            .rev()
            .find(|(area, _)| area.contains(position))
            .map(|(_, hit)| *hit)
    }

    fn pane_at(&self, position: Position) -> Option<(usize, Rect)> {
        self.areas
            .iter()
            .copied()
            .find(|(_, area)| area.contains(position))
    }

    /// Picks the drop zone from where the mouse sits in the pane: near an edge splits on that
    /// side, the middle swaps.
    fn drop_target(&self, position: Position) -> Option<(usize, Drop)> {
        let (pane, area) = self.pane_at(position)?;
        let x = f32::from(position.x - area.x) / f32::from(area.width.max(1));
        let y = f32::from(position.y - area.y) / f32::from(area.height.max(1));
        let edges = [
            (x, Drop::Left),
            (1.0 - x, Drop::Right),
            (y, Drop::Top),
            (1.0 - y, Drop::Bottom),
        ];
        let (distance, drop) = edges
            .into_iter()
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .unwrap();
        Some((pane, if distance < 0.3 { drop } else { Drop::Swap }))
    }

    fn move_pane(&mut self, pane: usize, target: usize, drop: Drop) {
        if drop == Drop::Swap {
            self.root.for_each_pane(&mut |id| {
                if *id == pane {
                    *id = target;
                } else if *id == target {
                    *id = pane;
                } else {
                    // Other panes stay where they are.
                }
            });
            return;
        }
        self.root.remove(pane);
        self.root.insert_beside(target, pane, drop);
        self.root.normalize();
    }

    fn split_focused(&mut self, drop: Drop) {
        let id = self.next_id;
        self.next_id += 1;
        self.root.insert_beside(self.focus, id, drop);
        self.root.normalize();
        self.focus = id;
    }

    fn close_focused(&mut self) {
        if matches!(self.root, Node::Pane(_)) {
            return;
        }
        self.root.remove(self.focus);
        self.root.normalize();
        self.focus = self.root.first_pane();
    }

    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let buf = frame.buffer_mut();
        buf.set_style(area, Style::new().bg(BG).fg(TEXT));
        let [menu_bar, main, status] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .spacing(1)
        .areas(area);
        self.hits.clear();
        self.areas.clear();
        let main = main.inner(Margin::new(2, 0));
        self.root.layout(main, &mut self.areas);

        self.render_menu_bar(menu_bar, buf);
        for (pane, area) in self.areas.clone() {
            self.render_pane(pane, area, buf);
        }
        if let Some(drag) = &self.drag {
            self.render_drop_preview(drag, buf);
        }
        self.render_status(status, buf);
        match self.overlay {
            Overlay::None => {}
            Overlay::Menu(index) => self.render_menu(index, area, buf),
            Overlay::Context(at) => {
                let menu = self.menu_area(at, CONTEXT_MENU, area);
                self.render_menu_items(menu, CONTEXT_MENU, None, buf);
            }
            Overlay::About => self.render_about(area, buf),
        }
        if let Some(drag) = self.drag.as_ref().filter(|drag| drag.moved) {
            render_ghost(drag, area, buf);
        }
        if self.show_grid {
            render_grid(area, buf);
        }
    }

    fn render_menu_bar(&mut self, area: Rect, buf: &mut Buffer) {
        Line::from(" ◆ panes ".fg(TEAL).bold()).render(area, buf);
        self.edge(area, BorderSegments::BOTTOM, stroke(LINE, THIN, 0), buf);
        let mut x = area.x + 9;
        for (index, (title, _)) in MENUS.iter().enumerate() {
            let title_area = Rect::new(x, area.y, title.len() as u16 + 2, 1).intersection(area);
            x += title_area.width;
            self.hits.push((title_area, Hit::MenuTitle(index)));
            let open = self.overlay == Overlay::Menu(index);
            let style = if open {
                Style::new().fg(TEXT).bg(RAISED)
            } else if title_area.contains(self.hover) {
                Style::new().fg(TEXT)
            } else {
                Style::new().fg(MUTED)
            };
            Line::styled(format!(" {title} "), style).render(title_area, buf);
            if open {
                // Open the line below the title so it joins the menu, like a tab.
                self.set_edges(title_area, BorderSegments::BOTTOM, false, buf);
                let sides = BorderSegments::LEFT | BorderSegments::RIGHT;
                self.outline_sides(title_area, sides, stroke(TEAL, THIN, 4), buf);
            }
        }
        let mode = if self.borders.enabled {
            "cell borders ●"
        } else {
            "box drawing ○"
        };
        Line::from(Span::from(format!("{mode} ")).fg(MUTED))
            .alignment(Alignment::Right)
            .render(area, buf);
    }

    fn render_pane(&mut self, pane: usize, area: Rect, buf: &mut Buffer) {
        let kind = Kind::of(pane);
        let focused = pane == self.focus;
        buf.set_style(area, Style::new().bg(kind.panel(focused)));
        let border = if focused {
            CellBorder {
                style: LineStyle::Double,
                ..stroke(kind.accent(), WIDE, 2)
            }
        } else {
            stroke(mix(kind.accent(), BG, 0.6), THIN, 1)
        };
        let inner = self.chrome(area, border, buf);
        let [header, body] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(inner);
        let tab = self.tabs.get(&pane).copied().unwrap_or(0);
        self.render_tabs(pane, tab, header, border, buf);
        // The outline is inside the pane's cells, so keep content a column clear of it.
        let body = body.inner(Margin::new(1, 0));
        match kind {
            Kind::Notes => self.render_notes(pane + tab, body, buf),
            Kind::Processes => self.render_processes(tab, body, buf),
            Kind::Logs => self.render_logs(pane + tab, body, buf),
            Kind::Settings => self.render_settings(body, buf),
        }
    }

    /// Draws the tab strip. The active tab is cut out of the line below the strip so it joins
    /// the pane body.
    fn render_tabs(
        &mut self,
        pane: usize,
        active: usize,
        area: Rect,
        border: CellBorder,
        buf: &mut Buffer,
    ) {
        let accent = Kind::of(pane).accent();
        self.edge(area, BorderSegments::BOTTOM, border, buf);
        let mut x = area.x;
        for (index, name) in Kind::of(pane).tabs().into_iter().enumerate() {
            let tab = Rect::new(x, area.y, name.len() as u16 + 2, 1).intersection(area);
            x += tab.width;
            self.hits.push((tab, Hit::Tab(pane, index)));
            if index == active {
                Line::from(format!(" {name} "))
                    .fg(BG)
                    .bg(accent)
                    .bold()
                    .render(tab, buf);
                self.set_edges(tab, BorderSegments::BOTTOM, false, buf);
                let sides = BorderSegments::LEFT | BorderSegments::RIGHT;
                self.outline_sides(tab, sides, border, buf);
            } else {
                Line::from(format!(" {name} ")).fg(MUTED).render(tab, buf);
            }
        }
        if !self.borders.enabled {
            return;
        }
        let pane_label = Line::from(Span::from(format!("#{pane} ")).fg(LINE));
        pane_label.alignment(Alignment::Right).render(area, buf);
    }

    fn render_notes(&self, seed: usize, area: Rect, buf: &mut Buffer) {
        let [heading, text, quote, _] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(2),
            Constraint::Length(1),
        ])
        .areas(area);
        let tags = [("rust", TEAL), ("tui", BLUE), ("ghostty", GOLD)];
        let mut title = vec![Span::from("Cell borders  ").fg(TEAL).bold()];
        for (tag, color) in tags {
            title.push(
                Span::from(format!(" {tag} "))
                    .fg(color)
                    .bg(mix(color, BG, 0.25)),
            );
            title.push(Span::from(" "));
        }
        Line::from(title).render(heading, buf);
        let paragraphs = (0..LIPSUM.len()).map(|i| Line::from(LIPSUM[(seed + i) % LIPSUM.len()]));
        Paragraph::new(paragraphs.collect::<Vec<_>>())
            .wrap(Wrap { trim: true })
            .render(text, buf);
        // A block quote with its rule on the cell edge, so the quoted text keeps every column.
        self.edge(
            quote.columns().next().unwrap_or_default(),
            BorderSegments::LEFT,
            stroke(TEAL, THICK, 0),
            buf,
        );
        Paragraph::new("Thin lines, full cells. Text keeps its own foreground.")
            .fg(MUTED)
            .italic()
            .wrap(Wrap { trim: true })
            .render(quote, buf);
    }

    /// A table whose column rules sit on cell edges, so there are no separator columns.
    fn render_processes(&self, selected: usize, area: Rect, buf: &mut Buffer) {
        let widths = [
            Constraint::Length(6),
            Constraint::Fill(1),
            Constraint::Length(6),
            Constraint::Length(7),
        ];
        let columns = Layout::horizontal(widths).split(area);
        let headers = ["PID ", " NAME", "CPU% ", "MEM "];
        let rows: Vec<[String; 4]> = PROCESSES
            .iter()
            .map(|(pid, name, cpu, mem)| {
                [
                    format!("{pid} "),
                    format!(" {name}"),
                    format!("{cpu:.1} "),
                    format!("{mem}M "),
                ]
            })
            .collect();
        for (index, column) in columns.iter().enumerate() {
            let alignment = if index == 1 {
                Alignment::Left
            } else {
                Alignment::Right
            };
            let header = Line::from(headers[index]).fg(MUTED).bold();
            let mut lines = vec![header.alignment(alignment)];
            lines.extend(PROCESSES.iter().zip(&rows).map(|((_, _, cpu, _), row)| {
                let line = Line::from(row[index].clone()).alignment(alignment);
                match index {
                    0 => line.fg(MUTED),
                    2 if *cpu > 30.0 => line.fg(RED).bold(),
                    2 if *cpu > 10.0 => line.fg(GOLD),
                    2 => line.fg(TEAL),
                    _ => line,
                }
            }));
            Paragraph::new(lines).render(*column, buf);
            if index > 0 {
                let rule = Rect::new(column.x, area.y, 1, rows.len() as u16 + 1).intersection(area);
                self.edge(rule, BorderSegments::LEFT, stroke(LINE, THIN, 0), buf);
            }
        }
        let header = Rect::new(area.x, area.y, area.width, 1).intersection(area);
        self.edge(header, BorderSegments::BOTTOM, stroke(BLUE, THIN, 0), buf);
        let row = Rect::new(area.x, area.y + 1 + selected as u16 * 2, area.width, 1);
        buf.set_style(row.intersection(area), Style::new().bg(mix(BLUE, BG, 0.3)));
    }

    /// Log lines, with errors marked by a rule on the cell edge rather than a gutter column.
    fn render_logs(&self, seed: usize, area: Rect, buf: &mut Buffer) {
        for (offset, row) in area.rows().enumerate() {
            let (level, message) = LOG_MESSAGES[(seed + offset) % LOG_MESSAGES.len()];
            let color = match level {
                "ERROR" => RED,
                "WARN" => GOLD,
                "DEBUG" => MUTED,
                _ => TEAL,
            };
            let time = format!("09:41:{:02} ", (seed * 7 + offset) % 60);
            Line::from(vec![
                Span::from(time).fg(MUTED),
                Span::from(format!(" {level:<5} "))
                    .fg(color)
                    .bg(mix(color, BG, 0.22))
                    .bold(),
                Span::from(format!(" {message}")),
            ])
            .render(row, buf);
            if level == "ERROR" {
                buf.set_style(row, Style::new().bg(mix(RED, BG, 0.16)));
                let gutter = Rect { width: 1, ..row };
                self.edge(gutter, BorderSegments::LEFT, stroke(RED, THICK, 0), buf);
            }
        }
    }

    /// A small form: underline-only inputs, a one-cell checkbox and one-row buttons.
    fn render_settings(&mut self, area: Rect, buf: &mut Buffer) {
        let rows = Layout::vertical([Constraint::Length(1); 9]).split(area);
        let field_width = area.width.min(28);
        let fields = [
            ("Display name", "Josh", true),
            ("Theme", "Ghostty Dark", false),
        ];
        for (index, (label, value, active)) in fields.into_iter().enumerate() {
            Line::from(label).fg(MUTED).render(rows[index * 3], buf);
            let input = Rect::new(area.x, rows[index * 3 + 1].y, field_width, 1).intersection(area);
            let color = if active { RED } else { LINE };
            buf.set_style(input, Style::new().bg(PANEL_FOCUS));
            Line::from(value).render(input, buf);
            if self.borders.enabled {
                self.edge(input, BorderSegments::BOTTOM, stroke(color, THIN, 0), buf);
            } else {
                buf.set_style(input, Style::new().add_modifier(Modifier::UNDERLINED));
            }
        }

        self.hits.push((rows[6], Hit::Checkbox));
        if self.borders.enabled {
            self.render_checkbox(Position::new(area.x + 1, rows[6].y), buf);
        } else {
            Line::from("[ ]").render(rows[6], buf);
        }
        let label = Rect {
            x: rows[6].x + 4,
            width: rows[6].width.saturating_sub(4),
            ..rows[6]
        };
        Line::from("Use cell borders").render(label, buf);

        let mut x = area.x;
        for (label, color) in [("Save", RED), ("Reset", MUTED)] {
            let button = Rect::new(x, rows[8].y, label.len() as u16 + 2, 1).intersection(area);
            x += button.width + 2;
            self.render_button(button, label, color, buf);
        }
    }

    /// A square checkbox around the cell at `centre`. Its sides run down the middle of the
    /// cells either side, so the box is two cells wide and one tall, about square, with the mark
    /// centred in it.
    fn render_checkbox(&self, centre: Position, buf: &mut Buffer) {
        let border = stroke(RED, THIN, 0);
        let half_left = BorderSegments::TOP_RIGHT | BorderSegments::BOTTOM_RIGHT;
        let half_right = BorderSegments::TOP_LEFT | BorderSegments::BOTTOM_LEFT;
        let cells = [
            (
                centre.x.saturating_sub(1),
                half_left | BorderSegments::VERTICAL,
            ),
            (centre.x, BorderSegments::TOP | BorderSegments::BOTTOM),
            (centre.x + 1, half_right | BorderSegments::VERTICAL),
        ];
        for (x, segments) in cells {
            self.edge(Rect::new(x, centre.y, 1, 1), segments, border, buf);
        }
        Line::from("✓")
            .fg(RED)
            .bold()
            .render(Rect::new(centre.x, centre.y, 1, 1), buf);
    }

    /// A one-row button: text with lines on all four of its edges.
    fn render_button(&self, area: Rect, label: &str, color: Color, buf: &mut Buffer) {
        let hovered = area.contains(self.hover);
        let bg = if hovered { RAISED } else { PANEL_FOCUS };
        buf.set_style(area, Style::new().bg(bg));
        if self.borders.enabled {
            Line::from(label).fg(color).centered().render(area, buf);
            self.outline(area, stroke(color, THIN, 0), buf);
        } else {
            Line::from(format!("[{label}]")).fg(color).render(area, buf);
        }
    }

    /// Shades the part of the target pane that the dragged pane would occupy, outlined in the
    /// dragged pane's accent colour.
    fn render_drop_preview(&self, drag: &Drag, buf: &mut Buffer) {
        let Some((target, drop)) = drag.target else {
            return;
        };
        let Some(&(_, area)) = self.areas.iter().find(|(pane, _)| *pane == target) else {
            return;
        };
        let halves = |direction| {
            Layout::new(direction, [Constraint::Fill(1), Constraint::Fill(1)]).areas::<2>(area)
        };
        let shade = match drop {
            Drop::Left => halves(Direction::Horizontal)[0],
            Drop::Right => halves(Direction::Horizontal)[1],
            Drop::Top => halves(Direction::Vertical)[0],
            Drop::Bottom => halves(Direction::Vertical)[1],
            Drop::Swap => area,
        };
        let accent = Kind::of(drag.pane).accent();
        buf.set_style(shade, Style::new().bg(Color::Rgb(42, 58, 76)));
        let preview = CellBorder {
            style: LineStyle::Dashed,
            ..stroke(accent, THICK, 3)
        };
        self.outline(shade, preview, buf);
    }

    fn render_status(&self, area: Rect, buf: &mut Buffer) {
        let kind = Kind::of(self.focus);
        let mut spans = vec![
            Span::from(format!(" {kind:?} ").to_uppercase())
                .fg(BG)
                .bg(kind.accent())
                .bold(),
            Span::from(format!(" #{} ", self.focus))
                .fg(kind.accent())
                .bg(mix(kind.accent(), BG, 0.25)),
            Span::from("  "),
        ];
        let hints = [
            ("drag", "move"),
            ("right click", "menu"),
            ("n/v", "split"),
            ("x", "close"),
            ("?", "about"),
            ("g", "cell grid"),
            ("q", "quit"),
        ];
        for (key, label) in hints {
            spans.push(Span::from(key).fg(kind.accent()).bold());
            spans.push(Span::from(format!(" {label}   ")).fg(MUTED));
        }
        Line::from(spans).render(area, buf);
    }

    fn render_menu(&mut self, index: usize, screen: Rect, buf: &mut Buffer) {
        let Some(&(title_area, _)) = self
            .hits
            .iter()
            .find(|(_, hit)| *hit == Hit::MenuTitle(index))
        else {
            return;
        };
        let groups = MENUS[index].1;
        let anchor = Position::new(title_area.x, title_area.y + 1);
        let menu = self.menu_area(anchor, groups, screen);
        self.render_menu_items(menu, groups, Some(title_area), buf);
    }

    fn menu_area(&self, at: Position, groups: &[&[Action]], screen: Rect) -> Rect {
        let rows = groups.iter().map(|group| group.len()).sum::<usize>() as u16;
        let chrome = if self.borders.enabled { 0 } else { 2 };
        let area = Rect::new(at.x, at.y, 28 + chrome, rows + chrome);
        area.clamp(screen)
    }

    /// Draws a menu. Groups are separated by a line on the edge between two items rather than a
    /// separator row. When the menu belongs to a menu bar title, its top edge opens below the
    /// title so the two join.
    fn render_menu_items(
        &mut self,
        area: Rect,
        groups: &[&[Action]],
        title: Option<Rect>,
        buf: &mut Buffer,
    ) {
        render_shadow(area, buf);
        Clear.render(area, buf);
        buf.set_style(area, Style::new().bg(RAISED).fg(TEXT));
        let border = stroke(TEAL, THIN, 4);
        let inner = self.chrome(area, border, buf);
        if let Some(title) = title {
            let gap = Rect::new(title.x, area.y, title.width, 1).intersection(area);
            self.set_edges(gap, BorderSegments::TOP, false, buf);
        }
        let mut rows = inner.rows();
        for (group_index, group) in groups.iter().enumerate() {
            for (item_index, action) in group.iter().enumerate() {
                let Some(row) = rows.next() else {
                    return;
                };
                self.hits.push((row, Hit::Item(*action)));
                if row.contains(self.hover) {
                    buf.set_style(row, Style::new().bg(Color::Rgb(38, 62, 65)).fg(TEAL));
                }
                Line::from(format!(" {}", action.label())).render(row, buf);
                Line::from(Span::from(format!("{} ", action.key())).fg(MUTED))
                    .alignment(Alignment::Right)
                    .render(row, buf);
                let last_in_group = item_index + 1 == group.len();
                if last_in_group && group_index + 1 < groups.len() {
                    self.edge(row, BorderSegments::BOTTOM, stroke(LINE, THIN, 3), buf);
                }
            }
        }
    }

    fn render_about(&mut self, screen: Rect, buf: &mut Buffer) {
        let area = screen.centered(Constraint::Length(50), Constraint::Length(11));
        render_shadow(area, buf);
        Clear.render(area, buf);
        buf.set_style(area, Style::new().bg(RAISED).fg(TEXT));
        let inner = self.chrome(area, stroke(TEAL, THICK, 4), buf);
        let [title, body, buttons] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(inner);
        Line::from(" About panes")
            .fg(TEAL)
            .bold()
            .render(title, buf);
        self.edge(title, BorderSegments::BOTTOM, stroke(TEAL, THICK, 4), buf);
        Paragraph::new(vec![
            Line::from(""),
            Line::from("Every line here sits on the edge of a cell, so the dialog,"),
            Line::from("its buttons and the panes behind it lose no columns or rows."),
            Line::from(""),
            Line::from("Drawn with the cell border protocol.").fg(MUTED),
        ])
        .centered()
        .render(body, buf);
        let ok =
            Rect::new(buttons.right().saturating_sub(8), buttons.y, 6, 1).intersection(buttons);
        self.hits.push((ok, Hit::DialogOk));
        self.render_button(ok, "OK", TEAL, buf);
    }

    /// Draws a frame around `area` and returns the area inside it.
    ///
    /// With cell borders the frame sits on the outer cell edges and takes no space. Otherwise it
    /// is a box-drawing block.
    fn chrome(&self, area: Rect, border: CellBorder, buf: &mut Buffer) -> Rect {
        if self.borders.enabled {
            self.outline(area, border, buf);
            return area;
        }
        let block = Block::bordered().border_style(Style::new().fg(border.color));
        let inner = block.inner(area);
        block.render(area, buf);
        inner
    }

    fn outline(&self, area: Rect, border: CellBorder, buf: &mut Buffer) {
        self.outline_sides(area, BorderSegments::OUTLINE, border, buf);
        self.round_corners(area, border, buf);
    }

    /// Rounds the corners of `area`'s outline, in the corner cells this stroke drew.
    fn round_corners(&self, area: Rect, border: CellBorder, buf: &mut Buffer) {
        if !self.borders.enabled || area.is_empty() {
            return;
        }
        let (left, right) = (area.x, area.right() - 1);
        let (top, bottom) = (area.y, area.bottom() - 1);
        let corners = [
            (left, top, BorderJoints::CORNER_TOP_LEFT),
            (right, top, BorderJoints::CORNER_TOP_RIGHT),
            (right, bottom, BorderJoints::CORNER_BOTTOM_RIGHT),
            (left, bottom, BorderJoints::CORNER_BOTTOM_LEFT),
        ];
        for (x, y, joint) in corners {
            // Fill the cut-off part of the corner with whatever is beside the shape.
            let beside = if joint
                .intersects(BorderJoints::CORNER_TOP_LEFT | BorderJoints::CORNER_BOTTOM_LEFT)
            {
                x.checked_sub(1)
            } else {
                x.checked_add(1)
            };
            let outside = beside
                .and_then(|beside| buf.cell(Position::new(beside, y)))
                .map_or(BG, |cell| cell.bg);
            if let Some(cell) = buf.cell_mut(Position::new(x, y))
                && cell.border.priority <= border.priority
            {
                cell.border.rounded |= joint;
                cell.border.outside = self.borders.round_outside.then_some(outside);
            }
        }
    }

    /// Draws the chosen outer sides of `area`.
    fn outline_sides(
        &self,
        area: Rect,
        sides: BorderSegments,
        border: CellBorder,
        buf: &mut Buffer,
    ) {
        if area.is_empty() {
            return;
        }
        let top = Rect { height: 1, ..area };
        let bottom = Rect {
            y: area.bottom() - 1,
            height: 1,
            ..area
        };
        let left = Rect { width: 1, ..area };
        let right = Rect {
            x: area.right() - 1,
            width: 1,
            ..area
        };
        for (edge_area, side) in [
            (top, BorderSegments::TOP),
            (bottom, BorderSegments::BOTTOM),
            (left, BorderSegments::LEFT),
            (right, BorderSegments::RIGHT),
        ] {
            if sides.contains(side) {
                self.edge(edge_area, side, border, buf);
            }
        }
    }

    /// Adds segments to every cell in `area`, drawn with the given stroke.
    ///
    /// A cell has one stroke for all its segments, so a cell that already has a higher priority
    /// stroke is skipped: a decoration that reaches a pane's outline stops there rather than
    /// taking on the outline's colour and width.
    fn edge(&self, area: Rect, segments: BorderSegments, border: CellBorder, buf: &mut Buffer) {
        if !self.borders.enabled {
            return;
        }
        for position in area.positions() {
            if let Some(cell) = buf.cell_mut(position) {
                let existing = cell.border;
                if !existing.is_empty() && existing.priority > border.priority {
                    continue;
                }
                cell.border = CellBorder {
                    segments: existing.segments | segments,
                    // Keep a rounded corner drawn earlier, and the colour outside it.
                    rounded: existing.rounded | border.rounded,
                    outside: border.outside.or(existing.outside),
                    ..border
                };
            }
        }
    }

    /// Turns segments on or off in every cell in `area`, keeping each cell's stroke.
    fn set_edges(&self, area: Rect, segments: BorderSegments, on: bool, buf: &mut Buffer) {
        if !self.borders.enabled {
            return;
        }
        for position in area.positions() {
            if let Some(cell) = buf.cell_mut(position) {
                cell.border.segments.set(segments, on);
            }
        }
    }
}

/// A stroke drawn inside the cell. Inside strokes never overlap a neighbour's lines, so a pane
/// outline and a line in the cell beside it both stay visible.
const fn stroke(color: Color, width: u8, priority: u8) -> CellBorder {
    CellBorder {
        width,
        priority,
        centered: false,
        ..CellBorder::new(BorderSegments::empty(), color)
    }
}

/// Blends `amount` of `color` into `base`.
fn mix(color: Color, base: Color, amount: f32) -> Color {
    let (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) = (color, base) else {
        return color;
    };
    let channel = |a: u8, b: u8| (f32::from(a) * amount + f32::from(b) * (1.0 - amount)) as u8;
    Color::Rgb(channel(r1, r2), channel(g1, g2), channel(b1, b2))
}

/// Lightens every other cell in a checkerboard.
fn render_grid(area: Rect, buf: &mut Buffer) {
    for position in area.positions() {
        if (position.x + position.y) % 2 == 1 {
            continue;
        }
        if let Some(cell) = buf.cell_mut(position) {
            let base = if matches!(cell.bg, Color::Rgb(..)) {
                cell.bg
            } else {
                BG
            };
            cell.bg = mix(Color::Rgb(255, 255, 255), base, 0.07);
        }
    }
}

/// Darkens the cells just right of and below `area`, like a drop shadow.
fn render_shadow(area: Rect, buf: &mut Buffer) {
    let right = Rect::new(area.right(), area.y + 1, 1, area.height);
    let bottom = Rect::new(area.x + 1, area.bottom(), area.width, 1);
    for shadow in [right, bottom] {
        buf.set_style(shadow.intersection(buf.area), Style::new().bg(SHADOW));
    }
}

/// A label that follows the mouse while dragging.
fn render_ghost(drag: &Drag, screen: Rect, buf: &mut Buffer) {
    let kind = Kind::of(drag.pane);
    let label = format!(" {} ", kind.tabs()[0]);
    let ghost =
        Rect::new(drag.mouse.x + 1, drag.mouse.y, label.len() as u16, 1).intersection(screen);
    Line::from(label)
        .fg(BG)
        .bg(kind.accent())
        .bold()
        .render(ghost, buf);
}

/// The gap between panes: two columns across, one row down, which look about the same size
/// because cells are roughly twice as tall as they are wide.
const fn gap(direction: Direction) -> u16 {
    match direction {
        Direction::Horizontal => 2,
        Direction::Vertical => 1,
    }
}

impl Node {
    fn layout(&self, area: Rect, out: &mut Vec<(usize, Rect)>) {
        match self {
            Self::Pane(id) => out.push((*id, area)),
            Self::Split(direction, children) => {
                let areas = Layout::new(*direction, children.iter().map(|_| Constraint::Fill(1)))
                    .spacing(gap(*direction))
                    .split(area);
                for (child, area) in children.iter().zip(areas.iter()) {
                    child.layout(*area, out);
                }
            }
        }
    }

    fn for_each_pane(&mut self, f: &mut impl FnMut(&mut usize)) {
        match self {
            Self::Pane(id) => f(id),
            Self::Split(_, children) => children.iter_mut().for_each(|c| c.for_each_pane(f)),
        }
    }

    fn first_pane(&self) -> usize {
        match self {
            Self::Pane(id) => *id,
            Self::Split(_, children) => children[0].first_pane(),
        }
    }

    /// Removes a pane, leaving any split it empties to be cleaned up by `normalize`.
    fn remove(&mut self, pane: usize) {
        if let Self::Split(_, children) = self {
            children.retain(|child| !matches!(child, Self::Pane(id) if *id == pane));
            for child in children.iter_mut() {
                child.remove(pane);
            }
        }
    }

    /// Replaces the target pane with a split holding the target and the new pane.
    fn insert_beside(&mut self, target: usize, pane: usize, drop: Drop) {
        match self {
            Self::Pane(id) if *id == target => {
                let (direction, first) = match drop {
                    Drop::Left => (Direction::Horizontal, true),
                    Drop::Right => (Direction::Horizontal, false),
                    Drop::Top => (Direction::Vertical, true),
                    Drop::Bottom | Drop::Swap => (Direction::Vertical, false),
                };
                let children = if first {
                    vec![Self::Pane(pane), Self::Pane(target)]
                } else {
                    vec![Self::Pane(target), Self::Pane(pane)]
                };
                *self = Self::Split(direction, children);
            }
            Self::Pane(_) => {}
            Self::Split(_, children) => children
                .iter_mut()
                .for_each(|child| child.insert_beside(target, pane, drop)),
        }
    }

    /// Drops empty splits, unwraps single-child splits, and flattens nested splits that run in
    /// the same direction so their children share space evenly.
    fn normalize(&mut self) {
        let Self::Split(direction, children) = self else {
            return;
        };
        children.iter_mut().for_each(Self::normalize);
        children.retain(|child| !matches!(child, Self::Split(_, c) if c.is_empty()));
        let direction = *direction;
        *children = children
            .drain(..)
            .flat_map(|child| match child {
                Self::Split(d, grandchildren) if d == direction => grandchildren,
                child => vec![child],
            })
            .collect();
        if children.len() == 1 {
            *self = children.pop().unwrap();
        }
    }
}
