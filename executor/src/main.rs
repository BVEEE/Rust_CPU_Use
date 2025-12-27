mod selector;
mod state;
mod uia;
mod win32;

use std::io::stdout;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use config::Config;
use log::{info, warn};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Row, Table};
use ratatui::{backend::CrosstermBackend, Terminal};
use selector::parse as parse_selector;
use state::{ActionRequest, ActionResult, ActionType};
use sysinfo::{ProcessExt, System, SystemExt};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

const DEFAULT_INTERVAL_SECS: u64 = 2;
#[derive(Debug, Parser)]
#[command(name = "cpu-use", version, about = "CPU monitor and UI automation helper")]
struct Cli {
    /// Optional configuration file (TOML)
    #[arg(long, value_name = "FILE", env = "CPU_USE_CONFIG", global = true)]
    config: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Stream CPU usage metrics
    Monitor {
        /// Interval between refreshes in seconds
        #[arg(short, long, default_value_t = DEFAULT_INTERVAL_SECS)]
        interval: u64,
        /// Include per-process CPU usage
        #[arg(long, default_value_t = false)]
        per_process: bool,
        /// Only print one sample
        #[arg(long, default_value_t = false)]
        once: bool,
    },
    /// Generate CPU load for stress testing
    Stress {
        /// Number of worker threads
        #[arg(short, long, default_value_t = default_thread_count())]
        threads: usize,
        /// How long to run the stress test (seconds). If omitted, runs until interrupted.
        #[arg(short, long)]
        duration: Option<u64>,
    },
    /// Perform UI automation commands against a window
    Automate {
        /// Window handle to target. Defaults to the focused window.
        #[arg(short, long)]
        window: Option<u64>,
        /// Action to perform
        #[arg(value_enum)]
        action: ActionKind,
        /// Selector string (e.g., name="CPU" control_type="Text")
        selector: String,
        /// Value used for set-value actions
        #[arg(long)]
        value: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ActionKind {
    Monitor,
    Click,
    Invoke,
    SetValue,
}

#[derive(Debug, Default, serde::Deserialize)]
struct AppConfig {
    monitor: Option<MonitorConfig>,
}

#[derive(Debug, Default, serde::Deserialize)]
struct MonitorConfig {
    interval: Option<u64>,
    per_process: Option<bool>,
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    let config = load_config(cli.config.as_deref())?;

    match cli.command {
        Commands::Monitor {
            interval,
            per_process,
            once,
        } => {
            let monitor_config = config.monitor.unwrap_or_default();
            let interval = monitor_config.interval.unwrap_or(interval);
            let per_process = monitor_config.per_process.unwrap_or(per_process);
            run_monitor(interval, per_process, once)?;
        }
        Commands::Stress { threads, duration } => run_stress(threads, duration)?,
        Commands::Automate {
            window,
            action,
            selector,
            value,
        } => run_automation(window, action, selector, value)?,
    }

    Ok(())
}

fn load_config(path: Option<&str>) -> Result<AppConfig> {
    if let Some(path) = path {
        let cfg = Config::builder()
            .add_source(config::File::with_name(path))
            .build()
            .context("failed to load configuration file")?;
        return cfg
            .try_deserialize()
            .context("unable to deserialize configuration file");
    }

    Ok(AppConfig::default())
}

fn run_monitor(interval: u64, per_process: bool, once: bool) -> Result<()> {
    if once {
        let mut system = System::new_all();
        let sample = CpuSnapshot::gather(&mut system, per_process);
        print_snapshot(&sample);
        return Ok(());
    }

    run_monitor_tui(interval, per_process)
}

fn run_stress(threads: usize, duration: Option<u64>) -> Result<()> {
    if threads == 0 {
        anyhow::bail!("threads must be greater than zero");
    }

    let running = Arc::new(AtomicBool::new(true));
    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let flag = running.clone();
            thread::spawn(move || {
                let mut acc: u64 = 0;
                while flag.load(Ordering::Relaxed) {
                    acc = acc.wrapping_add(1);
                    if acc == u64::MAX {
                        acc = 0;
                    }
                }
                acc
            })
        })
        .collect();

    if let Some(seconds) = duration {
        thread::sleep(Duration::from_secs(seconds));
        running.store(false, Ordering::Relaxed);
    } else {
        ctrlc::set_handler({
            let running = running.clone();
            move || running.store(false, Ordering::Relaxed)
        })
        .context("failed to install Ctrl+C handler")?;
        while running.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(200));
        }
    }

    for handle in handles {
        let _ = handle.join();
    }

    info!("Stress test completed");
    Ok(())
}

fn run_automation(
    window: Option<u64>,
    action: ActionKind,
    selector: String,
    value: Option<String>,
) -> Result<()> {
    #[cfg(not(target_os = "windows"))]
    {
        anyhow::bail!(
            "UI automation is only supported on Windows. Use monitor/stress subcommands on this platform."
        );
    }

    let target_window = window.or_else(|| match win32::get_focused_window() {
        Ok(handle) => handle,
        Err(err) => {
            warn!("failed to get focused window: {err}");
            None
        }
    });

    let Some(window_handle) = target_window else {
        anyhow::bail!("No window handle provided and unable to determine focused window");
    };

    uia::initialize_com()?;
    let _guard = ComGuard;

    let selector = parse_selector(&selector)?;
    let request = ActionRequest {
        action: match action {
            ActionKind::Monitor => ActionType::Invoke,
            ActionKind::Click => ActionType::Click,
            ActionKind::Invoke => ActionType::Invoke,
            ActionKind::SetValue => ActionType::SetValue,
        },
        selector: selector.clone(),
        value,
    };
    request.validate()?;

    let found = uia::find_element(window_handle, &request.selector)?;
    match request.action {
        ActionType::Click => uia::click_element(&found)?,
        ActionType::Invoke => uia::invoke_element(&found)?,
        ActionType::SetValue => {
            let Some(ref value) = request.value else {
                anyhow::bail!("set_value requires a --value payload");
            };
            uia::set_element_value(&found, value)?
        }
    }

    let result = ActionResult::success(found.info.clone());
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

/// RAII guard for COM cleanup
struct ComGuard;

impl Drop for ComGuard {
    fn drop(&mut self) {
        uia::uninitialize_com();
    }
}

fn default_thread_count() -> usize {
    thread_count_from_system()
}

fn thread_count_from_system() -> usize {
    match std::thread::available_parallelism() {
        Ok(count) => count.get(),
        Err(_) => 4,
    }
}

#[derive(Debug, Clone)]
struct ProcessRow {
    pid: sysinfo::Pid,
    name: String,
    cpu: f32,
}

#[derive(Debug, Clone)]
struct CpuSnapshot {
    total: f32,
    freq: u64,
    per_core: Vec<(f32, u64)>,
    processes: Vec<ProcessRow>,
}

impl CpuSnapshot {
    fn gather(system: &mut System, per_process: bool) -> Self {
        system.refresh_cpu();

        if per_process {
            system.refresh_processes();
        }

        let global_cpu = system.global_cpu_info();
        let mut processes = Vec::new();

        if per_process {
            processes = top_processes(system, 8);
        }

        Self {
            total: global_cpu.cpu_usage(),
            freq: global_cpu.frequency(),
            per_core: system
                .cpus()
                .iter()
                .map(|cpu| (cpu.cpu_usage(), cpu.frequency()))
                .collect(),
            processes,
        }
    }
}

fn top_processes(system: &System, take: usize) -> Vec<ProcessRow> {
    let mut processes: Vec<_> = system.processes().values().collect();
    processes.sort_by(|a, b| {
        b.cpu_usage()
            .partial_cmp(&a.cpu_usage())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    processes
        .into_iter()
        .take(take)
        .map(|process| ProcessRow {
            pid: process.pid(),
            name: process.name().to_string(),
            cpu: process.cpu_usage(),
        })
        .collect()
}

fn print_snapshot(snapshot: &CpuSnapshot) {
    println!(
        "Total CPU: {:.2}% | Avg freq: {} MHz",
        snapshot.total, snapshot.freq
    );

    for (idx, (usage, freq)) in snapshot.per_core.iter().enumerate() {
        println!("CPU {idx}: {:.2}% @ {freq} MHz", usage);
    }

    if !snapshot.processes.is_empty() {
        println!("\nTop processes:");
        for proc in &snapshot.processes {
            println!("{:>6} {:<30} {:>6.2}%", proc.pid, proc.name, proc.cpu);
        }
    }
}

fn run_monitor_tui(interval: u64, per_process: bool) -> Result<()> {
    let mut system = System::new_all();
    let mut snapshot = CpuSnapshot::gather(&mut system, per_process);

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let _guard = TerminalGuard;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let tick_rate = Duration::from_millis(250);
    let mut last_refresh = Instant::now();
    let interval = Duration::from_secs(interval);

    loop {
        terminal.draw(|f| render_snapshot(f, &snapshot, per_process))?;

        let timeout = tick_rate.saturating_sub(last_refresh.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                    break;
                }
            }
        }

        if last_refresh.elapsed() >= interval {
            snapshot = CpuSnapshot::gather(&mut system, per_process);
            last_refresh = Instant::now();
        }
    }

    Ok(())
}

/// Ensures terminal state is restored even if the TUI exits early due to errors.
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, cursor::Show);
    }
}

fn render_snapshot(f: &mut ratatui::Frame, snapshot: &CpuSnapshot, per_process: bool) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length((snapshot.per_core.len() as u16).saturating_add(2)),
                Constraint::Min(5),
            ]
            .as_ref(),
        )
        .split(f.size());

    let global_block = Block::default().title("CPU").borders(Borders::ALL);
    let gauge = Gauge::default()
        .block(global_block)
        .gauge_style(Style::default().fg(Color::Cyan))
        .ratio((snapshot.total / 100.0).clamp(0.0, 1.0))
        .label(Line::from(vec![
            Span::raw(format!("Total: {:>5.2}%", snapshot.total)),
            Span::raw("  |  "),
            Span::raw(format!("Freq: {} MHz", snapshot.freq)),
        ]));
    f.render_widget(gauge, layout[0]);

    let core_rows: Vec<Line> = snapshot
        .per_core
        .iter()
        .enumerate()
        .map(|(idx, (usage, freq))| {
            Line::from(vec![
                Span::styled(format!("Core {idx:02}: "), Style::default().fg(Color::Yellow)),
                Span::raw(format!("{:>5.2}% @ {:>4} MHz", usage, freq)),
            ])
        })
        .collect();

    let cores_block = Block::default()
        .title("Per-core usage")
        .borders(Borders::ALL);
    let cores_area = layout[1];
    let cores_widget = ratatui::widgets::Paragraph::new(core_rows).block(cores_block);
    f.render_widget(cores_widget, cores_area);

    if per_process {
        let rows: Vec<Row> = snapshot
            .processes
            .iter()
            .map(|p| Row::new(vec![
                p.pid.to_string(),
                p.name.clone(),
                format!("{:.2}%", p.cpu),
            ]))
            .collect();

        let table = Table::new(rows)
            .header(Row::new(vec!["PID", "Process", "CPU"]))
            .block(Block::default().title("Top processes (press q to quit)").borders(Borders::ALL))
            .widths(&[Constraint::Length(8), Constraint::Percentage(60), Constraint::Length(10)]);

        f.render_widget(table, layout[2]);
    } else {
        let info = ratatui::widgets::Paragraph::new("Press q or Esc to quit")
            .block(Block::default().title("Controls").borders(Borders::ALL));
        f.render_widget(info, layout[2]);
    }
}
