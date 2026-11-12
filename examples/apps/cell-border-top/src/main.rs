/// A btop-style system monitor drawn with the experimental cell border protocol.
///
/// Every line is drawn on the edges of cells, so none of it takes a column or row of its own:
/// box outlines sit on the boundary of each box's background, labels in the header and footer
/// rows are split by rules on cell edges, and nested boxes, column rules, graph grids, gauge
/// the table's row groups, scrollbar, selection and hover rings all sit between cells.
/// Terminals without the protocol get box-drawing borders instead.
///
/// The graphs start with made-up history so the screen is full from the first frame.
///
/// This example runs with the Ratatui library code in the branch that you are currently
/// reading. See the [`latest`] branch for the code which works with the most recent Ratatui
/// release.
///
/// [`latest`]: https://github.com/ratatui/ratatui/tree/latest
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{Read, Write};
use std::process::Command;
use std::sync::mpsc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use color_eyre::Result;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseButton, MouseEvent,
    MouseEventKind,
};
use crossterm::execute;
use ratatui::buffer::{BorderJoints, BorderSegments, Buffer, CellBorder, LineStyle};
use ratatui::layout::{Alignment, Constraint, Layout, Margin, Position, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Widget, Wrap};
use ratatui::{DefaultTerminal, Frame};
use sysinfo::{
    Components, Disks, Networks, Pid, ProcessRefreshKind, ProcessesToUpdate, System, Users,
};

const BG: Color = Color::Rgb(0, 0, 0);
const PANEL: Color = Color::Rgb(12, 17, 32);
const RAISED: Color = Color::Rgb(20, 27, 48);
const TRACK: Color = Color::Rgb(34, 40, 54);
const TEXT: Color = Color::Rgb(232, 236, 242);
const MUTED: Color = Color::Rgb(122, 134, 154);
const LINE: Color = Color::Rgb(58, 66, 86);
const FAINT: Color = Color::Rgb(36, 42, 56);
const TEAL: Color = Color::Rgb(0, 224, 184);
const RED: Color = Color::Rgb(255, 72, 120);
const GOLD: Color = Color::Rgb(255, 190, 30);
const BLUE: Color = Color::Rgb(64, 140, 255);
const VIOLET: Color = Color::Rgb(170, 110, 255);

/// Stroke widths in 1/256 of the cell width.
const HAIR: u8 = 12;
const THIN: u8 = 20;
const THICK: u8 = 40;
/// Wide enough for a double line to read as two lines.
const WIDE: u8 = 56;

const HISTORY: usize = 1024;
/// How much CPU history to keep for each process.
const PROCESS_HISTORY: usize = 128;
/// How many made-up samples the graphs start with.
const SEED_SAMPLES: usize = 600;
const TICKS: [u64; 6] = [250, 500, 1000, 2000, 3000, 5000];
/// How many refreshes between battery readings, which come from a subprocess.
const BATTERY_EVERY: usize = 15;

/// The menu's items and the keys they press.
const MENU: [(&str, char); 4] = [
    ("Help", 'h'),
    ("Cell borders", 'b'),
    ("Cell grid", 'g'),
    ("Quit", 'q'),
];

const HELP: [(&str, &str); 16] = [
    ("1 2 3 4", "Show or hide the cpu, mem, net and proc boxes"),
    ("esc", "Menu, or close the open popup or details"),
    ("h ?", "This help"),
    ("q", "Quit"),
    ("↑ ↓ j k", "Select a process"),
    ("pgup pgdn", "Select a page at a time"),
    ("enter", "Show details for the selected process"),
    ("← →", "Change the sort column"),
    ("c m", "Sort by cpu or memory"),
    ("r", "Reverse the sort"),
    ("t", "Show processes as a tree"),
    ("f /", "Filter processes"),
    ("p", "Pause the process list"),
    ("i", "Next network interface"),
    ("- +", "Refresh slower or faster"),
    ("b g o", "Cell borders, cell grid, corner fill"),
];

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sort {
    Pid,
    Name,
    Cpu,
    Memory,
}

impl Sort {
    const ALL: [Self; 4] = [Self::Pid, Self::Name, Self::Cpu, Self::Memory];

    const fn label(self) -> &'static str {
        match self {
            Self::Pid => "pid",
            Self::Name => "program",
            Self::Cpu => "cpu",
            Self::Memory => "memory",
        }
    }

    /// The table column this sort orders by.
    const fn column(self) -> usize {
        match self {
            Self::Pid => 0,
            Self::Name => 1,
            Self::Memory => 5,
            Self::Cpu => 6,
        }
    }
}

/// A process as shown in the table.
struct Proc {
    pid: u32,
    parent: Option<u32>,
    name: String,
    command: String,
    user: String,
    cpu: f32,
    memory: u64,
    /// Thread count, where the platform reports it.
    threads: Option<u32>,
    /// Tree prefix, when the table is shown as a tree.
    indent: String,
    /// Recent CPU use, oldest first, as a share of one core.
    history: VecDeque<f32>,
}

/// Apple Silicon readings from macmon: the GPU, power, temperatures, cluster frequencies and
/// fans.
#[derive(Debug, Clone, Default)]
struct Hardware {
    /// The share of time the GPU was busy.
    gpu: f32,
    gpu_mhz: u32,
    gpu_temperature: Option<f32>,
    cpu_temperature: Option<f32>,
    /// Watts.
    cpu_power: f32,
    gpu_power: f32,
    ane_power: f32,
    all_power: f32,
    /// Each CPU cluster's label (`E`, `P` or `S`) and frequency in MHz.
    clusters: Vec<(String, u32)>,
    /// Fan speeds in RPM.
    fans: Vec<u32>,
    /// CPU core cluster temperature sensors, in the order the system lists them.
    core_temperatures: Vec<f32>,
}

/// Samples Apple Silicon hardware once a second on a worker thread. The channel stays empty
/// on other machines, or if sampling fails.
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn sample_hardware() -> mpsc::Receiver<Hardware> {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let Ok(mut sampler) = macmon::Sampler::new() else {
            return;
        };
        let sensors = macmon::sources::IOHIDSensors::new().ok();
        while let Ok(metrics) = sampler.get_metrics(1000) {
            let hardware = Hardware {
                gpu: metrics.gpu_active_ratio,
                gpu_mhz: metrics.gpu_freq_mhz,
                gpu_temperature: metrics.temp.gpu_temp_avg,
                cpu_temperature: metrics.temp.cpu_temp_avg,
                cpu_power: metrics.cpu_power,
                gpu_power: metrics.gpu_power,
                ane_power: metrics.ane_power,
                all_power: metrics.all_power,
                clusters: metrics
                    .cpu_tiers
                    .iter()
                    .map(|tier| (tier.label.clone(), tier.freq_mhz))
                    .collect(),
                fans: metrics.fans.iter().map(|fan| fan.rpm).collect(),
                core_temperatures: sensors
                    .as_ref()
                    .map(|sensors| {
                        sensors
                            .get_metrics()
                            .into_iter()
                            .filter(|(name, _)| {
                                // The sensors macmon averages for its CPU temperature.
                                name.starts_with("pACC MTR Temp Sensor")
                                    || name.starts_with("eACC MTR Temp Sensor")
                            })
                            .map(|(_, celsius)| celsius)
                            .filter(|celsius| *celsius > 0.0)
                            .collect()
                    })
                    .unwrap_or_default(),
            };
            if sender.send(hardware).is_err() {
                return;
            }
        }
    });
    receiver
}

#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
fn sample_hardware() -> mpsc::Receiver<Hardware> {
    mpsc::channel().1
}

/// A mounted volume, from `df`.
struct Volume {
    /// `root` for `/`, otherwise the last part of the mount point.
    name: String,
    mount: String,
    /// Bytes.
    total: u64,
    available: u64,
}

/// Mounted volumes backed by a device, in the system's order. sysinfo skips macOS's system
/// volumes, which btop shows.
fn read_volumes() -> Vec<Volume> {
    let Ok(output) = Command::new("df").arg("-kP").output() else {
        return Vec::new();
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .skip(1)
        .filter(|line| line.starts_with("/dev/"))
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let _device = fields.next()?;
            let total: u64 = fields.next()?.parse().ok()?;
            let _used = fields.next()?;
            let available: u64 = fields.next()?.parse().ok()?;
            let _capacity = fields.next()?;
            let mount = fields.collect::<Vec<_>>().join(" ");
            let name = if mount == "/" {
                "root".to_string()
            } else {
                mount.rsplit('/').next().unwrap_or_default().to_string()
            };
            Some(Volume {
                name,
                mount,
                total: total * 1024,
                available: available * 1024,
            })
        })
        .collect()
}

/// A battery reading from `pmset`.
struct Battery {
    percent: u8,
    charging: bool,
    remaining: String,
}

/// A small xorshift generator for the made-up startup history.
struct Noise(u64);

impl Noise {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % 10_000) as f32 / 10_000.0
    }

    /// A wandering series that ends at `end`, oldest first, with occasional spikes.
    fn walk(&mut self, end: f32, wander: f32, spikes: f32) -> Vec<f32> {
        let mut value = end;
        let mut values: Vec<f32> = (0..SEED_SAMPLES)
            .map(|_| {
                value = (value + (self.next() - 0.5) * wander).clamp(0.0, 1.0);
                let spike = if self.next() < spikes {
                    self.next() * 0.6
                } else {
                    0.0
                };
                (value + spike).min(1.0)
            })
            .collect();
        values.reverse();
        values
    }
}

/// System readings, plus the history the graphs need.
struct Stats {
    system: System,
    networks: Networks,
    disks: Disks,
    /// Mounted volumes in the system's order, as btop lists them.
    volumes: Vec<Volume>,
    components: Components,
    users: Users,
    battery: Option<Battery>,
    hardware_samples: mpsc::Receiver<Hardware>,
    /// The latest Apple Silicon reading, once one has arrived.
    hardware: Option<Hardware>,
    gpu: VecDeque<f32>,
    cpu: VecDeque<f32>,
    cores: Vec<VecDeque<f32>>,
    memory: [VecDeque<f32>; 5],
    /// Bytes of file-backed memory the system can drop, which btop shows as cached.
    cached: Option<u64>,
    /// Bytes per second, for each network interface.
    received: HashMap<String, VecDeque<f64>>,
    transmitted: HashMap<String, VecDeque<f64>>,
    /// Bytes per second read and written, for each disk.
    disk_io: HashMap<String, (f64, f64)>,
    /// Recent bytes per second read and written together, for each disk.
    disk_history: HashMap<String, VecDeque<f64>>,
    /// Recent CPU use for each process, as a share of one core.
    process_cpu: HashMap<u32, VecDeque<f32>>,
    last_refresh: Instant,
    refreshes: usize,
}

impl Stats {
    fn new() -> Self {
        let mut system = System::new_all();
        // CPU usage is measured between two refreshes.
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        system.refresh_cpu_usage();
        let cores = vec![VecDeque::new(); system.cpus().len()];
        let mut stats = Self {
            system,
            networks: Networks::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list(),
            volumes: Vec::new(),
            components: Components::new_with_refreshed_list(),
            users: Users::new_with_refreshed_list(),
            battery: None,
            hardware_samples: sample_hardware(),
            hardware: None,
            gpu: VecDeque::new(),
            cpu: VecDeque::new(),
            cores,
            memory: Default::default(),
            cached: None,
            received: HashMap::new(),
            transmitted: HashMap::new(),
            disk_io: HashMap::new(),
            disk_history: HashMap::new(),
            process_cpu: HashMap::new(),
            last_refresh: Instant::now(),
            refreshes: 0,
        };
        stats.refresh(false);
        stats.seed_history();
        stats
    }

    /// Fills the graphs with made-up history that leads into the first real reading.
    fn seed_history(&mut self) {
        let mut noise = Noise(0x2545_f491_4f6c_dd1d);
        let prepend = |history: &mut VecDeque<f32>, values: Vec<f32>| {
            for value in values.into_iter().rev() {
                history.push_front(value);
            }
        };
        let cpu = self.cpu.back().copied().unwrap_or(0.2);
        prepend(&mut self.cpu, noise.walk(cpu, 0.08, 0.03));
        prepend(&mut self.gpu, noise.walk(0.08, 0.06, 0.04));
        for core in &mut self.cores {
            let end = core.back().copied().unwrap_or(0.2);
            prepend(core, noise.walk(end, 0.2, 0.06));
        }
        for history in &mut self.memory {
            let end = history.back().copied().unwrap_or(0.5);
            prepend(history, noise.walk(end, 0.01, 0.0));
        }
        for history in self.process_cpu.values_mut() {
            let end = history.back().copied().unwrap_or(0.0);
            let values = noise.walk(end, 0.06, 0.04);
            prepend(
                history,
                values[SEED_SAMPLES - PROCESS_HISTORY + 1..].to_vec(),
            );
        }
        for rates in self
            .received
            .values_mut()
            .chain(self.transmitted.values_mut())
            .chain(self.disk_history.values_mut())
        {
            let end = rates.back().copied().unwrap_or(0.0);
            // Wander around the current rate, so real readings fill the graph as soon as they
            // arrive rather than looking flat beside made-up spikes.
            let scale = end.max(2.0 * 1024.0);
            for value in noise.walk(0.4, 0.15, 0.05).into_iter().rev() {
                rates.push_front(f64::from(value) * scale * 2.0);
            }
        }
    }

    /// Takes new readings. A paused refresh leaves the process list alone.
    fn refresh(&mut self, paused: bool) {
        let elapsed = self.last_refresh.elapsed().as_secs_f64().max(0.001);
        self.last_refresh = Instant::now();
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
        if !paused {
            self.system.refresh_processes_specifics(
                ProcessesToUpdate::All,
                true,
                ProcessRefreshKind::everything(),
            );
        }
        self.networks.refresh(true);
        self.disks.refresh(true);
        self.volumes = read_volumes();
        self.components.refresh(false);
        if self.refreshes.is_multiple_of(BATTERY_EVERY) {
            self.battery = read_battery();
        }
        self.refreshes += 1;

        if !paused {
            let processes = self.system.processes();
            self.process_cpu
                .retain(|pid, _| processes.contains_key(&Pid::from_u32(*pid)));
            for (pid, process) in processes {
                let history = self.process_cpu.entry(pid.as_u32()).or_default();
                if history.len() == PROCESS_HISTORY {
                    history.pop_front();
                }
                history.push_back(process.cpu_usage() / 100.0);
            }
        }
        if let Some(hardware) = self.hardware_samples.try_iter().last() {
            self.hardware = Some(hardware);
        }
        if let Some(hardware) = &self.hardware {
            push(&mut self.gpu, hardware.gpu);
        }
        push(&mut self.cpu, self.system.global_cpu_usage() / 100.0);
        for (history, core) in self.cores.iter_mut().zip(self.system.cpus()) {
            push(history, core.cpu_usage() / 100.0);
        }
        self.cached = cached_memory();
        let total = self.system.total_memory().max(1) as f32;
        let swap = self.system.total_swap().max(1) as f32;
        let memory = [
            self.system.used_memory() as f32 / total,
            self.system.available_memory() as f32 / total,
            self.cached.unwrap_or(0) as f32 / total,
            self.system.free_memory() as f32 / total,
            self.system.used_swap() as f32 / swap,
        ];
        for (history, value) in self.memory.iter_mut().zip(memory) {
            push(history, value);
        }
        for (name, data) in &self.networks {
            let received = self.received.entry(name.clone()).or_default();
            push(received, data.received() as f64 / elapsed);
            let transmitted = self.transmitted.entry(name.clone()).or_default();
            push(transmitted, data.transmitted() as f64 / elapsed);
        }
        for disk in &self.disks {
            let usage = disk.usage();
            let name = disk.mount_point().to_string_lossy().into_owned();
            let rates = (
                usage.read_bytes as f64 / elapsed,
                usage.written_bytes as f64 / elapsed,
            );
            push(
                self.disk_history.entry(name.clone()).or_default(),
                rates.0 + rates.1,
            );
            self.disk_io.insert(name, rates);
        }
    }

    /// The average temperature of sensors that look like they belong to the CPU, or the hottest
    /// sensor if none do.
    fn cpu_temperature(&self) -> Option<f32> {
        if let Some(celsius) = self
            .hardware
            .as_ref()
            .and_then(|hardware| hardware.cpu_temperature)
        {
            return Some(celsius);
        }
        let readings: Vec<(String, f32)> = self
            .components
            .iter()
            .filter_map(|component| {
                let label = component.label().to_lowercase();
                Some((label, component.temperature()?))
            })
            .filter(|(_, temperature)| *temperature > 0.0)
            .collect();
        let names = ["cpu", "tdie", "tctl", "core", "package"];
        let cpu: Vec<f32> = readings
            .iter()
            .filter(|(label, _)| names.iter().any(|name| label.contains(name)))
            .map(|(_, temperature)| *temperature)
            .collect();
        if cpu.is_empty() {
            readings
                .iter()
                .map(|(_, temperature)| *temperature)
                .reduce(f32::max)
        } else {
            Some(cpu.iter().sum::<f32>() / cpu.len() as f32)
        }
    }

    /// Network interfaces, busiest first.
    fn interfaces(&self) -> Vec<String> {
        let mut names: Vec<String> = self.networks.keys().cloned().collect();
        names.sort_by_key(|name| {
            std::cmp::Reverse(
                self.networks
                    .get(name)
                    .map_or(0, |data| data.total_received() + data.total_transmitted()),
            )
        });
        names
    }

    fn processes(&self, sort: Sort, reverse: bool, filter: &str, tree: bool) -> Vec<Proc> {
        let filter = filter.to_lowercase();
        let mut processes: Vec<Proc> = self
            .system
            .processes()
            .values()
            .map(|process| Proc {
                pid: process.pid().as_u32(),
                parent: process.parent().map(Pid::as_u32),
                name: process.name().to_string_lossy().into_owned(),
                command: process
                    .cmd()
                    .iter()
                    .map(|part| part.to_string_lossy())
                    .collect::<Vec<_>>()
                    .join(" "),
                user: process
                    .user_id()
                    .and_then(|uid| self.users.get_user_by_id(uid))
                    .map(|user| user.name().to_string())
                    .unwrap_or_default(),
                cpu: process.cpu_usage(),
                memory: process.memory(),
                threads: thread_count(process.pid().as_u32()),
                indent: String::new(),
                history: self
                    .process_cpu
                    .get(&process.pid().as_u32())
                    .cloned()
                    .unwrap_or_default(),
            })
            .filter(|process| {
                filter.is_empty()
                    || process.name.to_lowercase().contains(&filter)
                    || process.command.to_lowercase().contains(&filter)
                    || process.pid.to_string().contains(&filter)
            })
            .collect();
        match sort {
            Sort::Pid => processes.sort_by_key(|process| process.pid),
            Sort::Name => processes.sort_by_key(|process| process.name.to_lowercase()),
            Sort::Cpu => processes.sort_by(|a, b| b.cpu.total_cmp(&a.cpu)),
            Sort::Memory => processes.sort_by_key(|process| std::cmp::Reverse(process.memory)),
        }
        if reverse {
            processes.reverse();
        }
        if tree {
            processes = as_tree(processes);
        }
        processes
    }
}

/// Orders processes depth first under their parents, keeping the sort order among siblings,
/// and gives each one a tree prefix.
fn as_tree(processes: Vec<Proc>) -> Vec<Proc> {
    let pids: HashSet<u32> = processes.iter().map(|process| process.pid).collect();
    let mut children: HashMap<Option<u32>, Vec<Proc>> = HashMap::new();
    for process in processes {
        let parent = process.parent.filter(|parent| pids.contains(parent));
        children.entry(parent).or_default().push(process);
    }
    let mut ordered = Vec::new();
    let mut stack: Vec<(Proc, String, bool)> = Vec::new();
    let roots = children.remove(&None).unwrap_or_default();
    let root_count = roots.len();
    for (index, root) in roots.into_iter().enumerate().rev() {
        stack.push((root, String::new(), index + 1 == root_count));
    }
    while let Some((mut process, prefix, last)) = stack.pop() {
        let has_parent = process.parent.is_some_and(|parent| pids.contains(&parent));
        let (own, inherited) = match (has_parent, last) {
            (false, _) => (String::new(), String::new()),
            (true, true) => (format!("{prefix}└ "), format!("{prefix}  ")),
            (true, false) => (format!("{prefix}├ "), format!("{prefix}│ ")),
        };
        let kids = children.remove(&Some(process.pid)).unwrap_or_default();
        let count = kids.len();
        for (index, kid) in kids.into_iter().enumerate().rev() {
            stack.push((kid, inherited.clone(), index + 1 == count));
        }
        process.indent = own;
        ordered.push(process);
    }
    ordered
}

fn push<T>(history: &mut VecDeque<T>, value: T) {
    if history.len() == HISTORY {
        history.pop_front();
    }
    history.push_back(value);
}

/// Reads the battery level from macOS's `pmset`. Returns `None` elsewhere or without a battery.
fn read_battery() -> Option<Battery> {
    let output = Command::new("pmset").args(["-g", "batt"]).output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let line = text.lines().find(|line| line.contains("InternalBattery"))?;
    let details = line.split('\t').nth(1)?;
    let mut fields = details.split(';').map(str::trim);
    let percent = fields.next()?.trim_end_matches('%').parse().ok()?;
    let state = fields.next().unwrap_or_default();
    let remaining = fields
        .next()
        .and_then(|field| field.split_whitespace().next())
        .filter(|time| time.contains(':'))
        .unwrap_or_default()
        .to_string();
    Some(Battery {
        percent,
        charging: state == "charging" || state == "charged",
        remaining,
    })
}

/// The local time zone's offset from UTC in seconds, from `date`. Zero if that fails.
fn utc_offset() -> i64 {
    let output = Command::new("date").arg("+%z").output();
    let text = output
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_default();
    let sign = if text.starts_with('-') { -1 } else { 1 };
    let digits = text.trim_start_matches(['+', '-']);
    let hours: i64 = digits.get(..2).and_then(|h| h.parse().ok()).unwrap_or(0);
    let minutes: i64 = digits.get(2..4).and_then(|m| m.parse().ok()).unwrap_or(0);
    sign * (hours * 3_600 + minutes * 60)
}

/// Where a label sits in a box's header or footer row.
#[derive(Debug, Clone, Copy)]
enum Place {
    /// After the previous left label.
    Left,
    /// Before the previous right label.
    Right,
    /// At a column offset from the box's left edge.
    At(u16),
}

/// A label in a box's header or footer row.
struct Label<'a> {
    place: Place,
    spans: Vec<Span<'a>>,
    /// The key a click on the label presses.
    key: Option<char>,
}

impl<'a> Label<'a> {
    const fn new(place: Place, spans: Vec<Span<'a>>) -> Self {
        Self {
            place,
            spans,
            key: None,
        }
    }

    const fn on_click(mut self, key: char) -> Self {
        self.key = Some(key);
        self
    }

    /// A label that does nothing when clicked.
    const fn inert(mut self) -> Self {
        self.key = None;
        self
    }

    /// A btop-style button: the hotkey letter picked out in the accent colour. Buttons that are
    /// off are dimmed.
    fn key(place: Place, text: &'a str, key: usize, accent: Color, on: bool) -> Self {
        let (before, rest) = text.split_at(key);
        let (hotkey, after) = rest.split_at(1);
        let rest_color = if on { TEXT } else { MUTED };
        let spans = vec![
            Span::from(before).fg(rest_color),
            Span::from(hotkey).fg(accent).bold(),
            Span::from(after).fg(rest_color),
        ];
        let click = hotkey.chars().next().unwrap_or(' ');
        Self::new(place, spans).on_click(click)
    }

    /// A box title: the box's number in a softer shade of the accent colour, and its name in
    /// bold. Clicking it hides
    /// the box, as its number key does.
    fn title(number: usize, name: &'a str, accent: Color) -> Self {
        Self::new(
            Place::Left,
            vec![
                Span::from(format!("{number} ")).fg(mix(accent, PANEL, 0.7)),
                Span::from(name).fg(accent).bold(),
            ],
        )
        .on_click(char::from(b'0' + number as u8))
    }

    fn width(&self) -> u16 {
        self.spans.iter().map(|span| span.width() as u16).sum()
    }
}

/// How the process table is filtered, ordered and refreshed.
struct ProcessView {
    sort: Sort,
    reverse: bool,
    tree: bool,
    paused: bool,
    filter: Filter,
}

/// The process filter text, and whether it is being typed.
struct Filter {
    text: String,
    editing: bool,
}

/// Which side of its box a docked pane sits against.
#[derive(Debug, Clone, Copy)]
enum Dock {
    /// Against the right edge, sized to its content and centred, like btop's side boxes.
    Right,
    /// Across the top, under the box's header.
    Top,
}

/// A popup over the boxes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Overlay {
    None,
    /// The menu, with its selected item.
    Menu(usize),
    Help,
}

/// Whether to draw with the cell border protocol, and which of its optional features to use.
struct Borders {
    enabled: bool,
    /// Whether rounded corners fill the part outside their arc. Needs the terminal's `outside`
    /// feature.
    round_outside: bool,
}

struct App {
    stats: Stats,
    view: ProcessView,
    overlay: Overlay,
    /// The process whose details are shown above the table.
    detailed: Option<u32>,
    /// The selected process from the last render.
    selected_pid: Option<u32>,
    /// Which of the cpu, mem, net and proc boxes are hidden.
    hidden: [bool; 4],
    /// Clickable labels from the last render, and the keys they press.
    hits: RefCell<Vec<(Rect, char)>>,
    interface: usize,
    selected: usize,
    tick: usize,
    utc_offset: i64,
    borders: Borders,
    show_grid: bool,
    mouse: Position,
    /// The process table rows from the last render, used for hit testing.
    rows: Rect,
    /// The first process shown in the table.
    scroll: usize,
    should_exit: bool,
}

impl App {
    fn new(cell_borders: bool) -> Self {
        Self {
            stats: Stats::new(),
            view: ProcessView {
                sort: Sort::Cpu,
                reverse: false,
                tree: false,
                paused: false,
                filter: Filter {
                    text: String::new(),
                    editing: false,
                },
            },
            overlay: Overlay::None,
            detailed: None,
            selected_pid: None,
            hidden: [false; 4],
            hits: RefCell::new(Vec::new()),
            interface: 0,
            selected: 0,
            tick: 2,
            utc_offset: utc_offset(),
            borders: Borders {
                enabled: cell_borders,
                round_outside: false,
            },
            show_grid: false,
            mouse: Position::default(),
            rows: Rect::default(),
            scroll: 0,
            should_exit: false,
        }
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        execute!(std::io::stdout(), EnableMouseCapture)?;
        while !self.should_exit {
            terminal.draw(|frame| self.render(frame))?;
            let tick = Duration::from_millis(TICKS[self.tick]);
            let elapsed = self.stats.last_refresh.elapsed();
            if event::poll(tick.saturating_sub(elapsed))? {
                self.handle_event(&event::read()?);
            }
            if self.stats.last_refresh.elapsed() >= tick {
                self.stats.refresh(self.view.paused);
            }
        }
        execute!(std::io::stdout(), DisableMouseCapture)?;
        Ok(())
    }

    fn handle_event(&mut self, event: &Event) {
        match event {
            Event::Key(key) if key.is_press() && self.view.filter.editing => match key.code {
                KeyCode::Enter => self.view.filter.editing = false,
                KeyCode::Esc => {
                    self.view.filter.editing = false;
                    self.view.filter.text.clear();
                }
                KeyCode::Backspace => {
                    self.view.filter.text.pop();
                }
                KeyCode::Char(c) => {
                    self.view.filter.text.push(c);
                    self.selected = 0;
                }
                _ => {}
            },
            Event::Key(key) if key.is_press() && self.overlay != Overlay::None => {
                self.on_overlay_key(key.code);
            }
            Event::Key(key) if key.is_press() => self.on_key(key.code),
            Event::Mouse(mouse) => self.on_mouse(*mouse),
            _ => {}
        }
    }

    fn on_overlay_key(&mut self, code: KeyCode) {
        let Overlay::Menu(selected) = self.overlay else {
            self.overlay = Overlay::None;
            return;
        };
        let item = match code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.overlay = Overlay::Menu((selected + MENU.len() - 1) % MENU.len());
                return;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.overlay = Overlay::Menu((selected + 1) % MENU.len());
                return;
            }
            KeyCode::Enter => MENU[selected].1,
            KeyCode::Char(c) if MENU.iter().any(|(_, key)| *key == c) => c,
            _ => {
                self.overlay = Overlay::None;
                return;
            }
        };
        self.overlay = Overlay::None;
        self.on_key(KeyCode::Char(item));
    }

    fn on_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc if self.detailed.is_some() => self.detailed = None,
            KeyCode::Esc if !self.view.filter.text.is_empty() => self.view.filter.text.clear(),
            KeyCode::Esc => self.overlay = Overlay::Menu(0),
            KeyCode::Char('q') => self.should_exit = true,
            KeyCode::Char('h' | '?') => self.overlay = Overlay::Help,
            KeyCode::Char(c @ '1'..='4') => {
                let index = usize::from(c as u8 - b'1');
                self.hidden[index] = !self.hidden[index];
            }
            KeyCode::Enter => {
                self.detailed = match self.detailed {
                    Some(pid) if Some(pid) == self.selected_pid => None,
                    _ => self.selected_pid,
                };
            }
            KeyCode::Char('f' | '/') => self.view.filter.editing = true,
            KeyCode::Char('t') => self.view.tree = !self.view.tree,
            KeyCode::Char('r') => self.view.reverse = !self.view.reverse,
            KeyCode::Char('p') => self.view.paused = !self.view.paused,
            KeyCode::Char('b') => self.borders.enabled = !self.borders.enabled,
            KeyCode::Char('o') => self.borders.round_outside = !self.borders.round_outside,
            KeyCode::Char('g') => self.show_grid = !self.show_grid,
            KeyCode::Char('i') => self.interface += 1,
            KeyCode::Char('c') => self.view.sort = Sort::Cpu,
            KeyCode::Char('m') => self.view.sort = Sort::Memory,
            KeyCode::Left | KeyCode::Right => {
                let index = Sort::ALL
                    .iter()
                    .position(|sort| *sort == self.view.sort)
                    .unwrap_or(0);
                let step = if code == KeyCode::Left {
                    Sort::ALL.len() - 1
                } else {
                    1
                };
                self.view.sort = Sort::ALL[(index + step) % Sort::ALL.len()];
            }
            KeyCode::Char('+' | '=') => self.tick = (self.tick + 1).min(TICKS.len() - 1),
            KeyCode::Char('-') => self.tick = self.tick.saturating_sub(1),
            KeyCode::Up | KeyCode::Char('k') => self.selected = self.selected.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => self.selected += 1,
            KeyCode::PageUp => self.selected = self.selected.saturating_sub(20),
            KeyCode::PageDown => self.selected += 20,
            KeyCode::Home => self.selected = 0,
            KeyCode::End => self.selected = usize::MAX,
            _ => {}
        }
    }

    fn on_mouse(&mut self, mouse: MouseEvent) {
        let position = Position::new(mouse.column, mouse.row);
        self.mouse = position;
        match mouse.kind {
            MouseEventKind::ScrollUp => self.selected = self.selected.saturating_sub(1),
            MouseEventKind::ScrollDown => self.selected += 1,
            MouseEventKind::Down(MouseButton::Left) => {
                let hit = self
                    .hits
                    .borrow()
                    .iter()
                    .rev()
                    .find(|(area, _)| area.contains(position))
                    .map(|(_, key)| *key);
                if self.overlay != Overlay::None {
                    self.overlay = Overlay::None;
                    if let Some(key) = hit {
                        self.on_key(key_code(key));
                    }
                } else if let Some(key) = hit {
                    self.on_key(key_code(key));
                } else if self.rows.contains(position) {
                    self.selected = self.scroll + usize::from(position.y - self.rows.y);
                } else {
                    // Clicks elsewhere do nothing.
                }
            }
            _ => {}
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let buf = frame.buffer_mut();
        buf.set_style(area, Style::new().bg(BG).fg(TEXT));
        self.hits.borrow_mut().clear();
        let [show_cpu, show_memory, show_network, show_processes] =
            self.hidden.map(|hidden| !hidden);
        let show_left = show_memory || show_network;
        let show_bottom = show_left || show_processes;
        // Two columns across and one row down look about the same size.
        let [cpu, bottom] = split(
            &Layout::vertical([Constraint::Percentage(34), Constraint::Fill(1)]).spacing(1),
            area,
            show_cpu,
            show_bottom,
        );
        let [left, processes] = split(
            &Layout::horizontal([Constraint::Percentage(46), Constraint::Fill(1)]).spacing(2),
            bottom,
            show_left,
            show_processes,
        );
        let [memory, network] = split(
            &Layout::vertical([Constraint::Percentage(55), Constraint::Fill(1)]).spacing(1),
            left,
            show_memory,
            show_network,
        );

        if show_cpu {
            self.render_cpu(cpu, buf);
        }
        if show_memory {
            self.render_memory(memory, buf);
        }
        if show_network {
            self.render_network(network, buf);
        }
        if show_processes {
            self.render_processes(processes, buf);
        } else {
            self.rows = Rect::default();
        }
        if self.show_grid {
            render_grid(area, buf);
        }
        match self.overlay {
            Overlay::None => {}
            Overlay::Menu(selected) => self.render_menu(selected, area, buf),
            Overlay::Help => self.render_help(area, buf),
        }
    }

    /// The menu: a popup of pill buttons, the selected one with a thick ring.
    fn render_menu(&self, selected: usize, screen: Rect, buf: &mut Buffer) {
        let height = MENU.len() as u16 * 2 + 3;
        let area = screen.centered(Constraint::Length(34), Constraint::Length(height));
        let inner = self.popup(area, "menu", TEAL, buf);
        let rows = Layout::vertical([Constraint::Length(1); MENU.len()])
            .spacing(1)
            .split(inner.inner(Margin::new(3, 1)));
        for (index, ((label, key), row)) in MENU.iter().zip(rows.iter()).enumerate() {
            let on = index == selected;
            let (ring, bg) = if on {
                (stroke(TEAL, THICK, 5), mix(TEAL, RAISED, 0.25))
            } else {
                (stroke(LINE, THIN, 5), RAISED)
            };
            buf.set_style(*row, Style::new().bg(bg));
            self.outline_round(*row, ring, buf);
            let state = match key {
                'b' => {
                    if self.borders.enabled {
                        "  on"
                    } else {
                        "  off"
                    }
                }
                'g' => {
                    if self.show_grid {
                        "  on"
                    } else {
                        "  off"
                    }
                }
                _ => "",
            };
            Line::from(vec![
                Span::from(label.to_string()).bold(),
                Span::from(state).fg(MUTED),
            ])
            .centered()
            .render(*row, buf);
            self.hits.borrow_mut().push((*row, *key));
        }
    }

    /// The key bindings, in a table with a rule between its columns.
    fn render_help(&self, screen: Rect, buf: &mut Buffer) {
        let height = HELP.len() as u16 + 4;
        let area = screen.centered(Constraint::Length(64), Constraint::Length(height));
        let inner = self.popup(area, "help", GOLD, buf);
        let [keys, actions] =
            Layout::horizontal([Constraint::Length(12), Constraint::Fill(1)]).areas(inner);
        for (index, ((key, action), row)) in HELP.iter().zip(inner.rows()).enumerate() {
            Line::from(*key).fg(GOLD).bold().right_aligned().render(
                Rect {
                    y: row.y,
                    height: 1,
                    ..keys
                }
                .inner(Margin::new(1, 0)),
                buf,
            );
            Line::from(*action).render(
                Rect {
                    y: row.y,
                    height: 1,
                    ..actions
                }
                .inner(Margin::new(1, 0)),
                buf,
            );
            if index % 4 == 3 && index + 1 < HELP.len() {
                self.edge(row, BorderSegments::BOTTOM, stroke(LINE, HAIR, 3), buf);
            }
        }
        let rule = Rect {
            width: 1,
            ..actions
        };
        self.edge(
            rule,
            BorderSegments::LEFT,
            stroke(mix(GOLD, RAISED, 0.5), THIN, 3),
            buf,
        );
    }

    /// Draws a floating box with a shadow and returns the area inside it. The title row is
    /// separated from the body by a double line on the cell edge.
    fn popup(&self, area: Rect, title: &str, accent: Color, buf: &mut Buffer) -> Rect {
        let shadow = Rect {
            x: area.x + 2,
            y: area.y + 1,
            ..area
        }
        .intersection(buf.area);
        for position in shadow.positions() {
            if let Some(cell) = buf.cell_mut(position) {
                cell.bg = BG;
                cell.fg = mix(cell.fg, BG, 0.3);
            }
        }
        Clear.render(area, buf);
        buf.set_style(area, Style::new().bg(RAISED).fg(TEXT));
        if !self.borders.enabled {
            let block = Block::bordered()
                .border_style(Style::new().fg(accent))
                .title(
                    Line::from(format!(" {title} "))
                        .fg(accent)
                        .bold()
                        .centered(),
                );
            let inner = block.inner(area);
            block.render(area, buf);
            return inner;
        }
        self.outline_round(area, stroke(accent, THICK, 6), buf);
        let heading = Rect { height: 1, ..area };
        buf.set_style(heading, Style::new().bg(mix(accent, RAISED, 0.2)));
        Line::from(title.to_string())
            .fg(accent)
            .bold()
            .centered()
            .render(heading, buf);
        let separator = CellBorder {
            style: LineStyle::Double,
            ..stroke(accent, WIDE, 5)
        };
        self.edge(heading, BorderSegments::BOTTOM, separator, buf);
        Rect {
            x: area.x + 1,
            y: area.y + 2,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(3),
        }
    }

    /// The CPU box's header and footer: menu buttons, clock, refresh rate, battery and power on
    /// top; uptime, host, fans and toggles below.
    fn cpu_labels(&self, width: u16) -> (Vec<Label<'_>>, Vec<Label<'_>>) {
        let clock = self.clock();
        let refresh = format!("{}ms", TICKS[self.tick]);
        let mut top = vec![
            Label::title(1, "cpu", TEAL),
            Label::key(Place::Left, "menu", 0, TEAL, true),
            Label::key(Place::Left, "preset", 0, TEAL, true),
            Label::new(Place::At(width / 2 - 4), vec![Span::from(clock).bold()]),
            Label::new(
                Place::Right,
                vec![
                    Span::from("- ").fg(TEAL).bold(),
                    Span::from(refresh).bold(),
                    Span::from(" +").fg(TEAL).bold(),
                ],
            ),
        ];
        if let Some(battery) = &self.stats.battery {
            top.push(battery_label(battery));
        }
        if let Some(hardware) = &self.stats.hardware {
            top.push(Label::new(
                Place::Right,
                vec![
                    Span::from("pwr ").fg(GOLD),
                    Span::from(format!("{:.1}W", hardware.all_power)).bold(),
                ],
            ));
        }
        let host = System::host_name().unwrap_or_default();
        let mut bottom = vec![
            Label::new(
                Place::Left,
                vec![
                    Span::from("up ").fg(MUTED),
                    Span::from(duration(System::uptime())),
                ],
            ),
            Label::new(Place::Left, vec![Span::from(host).fg(MUTED)]),
            Label::key(Place::Right, "borders", 0, TEAL, self.borders.enabled),
            Label::key(Place::Right, "grid", 0, TEAL, self.show_grid),
        ];
        let fans = self
            .stats
            .hardware
            .as_ref()
            .map(|hardware| hardware.fans.clone())
            .unwrap_or_default();
        if fans.iter().any(|rpm| *rpm > 0) {
            let speeds: Vec<String> = fans.iter().map(|rpm| format!("{rpm}")).collect();
            bottom.push(Label::new(
                Place::Left,
                vec![
                    Span::from("fan ").fg(MUTED),
                    Span::from(format!("{} rpm", speeds.join(" / "))),
                ],
            ));
        }
        (top, bottom)
    }

    /// A big history graph on the left, and a nested box of per-core sparklines on the right.
    fn render_cpu(&self, area: Rect, buf: &mut Buffer) {
        let (top, bottom) = self.cpu_labels(area.width);
        let inner = self.panel(area, TEAL, &top, &bottom, buf);

        let cores = self.stats.system.cpus().len();
        // The core box's title, total and load rows, plus the GPU, cluster and power rows when
        // there is Apple Silicon hardware to read.
        let extra_rows = if self.stats.hardware.is_some() { 7 } else { 4 };
        let max_rows = usize::from(inner.height.saturating_sub(extra_rows).max(1));
        // Like btop, split the cores evenly into columns rather than filling one first.
        let columns = cores.div_ceil(max_rows).max(if cores > 6 { 2 } else { 1 });
        let rows = cores.div_ceil(columns);
        let columns = columns as u16;
        let [graph, _, side] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(1),
            // The core columns plus the docked pane's one-column margins.
            Constraint::Length(columns * 20 + 2),
        ])
        .areas(inner);

        let (graph, gpu_graph) = if self.stats.hardware.is_some() {
            // btop's upper and lower graphs: the CPU rises, the GPU hangs below a centreline.
            let [upper, lower] =
                Layout::vertical([Constraint::Percentage(70), Constraint::Fill(1)]).areas(graph);
            (upper, Some(lower))
        } else {
            (graph, None)
        };
        self.render_graph(&self.stats.cpu, graph, false, heat, buf);
        self.render_time_grid(graph, buf);
        self.render_average_line(&self.stats.cpu, graph, buf);
        if let (Some(lower), Some(hardware)) = (gpu_graph, &self.stats.hardware) {
            let shade = |t| mix(VIOLET, PANEL, 0.45 + t * 0.55);
            self.render_graph(&self.stats.gpu, lower, true, shade, buf);
            self.render_time_grid(lower, buf);
            let centre = Rect {
                y: graph.bottom() - 1,
                height: 1,
                ..graph
            };
            self.edge(centre, BorderSegments::BOTTOM, stroke(LINE, THIN, 0), buf);
            let label = Rect {
                y: lower.bottom().saturating_sub(1),
                height: 1,
                ..lower
            };
            Line::from(vec![
                Span::from("gpu ").fg(MUTED),
                Span::from(format!("{:.0}%", hardware.gpu * 100.0))
                    .fg(VIOLET)
                    .bold(),
            ])
            .render(label, buf);
        }
        let usage = self.stats.system.global_cpu_usage();
        Line::from(vec![
            Span::from("total ").fg(MUTED),
            Span::from(format!("{usage:.0}%"))
                .fg(heat(usage / 100.0))
                .bold(),
        ])
        .render(graph, buf);
        let hardware_rows = if self.stats.hardware.is_some() { 3 } else { 0 };
        let height = (rows as u16 + 3 + hardware_rows).min(inner.height);
        self.render_cores(
            centred(to_right_edge(side, inner), height),
            rows,
            columns,
            buf,
        );
    }

    /// Vertical gridlines every ten cells, on the cell edges.
    fn render_time_grid(&self, graph: Rect, buf: &mut Buffer) {
        let mut x = graph.right().saturating_sub(10);
        while x > graph.x {
            let line = Rect::new(x, graph.y, 1, graph.height);
            self.edge(line, BorderSegments::LEFT, stroke(FAINT, HAIR, 0), buf);
            x = x.saturating_sub(10);
        }
    }

    /// A gold line at the average of the visible history, on the edge between two rows.
    fn render_average_line(&self, values: &VecDeque<f32>, graph: Rect, buf: &mut Buffer) {
        let visible = usize::from(graph.width) * 2;
        let recent: Vec<f32> = values.iter().rev().take(visible).copied().collect();
        if recent.is_empty() || graph.height == 0 {
            return;
        }
        let average = recent.iter().sum::<f32>() / recent.len() as f32;
        let rows_up = (average * f32::from(graph.height)).round() as u16;
        let y = graph.bottom().saturating_sub(rows_up.max(1));
        // A cell has one stroke, so the line replaces the gridlines on its row rather than
        // turning them gold.
        if self.borders.enabled {
            for x in graph.left()..graph.right() {
                if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                    cell.border = CellBorder {
                        segments: BorderSegments::TOP,
                        ..stroke(GOLD, THIN, 0)
                    };
                }
            }
        }
        let text = format!(" avg {:.0}% ", average * 100.0);
        let label =
            Rect::new(graph.right().saturating_sub(text.len() as u16), y, 9, 1).intersection(graph);
        Line::from(text).fg(GOLD).render(label, buf);
    }

    /// The nested box beside the CPU graph: a total gauge with the temperature, a sparkline per
    /// core and the load average.
    fn render_cores(&self, side: Rect, rows: usize, columns: u16, buf: &mut Buffer) {
        let cores = self.stats.system.cpus();
        let usage = self.stats.system.global_cpu_usage();
        let brand = cores
            .first()
            .map(|cpu| cpu.brand().trim().to_string())
            .unwrap_or_default();
        let side = self.dock(side, Dock::Right, Some(Span::from(brand).bold()), buf);
        let hardware_rows = u16::from(self.stats.hardware.is_some());
        let [total, gpu, list, _, clusters, power, load] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(hardware_rows),
            Constraint::Length(rows as u16),
            Constraint::Fill(1),
            Constraint::Length(hardware_rows),
            Constraint::Length(hardware_rows),
            Constraint::Length(1),
        ])
        .areas(side);
        if let Some(hardware) = &self.stats.hardware {
            self.render_hardware(hardware, gpu, clusters, power, buf);
        }
        let [label, gauge, _, spark, value, temperature] = summary_columns(total);
        Line::from("CPU").fg(TEAL).bold().render(label, buf);
        render_squares(gauge, usage / 100.0, heat, buf);
        self.render_graph(&self.stats.cpu, spark, false, heat, buf);
        Line::from(format!("{usage:>4.0}%"))
            .fg(heat(usage / 100.0))
            .render(value, buf);
        if let Some(celsius) = self.stats.cpu_temperature() {
            Line::from(format!("{celsius:>4.0}°C"))
                .fg(heat((celsius - 30.0) / 70.0))
                .alignment(Alignment::Right)
                .render(temperature, buf);
        }

        let column_areas =
            Layout::horizontal(vec![Constraint::Length(20); columns.into()]).split(list);
        // A rule on the cell edge between core columns, like btop's.
        for column in column_areas.iter().skip(1) {
            let rule = Rect {
                width: 1,
                ..*column
            };
            self.edge(rule, BorderSegments::LEFT, stroke(LINE, HAIR, 0), buf);
        }
        for (index, core) in cores.iter().enumerate() {
            let column = column_areas[index / rows];
            // Later columns start a cell in, clear of the rule before them.
            let indent = u16::from(index >= rows);
            let row = Rect::new(column.x + indent, column.y + (index % rows) as u16, 19, 1);
            let [label, spark, value, temperature] = Layout::horizontal([
                Constraint::Length(4),
                Constraint::Fill(1),
                Constraint::Length(5),
                Constraint::Length(5),
            ])
            .areas(row);
            // Sensors don't map one to one onto cores, so share them out in order, as btop does.
            let celsius = self
                .stats
                .hardware
                .as_ref()
                .map(|hardware| &hardware.core_temperatures)
                .filter(|temperatures| !temperatures.is_empty())
                .map(|temperatures| temperatures[index % temperatures.len()])
                // Without per-core sensors, show the CPU's temperature on every core, as btop
                // does.
                .or_else(|| self.stats.cpu_temperature());
            if let Some(celsius) = celsius {
                Line::from(format!("{celsius:>3.0}°C"))
                    .fg(heat((celsius - 30.0) / 70.0))
                    .right_aligned()
                    .render(temperature, buf);
            }
            let ratio = core.cpu_usage() / 100.0;
            Line::from(format!("C{index}")).fg(MUTED).render(label, buf);
            self.render_graph(&self.stats.cores[index], spark, false, heat, buf);
            // A baseline under each sparkline.
            self.edge(spark, BorderSegments::BOTTOM, stroke(FAINT, HAIR, 0), buf);
            Line::from(format!("{:>4.0}%", core.cpu_usage()))
                .fg(heat(ratio))
                .render(value, buf);
        }
        let average = System::load_average();
        Line::from(vec![
            Span::from("load ").fg(MUTED),
            Span::from(format!(
                "{:.2} {:.2} {:.2}",
                average.one, average.five, average.fifteen
            )),
        ])
        .alignment(Alignment::Right)
        .render(load, buf);
    }

    /// The GPU gauge with its temperature, the CPU cluster frequencies and the power draw.
    fn render_hardware(
        &self,
        hardware: &Hardware,
        gpu: Rect,
        clusters: Rect,
        power: Rect,
        buf: &mut Buffer,
    ) {
        let [label, gauge, _, spark, value, temperature] = summary_columns(gpu);
        Line::from("GPU").fg(VIOLET).bold().render(label, buf);
        render_squares(gauge, hardware.gpu, |_| VIOLET, buf);
        self.render_graph(&self.stats.gpu, spark, false, |_| VIOLET, buf);
        Line::from(format!("{:>4.0}%", hardware.gpu * 100.0))
            .fg(VIOLET)
            .render(value, buf);
        if let Some(celsius) = hardware.gpu_temperature {
            Line::from(format!("{celsius:>4.0}°C"))
                .fg(heat((celsius - 30.0) / 70.0))
                .right_aligned()
                .render(temperature, buf);
        }

        let mut spans = Vec::new();
        for (name, mhz) in &hardware.clusters {
            spans.push(Span::from(format!("{name}-cluster ")).fg(MUTED));
            spans.push(Span::from(format!("{:.2} GHz  ", f64::from(*mhz) / 1000.0)));
        }
        spans.push(Span::from("gpu ").fg(MUTED));
        spans.push(Span::from(format!("{} MHz", hardware.gpu_mhz)));
        Line::from(spans).render(clusters, buf);

        let watts = |label: &'static str, value: f32, color: Color| {
            [
                Span::from(label).fg(MUTED),
                Span::from(format!("{value:.1}W  ")).fg(color),
            ]
        };
        Line::from(
            [
                watts("cpu ", hardware.cpu_power, TEAL),
                watts("gpu ", hardware.gpu_power, VIOLET),
                watts("ane ", hardware.ane_power, BLUE),
            ]
            .concat(),
        )
        .render(power, buf);
    }

    /// Memory meters and disk meters side by side, split by a rule on a cell edge.
    fn render_memory(&self, area: Rect, buf: &mut Buffer) {
        let system = &self.stats.system;
        let split = area.width * 45 / 100;
        let total = system.total_memory();
        let top = [
            Label::title(2, "mem", GOLD),
            Label::new(
                Place::At(split + 1),
                vec![Span::from("disks").fg(GOLD).bold()],
            ),
            Label::key(Place::Right, "io", 0, GOLD, true),
        ];
        let bottom = [Label::new(
            Place::Left,
            vec![
                Span::from("Total: ").fg(MUTED),
                Span::from(bytes(total)).bold(),
            ],
        )];
        let inner = self.panel(area, GOLD, &top, &bottom, buf);
        let [memory, disks] = Layout::horizontal([
            Constraint::Length(split.saturating_sub(2)),
            Constraint::Fill(1),
        ])
        .spacing(2)
        .areas(inner);
        let rule = Rect {
            x: disks.x.saturating_sub(1),
            width: 1,
            ..inner
        };
        let rule_stroke = stroke(mix(GOLD, PANEL, 0.4), THIN, 0);
        self.edge(rule, BorderSegments::LEFT, rule_stroke, buf);

        let meters = [
            ("Used", system.used_memory(), GOLD),
            ("Avail", system.available_memory(), TEAL),
            ("Cache", self.stats.cached.unwrap_or(0), BLUE),
            ("Free", system.free_memory(), mix(BLUE, TEXT, 0.4)),
            ("Swap", system.used_swap(), VIOLET),
        ];
        let rows = Layout::vertical([Constraint::Length(2); 5])
            .spacing(1)
            .split(memory);
        for (index, ((label, used, color), row)) in meters.into_iter().zip(rows.iter()).enumerate()
        {
            let total = if label == "Swap" {
                system.total_swap()
            } else {
                total
            };
            let ratio = if total == 0 {
                0.0
            } else {
                used as f32 / total as f32
            };
            // btop's layout: the label and size, then the percentage beside the meter and graph.
            let [text, graphic] = Layout::vertical([Constraint::Length(1); 2]).areas(*row);
            self.render_rule_row(
                text,
                Line::from(format!("{label}:")).fg(MUTED),
                Line::from(bytes(used)),
                buf,
            );
            let [percent, gauge, _, graph] = Layout::horizontal([
                Constraint::Length(5),
                Constraint::Percentage(40),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .areas(graphic);
            Line::from(format!("{:.0}%", ratio * 100.0))
                .fg(color)
                .bold()
                .render(percent, buf);
            let shade = |t| mix(color, PANEL, 0.35 + t * 0.65);
            render_squares(gauge, ratio, shade, buf);
            self.render_graph(&self.stats.memory[index], graph, false, |_| color, buf);
            self.edge(graph, BorderSegments::BOTTOM, stroke(FAINT, HAIR, 0), buf);
        }
        self.render_disks(disks, buf);
    }

    fn render_disks(&self, area: Rect, buf: &mut Buffer) {
        let disks = &self.stats.volumes;
        let rows = Layout::vertical(vec![Constraint::Length(3); disks.len()]).split(area);
        for (disk, row) in disks.iter().zip(rows.iter()) {
            let [name, io, usage] = Layout::vertical([Constraint::Length(1); 3]).areas(*row);
            let label = disk.name.as_str();
            let total = disk.total;
            let used = total - disk.available;
            let ratio = used as f32 / total.max(1) as f32;
            self.render_rule_row(
                name,
                Line::from(label).bold(),
                Line::from(compact_bytes(total)).bold(),
                buf,
            );
            // btop's compact rows: "IO" and a graph, then "U", a meter and the used size.
            let [io_text, io_graph] =
                Layout::horizontal([Constraint::Length(3), Constraint::Fill(1)]).areas(io);
            Line::from("IO").fg(MUTED).render(io_text, buf);
            if let Some(history) = self.stats.disk_history.get(&disk.mount) {
                let peak = history.iter().copied().fold(1024.0 * 1024.0, f64::max);
                let values = history.iter().map(|value| (value / peak) as f32).collect();
                self.render_graph(&values, io_graph, false, heat, buf);
            }
            let [text, gauge, _, amount] = Layout::horizontal([
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Length(5),
            ])
            .areas(usage);
            Line::from("U").fg(MUTED).render(text, buf);
            render_squares(gauge, ratio, heat, buf);
            Line::from(compact_bytes(used))
                .right_aligned()
                .render(amount, buf);
        }
    }

    /// Download rises from a centreline and upload hangs below it, with a nested stats box.
    fn render_network(&self, area: Rect, buf: &mut Buffer) {
        let interfaces = self.stats.interfaces();
        let name = interfaces
            .get(self.interface % interfaces.len().max(1))
            .cloned()
            .unwrap_or_default();
        let empty = VecDeque::new();
        let received = self.stats.received.get(&name).unwrap_or(&empty);
        let transmitted = self.stats.transmitted.get(&name).unwrap_or(&empty);
        let address = self
            .stats
            .networks
            .get(&name)
            .and_then(|data| {
                data.ip_networks()
                    .iter()
                    .find(|network| network.addr.is_ipv4())
            })
            .map(|network| network.addr.to_string())
            .unwrap_or_default();

        let width = usize::from(area.width) * 2;
        let peak =
            |values: &VecDeque<f64>| values.iter().rev().take(width).copied().fold(0.0, f64::max);
        // Like btop, each direction scales to its own recent peak, so a quiet upload still fills
        // its half of the graph.
        let down_scale = peak(received).max(1024.0);
        let up_scale = peak(transmitted).max(1024.0);
        let top = [
            Label::title(3, "net", BLUE),
            Label::new(Place::Left, vec![Span::from(address).bold()]),
            Label::new(
                Place::Right,
                vec![
                    Span::from(name.clone()).bold(),
                    Span::from(" i").fg(BLUE).bold(),
                    Span::from("face").fg(TEXT),
                ],
            ),
            Label::key(Place::Right, "auto", 0, BLUE, true),
            Label::key(Place::Right, "sync", 0, BLUE, true),
        ];
        let bottom = [Label::new(
            Place::Left,
            vec![
                Span::from("scale ").fg(MUTED),
                Span::from("▼ ").fg(BLUE),
                Span::from(format!("{}/s ", bytes(down_scale as u64))),
                Span::from("▲ ").fg(VIOLET),
                Span::from(format!("{}/s", bytes(up_scale as u64))),
            ],
        )];
        let inner = self.panel(area, BLUE, &top, &bottom, buf);
        // The widest line the stats pane shows, like "▼ Total: 36.5 GiB", plus its margins.
        let totals = self.stats.networks.get(&name).map_or((0, 0), |data| {
            (data.total_received(), data.total_transmitted())
        });
        let latest = |values: &VecDeque<f64>| values.back().copied().unwrap_or(0.0);
        let lines = [
            ("", format!("{}/s", bytes(latest(received) as u64))),
            ("", format!("{}/s", bytes(latest(transmitted) as u64))),
            ("Top: ", bits(peak(received))),
            ("Top: ", bits(peak(transmitted))),
            ("Total: ", bytes(totals.0)),
            ("Total: ", bytes(totals.1)),
        ];
        let widest = lines
            .iter()
            .map(|(label, value)| 2 + label.len() + 1 + value.len())
            .max()
            .unwrap_or(0) as u16;
        let [graph, _, stats] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(widest.max(8) + 3),
        ])
        .areas(inner);
        let [down, up] = Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).areas(graph);
        let normalise = |values: &VecDeque<f64>, scale: f64| -> VecDeque<f32> {
            values.iter().map(|value| (value / scale) as f32).collect()
        };
        let down_shade = |t| mix(BLUE, PANEL, 0.4 + t * 0.6);
        let up_shade = |t| mix(VIOLET, PANEL, 0.4 + t * 0.6);
        self.render_graph(
            &normalise(received, down_scale),
            down,
            false,
            down_shade,
            buf,
        );
        self.render_graph(&normalise(transmitted, up_scale), up, true, up_shade, buf);
        self.render_time_grid(graph, buf);
        let centre = Rect::new(graph.x, down.bottom().saturating_sub(1), graph.width, 1);
        self.edge(centre, BorderSegments::BOTTOM, stroke(LINE, THIN, 0), buf);

        let stats = centred(to_right_edge(stats, inner), 9.min(inner.height));
        let stats = self.dock(stats, Dock::Right, None, buf);
        self.render_network_stats(stats, &name, received, transmitted, width, buf);
    }

    /// The network box's side pane: current, peak and total figures for each direction.
    fn render_network_stats(
        &self,
        stats: Rect,
        name: &str,
        received: &VecDeque<f64>,
        transmitted: &VecDeque<f64>,
        width: usize,
        buf: &mut Buffer,
    ) {
        let peak =
            |values: &VecDeque<f64>| values.iter().rev().take(width).copied().fold(0.0, f64::max);
        let latest = |values: &VecDeque<f64>| values.back().copied().unwrap_or(0.0);
        let totals = self.stats.networks.get(name).map_or((0, 0), |data| {
            (data.total_received(), data.total_transmitted())
        });
        // btop's layout: "download" heads the pane and "upload" foots it, each with three rows of
        // figures on their side.
        let [down_title, down_stats, _, up_stats, up_title] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .areas(stats);
        Line::from("download")
            .fg(BLUE)
            .bold()
            .render(down_title, buf);
        self.edge(
            down_title,
            BorderSegments::BOTTOM,
            stroke(FAINT, HAIR, 0),
            buf,
        );
        Line::from("upload").fg(VIOLET).bold().render(up_title, buf);
        self.edge(up_title, BorderSegments::TOP, stroke(FAINT, HAIR, 0), buf);
        let sections = [
            ("▼", BLUE, down_stats, received, totals.0),
            ("▲", VIOLET, up_stats, transmitted, totals.1),
        ];
        for (arrow, color, area, values, total) in sections {
            let figures = [
                ("", format!("{}/s", bytes(latest(values) as u64))),
                ("Top: ", bits(peak(values))),
                ("Total: ", bytes(total)),
            ];
            for ((label, value), row) in figures.into_iter().zip(area.rows()) {
                Line::from(vec![
                    Span::from(format!("{arrow} ")).fg(color),
                    Span::from(label).fg(MUTED),
                ])
                .render(row, buf);
                Line::from(value).right_aligned().render(row, buf);
            }
        }
    }

    fn render_processes(&mut self, area: Rect, buf: &mut Buffer) {
        let processes = self.stats.processes(
            self.view.sort,
            self.view.reverse,
            &self.view.filter.text,
            self.view.tree,
        );
        self.selected = self.selected.min(processes.len().saturating_sub(1));
        let top = self.process_labels();
        let position = format!("{}/{}", self.selected + 1, processes.len());
        let bottom = process_footer(position);
        let inner = self.panel(area, RED, &top, &bottom, buf);
        self.selected_pid = processes.get(self.selected).map(|process| process.pid);
        let inner = match self.detailed {
            Some(pid) if inner.height > 16 => {
                let [detail, rest] = Layout::vertical([Constraint::Length(9), Constraint::Fill(1)])
                    .spacing(1)
                    .areas(inner);
                // Docked across the full width of the box, under its header.
                let detail = Rect {
                    x: detail.x - 1,
                    width: detail.width + 2,
                    ..detail
                };
                self.render_detail(pid, detail, buf);
                rest
            }
            _ => inner,
        };
        let [header, rows] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(inner);
        self.rows = rows;

        let visible = usize::from(rows.height);
        if self.selected < self.scroll {
            self.scroll = self.selected;
        } else if self.selected >= self.scroll + visible {
            self.scroll = self.selected + 1 - visible;
        } else {
            // The selection is already on screen.
        }

        // Like btop, drop the command and thread columns when there's no room for them.
        let wide = inner.width >= 90;
        let columns = process_columns(inner, wide);
        self.render_process_header(header, &columns, buf);

        for (offset, row) in rows.rows().enumerate() {
            let index = self.scroll + offset;
            let Some(process) = processes.get(index) else {
                break;
            };
            if index == self.selected {
                buf.set_style(row, Style::new().bg(mix(RED, PANEL, 0.2)));
                // A ring around the row, drawn in the row's own cells.
                self.outline_round(row, stroke(RED, THICK, 2), buf);
            } else if row.contains(self.mouse) {
                self.outline_round(row, stroke(mix(RED, PANEL, 0.5), HAIR, 1), buf);
            } else if index % 5 == 4 {
                // Group rows in fives with a faint rule on the edge below.
                self.edge(row, BorderSegments::BOTTOM, stroke(FAINT, HAIR, 0), buf);
            } else {
                // Other rows have no lines of their own.
            }
            render_process(process, row, &columns, buf);
            self.render_process_cpu(
                process,
                Rect {
                    y: row.y,
                    height: 1,
                    ..columns[6]
                },
                buf,
            );
        }
        self.render_scrollbar(rows, processes.len(), buf);
    }

    /// btop's detailed view: the process's CPU history beside a table of its status, a memory
    /// gauge and its command line, in a rounded box.
    fn render_detail(&self, pid: u32, area: Rect, buf: &mut Buffer) {
        let system = &self.stats.system;
        let process = system.process(Pid::from_u32(pid));
        let name = process.map_or_else(
            || "(ended)".to_string(),
            |process| process.name().to_string_lossy().into_owned(),
        );
        let title = Span::from(format!("{pid}  {name}")).fg(RED).bold();
        let inner = self.dock(area, Dock::Top, Some(title), buf);
        let [graph, info] = Layout::horizontal([Constraint::Percentage(34), Constraint::Fill(1)])
            .spacing(1)
            .areas(inner);
        let empty = VecDeque::new();
        let history = self.stats.process_cpu.get(&pid).unwrap_or(&empty);
        self.render_graph(history, graph, false, heat, buf);
        let cpu = history.back().copied().unwrap_or(0.0);
        Line::from(vec![
            Span::from("cpu ").fg(MUTED),
            Span::from(format!("{:.1}%", cpu * 100.0))
                .fg(heat(cpu))
                .bold(),
        ])
        .render(graph, buf);
        let divider = Rect {
            x: info.x,
            width: 1,
            ..info
        };
        self.edge(divider, BorderSegments::LEFT, stroke(LINE, THIN, 0), buf);
        let info = info.inner(Margin::new(2, 0));

        let Some(process) = process else {
            Line::from("This process has ended.")
                .fg(MUTED)
                .render(info, buf);
            return;
        };
        self.render_detail_info(process, info, buf);
    }

    /// The right of the detailed view: a status table with rules between its columns, a memory
    /// gauge and the command line.
    fn render_detail_info(&self, process: &sysinfo::Process, info: Rect, buf: &mut Buffer) {
        let system = &self.stats.system;
        let parent = process
            .parent()
            .and_then(|parent| system.process(parent))
            .map_or_else(String::new, |parent| {
                parent.name().to_string_lossy().into_owned()
            });
        let user = process
            .user_id()
            .and_then(|uid| self.stats.users.get_user_by_id(uid))
            .map_or_else(String::new, |user| user.name().to_string());
        let io = process.disk_usage();
        let fields = [
            ("Status", process.status().to_string()),
            ("Elapsed", duration(process.run_time())),
            ("Parent", parent),
            ("User", user),
            ("Read", bytes(io.total_read_bytes)),
            ("Written", bytes(io.total_written_bytes)),
        ];
        let [labels, values, _, memory, _, command] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(info);
        // Each field is as wide as its label or value, rather than sharing the width evenly.
        let widths = fields.iter().map(|(label, value)| {
            Constraint::Length(label.len().max(value.chars().count()) as u16 + 2)
        });
        let columns = Layout::horizontal(widths).split(info);
        for (index, ((label, value), column)) in fields.iter().zip(columns.iter()).enumerate() {
            let at = |row: Rect| Rect {
                y: row.y,
                height: 1,
                ..*column
            };
            Line::from(*label)
                .fg(MUTED)
                .bold()
                .centered()
                .render(at(labels), buf);
            Line::from(value.as_str())
                .centered()
                .render(at(values), buf);
            if index > 0 {
                let rule = Rect {
                    width: 1,
                    height: 2,
                    ..at(labels)
                };
                self.edge(rule, BorderSegments::LEFT, stroke(LINE, HAIR, 0), buf);
            }
        }
        self.edge(
            labels,
            BorderSegments::BOTTOM,
            stroke(mix(RED, RAISED, 0.5), HAIR, 0),
            buf,
        );

        let ratio = process.memory() as f32 / system.total_memory().max(1) as f32;
        let [text, gauge] =
            Layout::horizontal([Constraint::Length(18), Constraint::Fill(1)]).areas(memory);
        Line::from(vec![
            Span::from("Memory ").fg(MUTED),
            Span::from(bytes(process.memory())).bold(),
        ])
        .render(text, buf);
        render_squares(gauge, ratio, |t| mix(GOLD, RAISED, 0.4 + t * 0.6), buf);

        let command_line = process
            .cmd()
            .iter()
            .map(|part| part.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ");
        Paragraph::new(Line::from(vec![
            Span::from("Cmd ").fg(MUTED),
            Span::from(command_line).fg(mix(RED, TEXT, 0.3)),
        ]))
        .wrap(Wrap { trim: true })
        .render(command, buf);
    }

    /// A process's cpu cell: a mini graph of its recent use, then the current figure.
    fn render_process_cpu(&self, process: &Proc, cell: Rect, buf: &mut Buffer) {
        // The last cell stays clear for the scrollbar on its right edge.
        let [spark, _, value, _] = Layout::horizontal([
            Constraint::Length(5),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(cell);
        self.render_graph(&process.history, spark, false, heat, buf);
        let ratio = process.cpu / 100.0;
        let text = if process.cpu >= 100.0 {
            format!("{:.0}", process.cpu)
        } else {
            format!("{:.1}", process.cpu)
        };
        Line::from(text)
            .fg(heat(ratio))
            .right_aligned()
            .render(value, buf);
    }

    /// The column headings, with a thick underline under the column the table is sorted by.
    fn render_process_header(&self, header: Rect, columns: &[Rect], buf: &mut Buffer) {
        let headers = [
            "Pid:", "Program:", "Command:", "Threads:", "User:", "MemB", "Cpu%",
        ];
        let underline = stroke(mix(RED, PANEL, 0.5), THIN, 0);
        self.edge(header, BorderSegments::BOTTOM, underline, buf);
        for (index, column) in columns.iter().enumerate() {
            let cell = Rect {
                height: 1,
                ..*column
            };
            let sorted = index == self.view.sort.column();
            let text = Line::from(headers[index]).bold();
            let text = if sorted { text.fg(RED) } else { text };
            let text = if matches!(index, 0 | 3 | 5 | 6) {
                text.right_aligned()
            } else {
                text
            };
            text.render(cell, buf);
            if sorted {
                self.edge(cell, BorderSegments::BOTTOM, stroke(RED, THICK, 1), buf);
            }
        }
    }

    /// A scrollbar on the right edge of the table's rows: a hairline track with a thick thumb.
    fn render_scrollbar(&self, rows: Rect, total: usize, buf: &mut Buffer) {
        if rows.is_empty() || total <= usize::from(rows.height) {
            return;
        }
        let height = usize::from(rows.height);
        let track = Rect {
            x: rows.right() - 1,
            width: 1,
            ..rows
        };
        self.edge(track, BorderSegments::RIGHT, stroke(LINE, HAIR, 1), buf);
        let thumb_height = (height * height / total).max(1);
        let thumb_top = self.scroll * height / total;
        let thumb = Rect {
            y: rows.y + thumb_top as u16,
            height: thumb_height as u16,
            ..track
        }
        .intersection(track);
        self.edge(thumb, BorderSegments::RIGHT, stroke(RED, THICK, 3), buf);
    }

    fn process_labels(&self) -> Vec<Label<'_>> {
        let filter = if self.view.filter.editing || !self.view.filter.text.is_empty() {
            let cursor = if self.view.filter.editing { "▏" } else { "" };
            Label::new(
                Place::Left,
                vec![
                    Span::from("f").fg(RED).bold(),
                    Span::from(format!("ilter: {}{cursor}", self.view.filter.text)).fg(TEXT),
                ],
            )
        } else {
            Label::key(Place::Left, "filter", 0, RED, true)
        };
        vec![
            Label::title(4, "proc", RED),
            filter,
            Label::new(
                Place::Right,
                vec![
                    Span::from("‹ ").fg(RED).bold(),
                    Span::from(self.view.sort.label()).bold(),
                    Span::from(" ›").fg(RED).bold(),
                ],
            ),
            Label::key(Place::Right, "reverse", 0, RED, self.view.reverse),
            Label::key(Place::Right, "tree", 0, RED, self.view.tree),
            Label::key(Place::Right, "pause", 0, RED, self.view.paused),
        ]
    }

    /// The local wall clock time.
    fn clock(&self) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |time| time.as_secs() as i64);
        let seconds = (now + self.utc_offset).rem_euclid(86_400);
        format!(
            "{:02}:{:02}:{:02}",
            seconds / 3_600,
            seconds % 3_600 / 60,
            seconds % 60
        )
    }

    /// Draws a box and returns the area inside it.
    ///
    /// With cell borders the outline sits on the outer edges of the box's cells, exactly where
    /// its background starts. The labels sit in tinted header and footer rows, split by rules on
    /// cell edges.
    fn panel(
        &self,
        area: Rect,
        accent: Color,
        top: &[Label],
        bottom: &[Label],
        buf: &mut Buffer,
    ) -> Rect {
        buf.set_style(area, Style::new().bg(PANEL));
        let line_color = mix(accent, PANEL, 0.6);
        if !self.borders.enabled {
            let mut block = Block::bordered().border_style(Style::new().fg(line_color));
            for (labels, is_top) in [(top, true), (bottom, false)] {
                for label in labels {
                    let line = Line::from(label.spans.clone());
                    let line = match label.place {
                        Place::Right => line.right_aligned(),
                        Place::Left | Place::At(_) => line,
                    };
                    block = if is_top {
                        block.title_top(line)
                    } else {
                        block.title_bottom(line)
                    };
                }
            }
            let inner = block.inner(area);
            block.render(area, buf);
            return inner.inner(Margin::new(1, 0));
        }
        if area.width < 6 || area.height < 3 {
            return Rect::default();
        }
        let first = Rect { height: 1, ..area };
        let last = Rect {
            y: area.bottom() - 1,
            height: 1,
            ..area
        };
        let tint = Style::new().bg(mix(accent, PANEL, 0.18));
        buf.set_style(first, tint);
        buf.set_style(last, tint);
        self.outline_round(area, stroke(line_color, THIN, 1), buf);
        let rule = stroke(mix(accent, PANEL, 0.35), THIN, 0);
        self.render_labels(first, top, rule, buf);
        self.render_labels(last, bottom, rule, buf);
        Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width - 2,
            height: area.height - 2,
        }
    }

    /// Draws a raised pane docked against the inside of a box and returns the area inside it.
    ///
    /// The pane runs to the box's edge on the side it docks to, so it adds no line there. A title
    /// sits in its first row, underlined on the cell edge.
    fn dock(&self, area: Rect, side: Dock, title: Option<Span>, buf: &mut Buffer) -> Rect {
        buf.set_style(area, Style::new().bg(RAISED));
        if !self.borders.enabled {
            let mut block = Block::bordered().border_style(Style::new().fg(LINE));
            if let Some(title) = title {
                block = block.title(Line::from(title));
            }
            let inner = block.inner(area);
            block.render(area, buf);
            return inner;
        }
        let line = stroke(LINE, THIN, 1);
        let bottom = Rect {
            y: area.bottom().saturating_sub(1),
            height: 1,
            ..area
        };
        match side {
            Dock::Right => {
                // Open only on the right, where it meets the box's own outline.
                self.edge(Rect { width: 1, ..area }, BorderSegments::LEFT, line, buf);
                self.edge(Rect { height: 1, ..area }, BorderSegments::TOP, line, buf);
                self.edge(bottom, BorderSegments::BOTTOM, line, buf);
            }
            Dock::Top => self.edge(bottom, BorderSegments::BOTTOM, line, buf),
        }
        let mut inner = area.inner(Margin::new(1, 0));
        if matches!(side, Dock::Right) {
            // The pane's last column is the box's outline, so keep a cell clear inside it.
            inner.width = inner.width.saturating_sub(1);
        }
        let Some(title) = title else {
            return inner;
        };
        let first = Rect { height: 1, ..inner };
        Line::from(title).render(first, buf);
        self.edge(first, BorderSegments::BOTTOM, stroke(LINE, HAIR, 0), buf);
        Rect {
            y: inner.y + 1,
            height: inner.height.saturating_sub(1),
            ..inner
        }
    }

    /// Places labels along a header or footer row, with a rule on the cell edge between each.
    fn render_labels(&self, row: Rect, labels: &[Label], rule: CellBorder, buf: &mut Buffer) {
        let mut left = row.x;
        let mut right = row.right();
        // Cells already taken, so a label that would overlap another is left out instead.
        let mut taken: Vec<(u16, u16)> = Vec::new();
        for label in labels {
            let width = label.width() + 2;
            let (x, divider) = match label.place {
                Place::Left => (left, BorderSegments::RIGHT),
                Place::Right => (right.saturating_sub(width), BorderSegments::LEFT),
                Place::At(offset) => (row.x + offset, BorderSegments::empty()),
            };
            let end = x + width;
            let overlaps = taken.iter().any(|&(start, stop)| x < stop && start < end);
            if overlaps || x < row.x || end > row.right() || (right < left + width) {
                continue;
            }
            match label.place {
                Place::Left => left = end,
                Place::Right => right = x,
                Place::At(_) => {}
            }
            taken.push((x, end));
            let area = Rect::new(x, row.y, width, 1);
            let mut spans = vec![Span::from(" ")];
            spans.extend(label.spans.iter().cloned());
            Line::from(spans).render(area, buf);
            if let Some(key) = label.key {
                self.hits.borrow_mut().push((area, key));
            }
            if divider.is_empty() || area.is_empty() {
                continue;
            }
            let edge_x = if divider == BorderSegments::RIGHT {
                area.right() - 1
            } else {
                area.x
            };
            self.edge(Rect::new(edge_x, row.y, 1, 1), divider, rule, buf);
        }
    }

    /// A row with a line through its middle, broken around a label on the left and a value on
    /// the right, like btop's `├─Used:──34.6─GiB─┤`. The line takes none of the row's text space.
    fn render_rule_row(&self, row: Rect, left: Line, right: Line, buf: &mut Buffer) {
        if row.is_empty() {
            return;
        }
        self.edge(row, BorderSegments::HORIZONTAL, stroke(LINE, HAIR, 0), buf);
        // Text sits flush at the ends, with one cell of gap before the line.
        let right_width = (right.width() as u16 + 1).min(row.width);
        // The value wins: a long label is cut short rather than running into it.
        let left_width = (left.width() as u16 + 1).min(row.width - right_width);
        let left_area = Rect {
            width: left_width,
            ..row
        };
        let right_area = Rect {
            x: row.right() - right_width,
            width: right_width,
            ..row
        };
        for area in [left_area, right_area] {
            for position in area.positions() {
                if let Some(cell) = buf.cell_mut(position) {
                    cell.border.segments.remove(BorderSegments::HORIZONTAL);
                }
            }
        }
        left.render(left_area, buf);
        right.right_aligned().render(right_area, buf);
    }

    /// Draws a filled braille history graph, newest on the right. Each cell holds two samples
    /// and four steps of height. Rising graphs fill from the bottom, hanging ones from the top.
    fn render_graph(
        &self,
        values: &VecDeque<f32>,
        area: Rect,
        hanging: bool,
        color: impl Fn(f32) -> Color,
        buf: &mut Buffer,
    ) {
        if area.is_empty() {
            return;
        }
        if area.height == 1 {
            // A dotted baseline, like btop's, so an idle graph still reads as a graph.
            for position in area.positions() {
                if let Some(cell) = buf.cell_mut(position) {
                    cell.set_symbol("⣀").set_fg(FAINT);
                }
            }
        }
        let dots_high = usize::from(area.height) * 4;
        let samples = usize::from(area.width) * 2;
        let skip = values.len().saturating_sub(samples);
        // Right-align the newest sample.
        let offset = samples.saturating_sub(values.len());
        for (index, value) in values.iter().skip(skip).enumerate() {
            let column = offset + index;
            let x = area.x + (column / 2) as u16;
            let right = column % 2 == 1;
            let filled = (value.clamp(0.0, 1.0) * dots_high as f32).round() as usize;
            for dot in 0..filled {
                let row_from_edge = dot / 4;
                let y = if hanging {
                    area.y + row_from_edge as u16
                } else {
                    area.bottom() - 1 - row_from_edge as u16
                };
                // Braille rows count down from the top of the cell.
                let within = if hanging { dot % 4 } else { 3 - dot % 4 };
                let Some(cell) = buf.cell_mut(Position::new(x, y)) else {
                    continue;
                };
                let bits = braille_bits(cell.symbol()) | braille_dot(within, right);
                let shade = color(row_from_edge as f32 / f32::from(area.height));
                cell.set_char(char::from_u32(0x2800 + bits).unwrap_or(' '))
                    .set_fg(shade);
            }
        }
        if !hanging && area.height >= 4 {
            // Gridlines at each quarter, on the edges between rows.
            for quarter in 1..4 {
                let y = area.bottom() - 1 - area.height * quarter / 4;
                let line = Rect::new(area.x, y, area.width, 1).intersection(area);
                self.edge(line, BorderSegments::BOTTOM, stroke(LINE, HAIR, 0), buf);
            }
        }
    }

    fn outline(&self, area: Rect, border: CellBorder, buf: &mut Buffer) {
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
        self.edge(top, BorderSegments::TOP, border, buf);
        self.edge(bottom, BorderSegments::BOTTOM, border, buf);
        self.edge(left, BorderSegments::LEFT, border, buf);
        self.edge(right, BorderSegments::RIGHT, border, buf);
    }

    /// An outline with rounded corners. Only top-level shapes are rounded: rounding inside
    /// rounding looks busy.
    fn outline_round(&self, area: Rect, border: CellBorder, buf: &mut Buffer) {
        self.outline(area, border, buf);
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

    /// Adds segments to every cell in `area`, drawn with the given stroke.
    ///
    /// A cell has one stroke for all its segments, so a cell that already has a higher priority
    /// stroke is skipped: a decoration that reaches a box's outline stops there rather than
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
}

/// btop's battery readout: direction, percentage and a ten-step meter, then time remaining.
fn battery_label(battery: &Battery) -> Label<'static> {
    let arrow = if battery.charging { "▲" } else { "▼" };
    let ratio = f32::from(battery.percent) / 100.0;
    let filled = usize::from(battery.percent).div_ceil(10);
    let mut spans = vec![
        Span::from(format!("BAT{arrow} ")).bold(),
        Span::from(format!("{}% ", battery.percent)).fg(heat(1.0 - ratio)),
    ];
    for step in 0..10 {
        let color = if step < filled {
            heat(1.0 - step as f32 / 10.0)
        } else {
            LINE
        };
        spans.push(Span::from("■").fg(color));
    }
    if !battery.remaining.is_empty() {
        spans.push(Span::from(format!(" {}", battery.remaining)));
    }
    Label::new(Place::Right, spans)
}

/// The process box's footer: selection and sort hints, btop's process buttons, and the
/// position in the list.
fn process_footer(position: String) -> Vec<Label<'static>> {
    vec![
        Label::new(
            Place::Left,
            vec![
                Span::from("↑").fg(RED).bold(),
                Span::from(" select ").fg(TEXT),
                Span::from("↓").fg(RED).bold(),
            ],
        ),
        Label::new(
            Place::Left,
            vec![
                Span::from("←").fg(RED).bold(),
                Span::from(" sort ").fg(TEXT),
                Span::from("→").fg(RED).bold(),
            ],
        ),
        Label::new(
            Place::Left,
            vec![Span::from("↵").fg(RED).bold(), Span::from(" info").fg(TEXT)],
        )
        .on_click('\r'),
        // btop's signal buttons, shown dimmed: this demo doesn't send signals.
        Label::key(Place::Left, "terminate", 0, RED, false).inert(),
        Label::key(Place::Left, "kill", 0, RED, false).inert(),
        Label::key(Place::Left, "signals", 0, RED, false).inert(),
        Label::key(Place::Left, "nice", 0, RED, false).inert(),
        Label::new(Place::Right, vec![Span::from(position).fg(MUTED)]),
    ]
}

/// One row of the process table. The mini graph in the cpu column is drawn by the caller.
fn render_process(process: &Proc, row: Rect, columns: &[Rect], buf: &mut Buffer) {
    let cells = |area: Rect| Rect {
        y: row.y,
        height: 1,
        ..area
    };
    let name_color = mix(RED, TEXT, 0.3);
    let values = [
        Line::from(process.pid.to_string())
            .fg(MUTED)
            .right_aligned(),
        Line::from(vec![
            Span::from(process.indent.as_str()).fg(LINE),
            Span::from(process.name.as_str()).fg(name_color),
        ]),
        Line::from(process.command.as_str()).fg(MUTED),
        Line::from(
            process
                .threads
                .map_or_else(String::new, |threads| threads.to_string()),
        )
        .right_aligned(),
        Line::from(short_user(&process.user)),
        Line::from(short_bytes(process.memory)).right_aligned(),
        Line::default(),
    ];
    for (value, column) in values.into_iter().zip(columns.iter()) {
        value.render(cells(*column), buf);
    }
}

/// A stroke drawn inside the cell, with the given colour, width and priority.
const fn stroke(color: Color, width: u8, priority: u8) -> CellBorder {
    CellBorder {
        width,
        priority,
        centered: false,
        ..CellBorder::new(BorderSegments::empty(), color)
    }
}

/// The braille dot bits already set in a symbol, or none if it isn't braille.
fn braille_bits(symbol: &str) -> u32 {
    symbol
        .chars()
        .next()
        .map(u32::from)
        .filter(|code| (0x2800..=0x28ff).contains(code))
        .map_or(0, |code| code - 0x2800)
}

/// The bit for a braille dot, by row from the top (0 to 3) and column.
const fn braille_dot(row: usize, right: bool) -> u32 {
    match (row, right) {
        (0, false) => 0x01,
        (1, false) => 0x02,
        (2, false) => 0x04,
        (3, false) => 0x40,
        (0, true) => 0x08,
        (1, true) => 0x10,
        (2, true) => 0x20,
        _ => 0x80,
    }
}

/// Widens `area` to the right edge of the box whose inside is `inner`, so a docked pane meets
/// the box's outline.
const fn to_right_edge(area: Rect, inner: Rect) -> Rect {
    Rect {
        width: inner.right() + 1 - area.x,
        ..area
    }
}

/// The key a clickable label presses. Labels store a `char`, with `\r` standing for Enter.
const fn key_code(key: char) -> KeyCode {
    if key == '\r' {
        KeyCode::Enter
    } else {
        KeyCode::Char(key)
    }
}

/// A process's thread count, from the kernel's task info.
#[cfg(target_os = "macos")]
fn thread_count(pid: u32) -> Option<u32> {
    use libproc::proc_pid::pidinfo;
    use libproc::task_info::TaskInfo;
    let info = pidinfo::<TaskInfo>(i32::try_from(pid).ok()?, 0).ok()?;
    u32::try_from(info.pti_threadnum).ok()
}

#[cfg(not(target_os = "macos"))]
const fn thread_count(_pid: u32) -> Option<u32> {
    None
}

/// File-backed memory the system can drop, which btop reports as cached: the pageable
/// external page count times the page size, from `sysctl`.
fn cached_memory() -> Option<u64> {
    let output = Command::new("sysctl")
        .args(["-n", "vm.page_pageable_external_count", "hw.pagesize"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let mut values = text.lines().map(|line| line.trim().parse::<u64>().ok());
    Some(values.next()?? * values.next()??)
}

/// The CPU and GPU summary rows: label, gauge, gap, history graph, percentage, temperature.
fn summary_columns(row: Rect) -> [Rect; 6] {
    Layout::horizontal([
        Constraint::Length(4),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(10),
        Constraint::Length(5),
        Constraint::Length(7),
    ])
    .areas(row)
}

/// The part of `area` `height` rows tall, centred vertically.
const fn centred(area: Rect, height: u16) -> Rect {
    Rect {
        y: area.y + (area.height - height) / 2,
        height,
        ..area
    }
}

/// btop's square meter: one `■` per cell, filled ones shaded by `color` along the meter and
/// the rest dim.
fn render_squares(area: Rect, ratio: f32, color: impl Fn(f32) -> Color, buf: &mut Buffer) {
    let filled = (ratio.clamp(0.0, 1.0) * f32::from(area.width)).round() as u16;
    for (index, position) in area.positions().enumerate() {
        let color = if (index as u16) < filled {
            color(index as f32 / f32::from(area.width.max(1)))
        } else {
            TRACK
        };
        if let Some(cell) = buf.cell_mut(position) {
            cell.set_symbol("■").set_fg(color);
        }
    }
}

/// The process table's columns: pid, program, command, threads, user, memory and cpu. Like
/// btop, a narrow table drops the command and thread columns, which then get no space or
/// separator at all.
fn process_columns(area: Rect, wide: bool) -> Vec<Rect> {
    let columns = [
        (Constraint::Length(5), true),
        (Constraint::Length(if wide { 16 } else { 0 }), wide),
        (Constraint::Fill(1), wide),
        (Constraint::Length(8), wide),
        (Constraint::Length(5), true),
        (Constraint::Length(5), true),
        (Constraint::Length(11), true),
    ];
    // Program fills the space when the command column is hidden.
    let constraints = columns
        .iter()
        .enumerate()
        .filter_map(|(index, (constraint, shown))| match (index, shown) {
            (1, false) => Some(Constraint::Fill(1)),
            (_, true) => Some(*constraint),
            (_, false) => None,
        });
    let areas = Layout::horizontal(constraints).spacing(1).split(area);
    let mut areas = areas.iter();
    columns
        .iter()
        .enumerate()
        .map(|(index, (_, shown))| {
            if *shown || index == 1 {
                areas.next().copied().unwrap_or_default()
            } else {
                Rect::default()
            }
        })
        .collect()
}

/// A user name cut to five characters, like btop's `josh+`.
fn short_user(user: &str) -> String {
    if user.chars().count() <= 5 {
        user.to_string()
    } else {
        let start: String = user.chars().take(4).collect();
        format!("{start}+")
    }
}

/// Splits `area` in two with `layout`, or gives all of it to whichever side is shown.
fn split(layout: &Layout, area: Rect, first: bool, second: bool) -> [Rect; 2] {
    match (first, second) {
        (true, true) => layout.areas(area),
        (true, false) => [area, Rect::default()],
        (false, true) => [Rect::default(), area],
        (false, false) => [Rect::default(); 2],
    }
}

/// Teal for low load, through gold, to red for full load.
fn heat(ratio: f32) -> Color {
    let ratio = ratio.clamp(0.0, 1.0);
    if ratio < 0.5 {
        mix(GOLD, TEAL, ratio * 2.0)
    } else {
        mix(RED, GOLD, (ratio - 0.5) * 2.0)
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

fn bytes(value: u64) -> String {
    let units = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut size = value as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < units.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{value} B")
    } else {
        format!("{size:.1} {}", units[unit])
    }
}

/// A rate in bits per second, like btop's `31.9 Kibps`.
fn bits(bytes_per_second: f64) -> String {
    let units = ["bps", "Kibps", "Mibps", "Gibps"];
    let mut value = bytes_per_second * 8.0;
    let mut unit = 0;
    while value >= 1024.0 && unit < units.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", units[unit])
}

/// A size in at most five characters, like btop's disk sizes: `3.6T`, `500M`.
fn compact_bytes(value: u64) -> String {
    let units = ["B", "K", "M", "G", "T", "P"];
    let mut size = value as f64;
    let mut unit = 0;
    while size >= 1000.0 && unit < units.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if size < 10.0 && unit > 0 {
        format!("{size:.1}{}", units[unit])
    } else {
        format!("{size:.0}{}", units[unit])
    }
}

/// A compact size like btop's memory column: `17M`, `1.4G`.
fn short_bytes(value: u64) -> String {
    let mib = value as f64 / (1024.0 * 1024.0);
    if mib >= 1024.0 {
        format!("{:.1}G", mib / 1024.0)
    } else {
        format!("{mib:.0}M")
    }
}

fn duration(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = seconds % 86_400 / 3_600;
    let minutes = seconds % 3_600 / 60;
    if days > 0 {
        format!("{days}d {hours:02}:{minutes:02}")
    } else {
        format!("{hours:02}:{minutes:02}")
    }
}

/// Lightens every other cell in a checkerboard, to show that the lines sit on cell edges.
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
            cell.bg = mix(Color::Rgb(255, 255, 255), base, 0.1);
        }
    }
}
