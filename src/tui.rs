use anyhow::Result;
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, HighlightSpacing, Paragraph, Row, Table, TableState, Wrap},
};
use std::io::stdout;
use sysinfo::{CpuRefreshKind, Pid, ProcessRefreshKind, RefreshKind, System};

// Use crate:: instead of winlog::
use crate::record::{EventLevel, EventRecord};
use crate::win_api::EventLogQuery;

#[derive(PartialEq, Eq)]
pub enum DetailViewMode {
    Parameters,
    RawXml,
}

pub struct App {
    pub records: Vec<EventRecord>,
    pub table_state: TableState,
    pub channel: String,
    pub detail_expanded: bool,
    pub detail_mode: DetailViewMode,
    pub active_filter: String,
    pub sys: System,
    pub pid: Pid,
}

impl App {
    pub fn new(channel: String, records: Vec<EventRecord>) -> Self {
        let mut table_state = TableState::default();
        if !records.is_empty() {
            table_state.select(Some(0));
        }

        let pid = Pid::from_u32(std::process::id());
        let mut sys = System::new_with_specifics(
            RefreshKind::nothing()
                .with_processes(ProcessRefreshKind::nothing().with_cpu().with_memory())
                .with_cpu(CpuRefreshKind::nothing()),
        );

        sys.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::Some(&[pid]),
            true,
            ProcessRefreshKind::nothing().with_cpu().with_memory(),
        );

        Self {
            records,
            table_state,
            channel,
            detail_expanded: true,
            detail_mode: DetailViewMode::Parameters,
            active_filter: "None".to_string(),
            sys,
            pid,
        }
    }

    pub fn refresh_telemetry(&mut self) {
        self.sys.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::Some(&[self.pid]),
            true,
            ProcessRefreshKind::nothing().with_cpu().with_memory(),
        );
    }

    pub fn get_resource_usage(&self) -> (f32, u64) {
        if let Some(proc) = self.sys.process(self.pid) {
            let cpu = proc.cpu_usage();
            let mem_mb = proc.memory() / (1024 * 1024);
            (cpu, mem_mb)
        } else {
            (0.0, 0)
        }
    }

    pub fn selected_record(&self) -> Option<&EventRecord> {
        self.table_state
            .selected()
            .and_then(|i| self.records.get(i))
    }

    pub fn next(&mut self) {
        if self.records.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.records.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.records.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.records.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn toggle_detail(&mut self) {
        self.detail_expanded = !self.detail_expanded;
    }

    pub fn toggle_view_mode(&mut self) {
        self.detail_mode = match self.detail_mode {
            DetailViewMode::Parameters => DetailViewMode::RawXml,
            DetailViewMode::RawXml => DetailViewMode::Parameters,
        };
    }
}

pub fn run_tui(channel: &str, limit: u32) -> Result<()> {
    let query = EventLogQuery::open_path_or_channel(channel)?;
    let raw_events = query.next_events(limit)?;

    let mut records = Vec::new();
    for handle in raw_events {
        if let Ok(xml) = handle.to_xml() {
            if let Ok(record) = EventRecord::from_xml(&xml) {
                records.push(record);
            }
        }
    }

    let mut app = App::new(channel.to_string(), records);

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let res = main_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res
}

fn main_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        app.refresh_telemetry();
        terminal.draw(|f| ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            return Ok(());
                        }
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                        KeyCode::Char(' ') => app.toggle_detail(),
                        KeyCode::Tab => app.toggle_view_mode(),
                        _ => {}
                    }
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let outer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    let content_constraints = if app.detail_expanded {
        vec![Constraint::Percentage(60), Constraint::Percentage(40)]
    } else {
        vec![Constraint::Min(0)]
    };

    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(content_constraints)
        .split(outer_chunks[0]);

    // 1. Render Top Table
    let rows: Vec<Row> = app
        .records
        .iter()
        .map(|r| {
            let timestamp = r
                .timestamp
                .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "N/A".to_string());

            let (level_str, level_color) = match r.level {
                EventLevel::Error | EventLevel::Critical => ("Error", Color::Red),
                EventLevel::Warning => ("Warning", Color::Yellow),
                EventLevel::Information => ("Info", Color::Green),
                _ => ("Unknown", Color::DarkGray),
            };

            let payload_summary = r
                .payload
                .first()
                .map(|(k, v)| format!("{}: {}", k, v))
                .unwrap_or_default();

            Row::new(vec![
                Cell::from(timestamp),
                Cell::from(level_str).style(Style::default().fg(level_color)),
                Cell::from(r.event_id.to_string()),
                Cell::from(r.provider.clone()),
                Cell::from(payload_summary),
            ])
        })
        .collect();

    let header = Row::new(vec![
        "Timestamp",
        "Level",
        "ID",
        "Source",
        "Message Summary",
    ])
    .style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .bottom_margin(1);

    let table = Table::new(
        rows,
        [
            Constraint::Length(20),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(25),
            Constraint::Min(30),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" winlog TUI - Channel: {} ", app.channel)),
    )
    .row_highlight_style(
        Style::default()
            .bg(Color::Rgb(40, 44, 52))
            .add_modifier(Modifier::BOLD),
    )
    .highlight_spacing(HighlightSpacing::Always);

    f.render_stateful_widget(table, content_chunks[0], &mut app.table_state);

    // 2. Render Collapsible Bottom Detail Pane
    if app.detail_expanded {
        if let Some(record) = app.selected_record() {
            let title = match app.detail_mode {
                DetailViewMode::Parameters => format!(
                    " Event Details: ID {} [{}] (Press Tab for Raw XML) ",
                    record.event_id, record.provider
                ),
                DetailViewMode::RawXml => format!(
                    " Raw XML View: Event ID {} (Press Tab for Key-Values) ",
                    record.event_id
                ),
            };

            let detail_block = Block::default().borders(Borders::ALL).title(title);

            let content: Vec<Line> = match app.detail_mode {
                DetailViewMode::Parameters => {
                    let mut lines = vec![
                        Line::from(vec![
                            Span::styled("Computer: ", Style::default().fg(Color::DarkGray)),
                            Span::raw(&record.computer),
                            Span::styled(" | Channel: ", Style::default().fg(Color::DarkGray)),
                            Span::raw(&record.channel),
                        ]),
                        Line::from(vec![
                            Span::styled("Process ID: ", Style::default().fg(Color::DarkGray)),
                            Span::raw(
                                record
                                    .process_id
                                    .map(|p| p.to_string())
                                    .unwrap_or_else(|| "N/A".to_string()),
                            ),
                            Span::styled(" | Thread ID: ", Style::default().fg(Color::DarkGray)),
                            Span::raw(
                                record
                                    .thread_id
                                    .map(|t| t.to_string())
                                    .unwrap_or_else(|| "N/A".to_string()),
                            ),
                        ]),
                        Line::from(""),
                        Line::from(Span::styled(
                            "--- Parameters ---",
                            Style::default().fg(Color::Yellow),
                        )),
                    ];

                    if record.payload.is_empty() {
                        lines.push(Line::from(Span::styled(
                            "No event payload parameters available.",
                            Style::default().fg(Color::DarkGray),
                        )));
                    } else {
                        for (key, val) in &record.payload {
                            lines.push(Line::from(vec![
                                Span::styled(
                                    format!("  {}: ", key),
                                    Style::default().fg(Color::Cyan),
                                ),
                                Span::raw(val),
                            ]));
                        }
                    }
                    lines
                }
                DetailViewMode::RawXml => record.raw_xml.lines().map(Line::from).collect(),
            };

            let paragraph = Paragraph::new(content)
                .block(detail_block)
                .wrap(Wrap { trim: false });
            f.render_widget(paragraph, content_chunks[1]);
        }
    }

    // 3. Render Status Bar
    let (cpu, mem) = app.get_resource_usage();
    let status_line = Line::from(vec![
        Span::styled(
            " CH: ",
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {} ", app.channel),
            Style::default().bg(Color::Rgb(50, 50, 50)).fg(Color::White),
        ),
        Span::raw(" "),
        Span::styled(
            " FILTER: ",
            Style::default()
                .bg(Color::Magenta)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {} ", app.active_filter),
            Style::default().bg(Color::Rgb(50, 50, 50)).fg(Color::White),
        ),
        Span::raw(" "),
        Span::styled(
            " TOTAL: ",
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {} ", app.records.len()),
            Style::default().bg(Color::Rgb(50, 50, 50)).fg(Color::White),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("CPU: {:.1}% | MEM: {}MB", cpu, mem),
            Style::default().fg(Color::Green),
        ),
        Span::raw(" | "),
        Span::styled(
            "j/k:Nav Space:Pane Tab:XML q:Quit",
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let status_bar = Paragraph::new(status_line);
    f.render_widget(status_bar, outer_chunks[1]);
}
