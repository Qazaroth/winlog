use anyhow::Result;
use arboard::Clipboard;
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use nucleo_matcher::{
    Config, Matcher, Utf32Str,
    pattern::{Atom, AtomKind, CaseMatching, Normalization},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, HighlightSpacing, Paragraph, Row, Table, TableState, Wrap},
};
use regex::RegexBuilder;
use std::io::stdout;
use std::time::Instant;
use sysinfo::{CpuRefreshKind, Pid, ProcessRefreshKind, RefreshKind, System};

use crate::record::{EventLevel, EventRecord};
use crate::win_api::EventLogQuery;

#[derive(PartialEq, Eq)]
pub enum DetailViewMode {
    Parameters,
    RawXml,
}

#[derive(PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Search,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum SearchMode {
    Fuzzy,
    Regex,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ActiveLevelFilter {
    All,
    Error,
    Warning,
    Information,
}

impl ActiveLevelFilter {
    pub fn label(&self) -> &'static str {
        match self {
            ActiveLevelFilter::All => "ALL",
            ActiveLevelFilter::Error => "ERROR",
            ActiveLevelFilter::Warning => "WARN",
            ActiveLevelFilter::Information => "INFO",
        }
    }
}

pub struct App {
    pub all_records: Vec<EventRecord>,
    pub filtered_indices: Vec<usize>,
    pub table_state: TableState,
    pub channel: String,
    pub detail_expanded: bool,
    pub detail_mode: DetailViewMode,
    pub input_mode: InputMode,
    pub search_mode: SearchMode,
    pub search_query: String,
    pub regex_error: Option<String>,
    pub level_filter: ActiveLevelFilter,
    pub sys: System,
    pub pid: Pid,
    pub status_message: Option<(String, Instant)>,
    pub clipboard: Option<Clipboard>,
    pub matcher: Matcher,
}

impl App {
    pub fn new(channel: String, records: Vec<EventRecord>) -> Self {
        let filtered_indices = (0..records.len()).collect();
        let mut table_state = TableState::default();
        if !records.is_empty() {
            table_state.select(Some(0));
        }

        let pid = Pid::from_u32(std::process::id());
        let mut sys = System::new_with_specifics(
            RefreshKind::nothing()
                .with_processes(ProcessRefreshKind::everything())
                .with_cpu(CpuRefreshKind::everything()), // <-- Enable CPU tracking
        );

        sys.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::Some(&[pid]),
            true,
            ProcessRefreshKind::nothing().with_cpu().with_memory(),
        );

        let clipboard = Clipboard::new().ok();

        Self {
            all_records: records,
            filtered_indices,
            table_state,
            channel,
            detail_expanded: true,
            detail_mode: DetailViewMode::Parameters,
            input_mode: InputMode::Normal,
            search_mode: SearchMode::Fuzzy,
            search_query: String::new(),
            regex_error: None,
            level_filter: ActiveLevelFilter::All,
            sys,
            pid,
            status_message: None,
            clipboard,
            matcher: Matcher::new(Config::DEFAULT),
        }
    }

    pub fn set_status(&mut self, msg: String) {
        self.status_message = Some((msg, Instant::now()));
    }

    pub fn visible_count(&self) -> usize {
        self.filtered_indices.len()
    }

    pub fn selected_record(&self) -> Option<&EventRecord> {
        self.table_state
            .selected()
            .and_then(|idx| self.filtered_indices.get(idx))
            .and_then(|&real_idx| self.all_records.get(real_idx))
    }

    pub fn toggle_search_mode(&mut self) {
        self.search_mode = match self.search_mode {
            SearchMode::Fuzzy => SearchMode::Regex,
            SearchMode::Regex => SearchMode::Fuzzy,
        };
        self.apply_filters();
    }

    pub fn apply_filters(&mut self) {
        let query_trimmed = self.search_query.trim();

        // Check search strategy
        match self.search_mode {
            SearchMode::Regex => {
                if query_trimmed.is_empty() {
                    self.regex_error = None;
                    self.filter_by_level_only();
                    return;
                }

                // Live regex parsing & compilation check
                let compiled_re = match RegexBuilder::new(query_trimmed)
                    .case_insensitive(true)
                    .build()
                {
                    Ok(re) => {
                        self.regex_error = None;
                        re
                    }
                    Err(err) => {
                        // Store friendly error description to highlight UI
                        let clean_err = err
                            .to_string()
                            .lines()
                            .next()
                            .unwrap_or("Invalid regex")
                            .to_string();
                        self.regex_error = Some(clean_err);
                        return; // Keep existing filtered indices on error
                    }
                };

                let mut matched_indices = Vec::new();
                for (idx, record) in self.all_records.iter().enumerate() {
                    if !self.matches_level(record) {
                        continue;
                    }

                    let payload_text = record
                        .payload
                        .iter()
                        .map(|(k, v)| format!("{} {}", k, v))
                        .collect::<Vec<_>>()
                        .join(" ");

                    let record_text = format!(
                        "{} {} {} {}",
                        record.event_id, record.provider, record.computer, payload_text
                    );

                    if compiled_re.is_match(&record_text) {
                        matched_indices.push(idx);
                    }
                }

                self.filtered_indices = matched_indices;
            }
            SearchMode::Fuzzy => {
                self.regex_error = None;

                let atom = if !query_trimmed.is_empty() {
                    Some(Atom::new(
                        query_trimmed,
                        CaseMatching::Ignore,
                        Normalization::Smart,
                        AtomKind::Fuzzy,
                        false,
                    ))
                } else {
                    None
                };

                let mut scored_indices: Vec<(usize, u32)> =
                    Vec::with_capacity(self.all_records.len());

                for (idx, record) in self.all_records.iter().enumerate() {
                    if !self.matches_level(record) {
                        continue;
                    }

                    if let Some(ref atom_pattern) = atom {
                        let mut max_score: u32 = 0;
                        let mut buf = Vec::new();

                        let payload_text = record
                            .payload
                            .iter()
                            .map(|(k, v)| format!("{} {}", k, v))
                            .collect::<Vec<_>>()
                            .join(" ");

                        let searchable_targets =
                            [&record.provider, &record.computer, &payload_text];

                        for target in searchable_targets {
                            let utf32_target = Utf32Str::new(target, &mut buf);
                            if let Some(score) = atom_pattern.score(utf32_target, &mut self.matcher)
                            {
                                let score_val = score as u32;
                                if score_val > max_score {
                                    max_score = score_val;
                                }
                            }
                        }

                        let event_id_str = record.event_id.to_string();
                        let utf32_id = Utf32Str::new(&event_id_str, &mut buf);
                        if let Some(score) = atom_pattern.score(utf32_id, &mut self.matcher) {
                            let score_val = score as u32 + 50;
                            if score_val > max_score {
                                max_score = score_val;
                            }
                        }

                        if max_score > 0 {
                            scored_indices.push((idx, max_score));
                        }
                    } else {
                        scored_indices.push((idx, 0));
                    }
                }

                if atom.is_some() {
                    scored_indices.sort_by(|a, b| b.1.cmp(&a.1));
                }

                self.filtered_indices = scored_indices.into_iter().map(|(idx, _)| idx).collect();
            }
        }

        if self.visible_count() > 0 {
            self.table_state.select(Some(0));
        } else {
            self.table_state.select(None);
        }
    }

    fn matches_level(&self, record: &EventRecord) -> bool {
        match self.level_filter {
            ActiveLevelFilter::All => true,
            ActiveLevelFilter::Error => {
                matches!(record.level, EventLevel::Error | EventLevel::Critical)
            }
            ActiveLevelFilter::Warning => matches!(record.level, EventLevel::Warning),
            ActiveLevelFilter::Information => matches!(record.level, EventLevel::Information),
        }
    }

    fn filter_by_level_only(&mut self) {
        self.filtered_indices = self
            .all_records
            .iter()
            .enumerate()
            .filter(|(_, r)| self.matches_level(r))
            .map(|(i, _)| i)
            .collect();

        if self.visible_count() > 0 {
            self.table_state.select(Some(0));
        } else {
            self.table_state.select(None);
        }
    }

    pub fn set_level_filter(&mut self, filter: ActiveLevelFilter) {
        if self.level_filter == filter {
            self.level_filter = ActiveLevelFilter::All;
        } else {
            self.level_filter = filter;
        }
        self.apply_filters();
    }

    pub fn copy_summary_to_clipboard(&mut self) {
        let record = match self.selected_record() {
            Some(r) => r.clone(),
            None => return,
        };

        let timestamp = record
            .timestamp
            .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "N/A".to_string());

        let mut summary = format!(
            "Event ID: {}\nProvider: {}\nLevel: {:?}\nTimestamp: {}\nComputer: {}\nChannel: {}\n\nPayload:\n",
            record.event_id,
            record.provider,
            record.level,
            timestamp,
            record.computer,
            record.channel
        );

        for (k, v) in &record.payload {
            summary.push_str(&format!("  {}: {}\n", k, v));
        }

        if let Some(cb) = self.clipboard.as_mut() {
            if cb.set_text(summary).is_ok() {
                self.set_status(format!("Copied summary of Event ID {}!", record.event_id));
            } else {
                self.set_status("Failed to access system clipboard.".to_string());
            }
        } else {
            self.set_status("Clipboard unavailable.".to_string());
        }
    }

    pub fn copy_raw_xml_to_clipboard(&mut self) {
        let record = match self.selected_record() {
            Some(r) => r.clone(),
            None => return,
        };

        if let Some(cb) = self.clipboard.as_mut() {
            if cb.set_text(&record.raw_xml).is_ok() {
                self.set_status(format!("Copied Raw XML of Event ID {}!", record.event_id));
            } else {
                self.set_status("Failed to access system clipboard.".to_string());
            }
        } else {
            self.set_status("Clipboard unavailable.".to_string());
        }
    }

    pub fn next(&mut self) {
        if self.visible_count() == 0 {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.visible_count() - 1 {
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
        if self.visible_count() == 0 {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.visible_count() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn jump_first(&mut self) {
        if self.visible_count() > 0 {
            self.table_state.select(Some(0));
        }
    }

    pub fn jump_last(&mut self) {
        if self.visible_count() > 0 {
            self.table_state.select(Some(self.visible_count() - 1));
        }
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

    pub fn refresh_telemetry(&mut self) {
        // 1. Refresh global CPU stats so sysinfo can calculate deltas
        self.sys.refresh_cpu_all();

        // 2. Refresh process metrics
        self.sys.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::Some(&[self.pid]),
            true,
            ProcessRefreshKind::nothing().with_cpu().with_memory(),
        );
    }

    pub fn get_resource_usage(&self) -> (f32, u64) {
        if let Some(proc) = self.sys.process(self.pid) {
            let cpu_raw = proc.cpu_usage();
            let cpus = self.sys.cpus().len().max(1) as f32;
            let cpu_normalized = cpu_raw / cpus; // Scale across available CPU cores

            let mem_mb = proc.memory() / (1024 * 1024); // Physical RAM (RSS)
            (cpu_normalized, mem_mb)
        } else {
            (0.0, 0)
        }
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

        if event::poll(std::time::Duration::from_millis(150))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.input_mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                return Ok(());
                            }
                            KeyCode::Down | KeyCode::Char('j') => app.next(),
                            KeyCode::Up | KeyCode::Char('k') => app.previous(),
                            KeyCode::Char('g') => app.jump_first(),
                            KeyCode::Char('G') => app.jump_last(),
                            KeyCode::Char('y') => app.copy_summary_to_clipboard(),
                            KeyCode::Char('Y') => app.copy_raw_xml_to_clipboard(),
                            KeyCode::Char('/') => app.input_mode = InputMode::Search,
                            KeyCode::Char('1') => app.set_level_filter(ActiveLevelFilter::Error),
                            KeyCode::Char('2') => app.set_level_filter(ActiveLevelFilter::Warning),
                            KeyCode::Char('3') => {
                                app.set_level_filter(ActiveLevelFilter::Information)
                            }
                            KeyCode::Char('0') => app.set_level_filter(ActiveLevelFilter::All),
                            KeyCode::Char(' ') => app.toggle_detail(),
                            KeyCode::Tab => app.toggle_view_mode(),
                            _ => {}
                        },
                        InputMode::Search => match key.code {
                            KeyCode::Enter => {
                                app.input_mode = InputMode::Normal;
                            }
                            KeyCode::Esc => {
                                app.search_query.clear();
                                app.regex_error = None;
                                app.apply_filters();
                                app.input_mode = InputMode::Normal;
                            }
                            KeyCode::Tab => {
                                app.toggle_search_mode();
                            }
                            KeyCode::Char(c) => {
                                app.search_query.push(c);
                                app.apply_filters();
                            }
                            KeyCode::Backspace => {
                                app.search_query.pop();
                                app.apply_filters();
                            }
                            _ => {}
                        },
                    }
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let outer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
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

    // 1. Event Table
    let rows: Vec<Row> = app
        .filtered_indices
        .iter()
        .filter_map(|&idx| app.all_records.get(idx))
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

    // 2. Detail Pane
    if app.detail_expanded {
        if let Some(record) = app.selected_record() {
            let title = match app.detail_mode {
                DetailViewMode::Parameters => format!(
                    " Event Details: ID {} [{}] (Tab for Raw XML) ",
                    record.event_id, record.provider
                ),
                DetailViewMode::RawXml => format!(
                    " Raw XML View: Event ID {} (Tab for Key-Values) ",
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

    // 3. Two-Line Status Footer
    let (cpu, mem) = app.get_resource_usage();

    let (level_bg, level_fg) = match app.level_filter {
        ActiveLevelFilter::All => (Color::Rgb(50, 50, 50), Color::White),
        ActiveLevelFilter::Error => (Color::Red, Color::White),
        ActiveLevelFilter::Warning => (Color::Yellow, Color::Black),
        ActiveLevelFilter::Information => (Color::Green, Color::Black),
    };

    let line1 = match app.input_mode {
        InputMode::Search => {
            let (mode_badge, mode_bg) = match app.search_mode {
                SearchMode::Fuzzy => (" FUZZY SEARCH ", Color::Yellow),
                SearchMode::Regex => (" REGEX SEARCH ", Color::Cyan),
            };

            let mut spans = vec![
                Span::styled(
                    mode_badge,
                    Style::default()
                        .bg(mode_bg)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" /{:.40} ", app.search_query),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ];

            if let Some(ref err) = app.regex_error {
                spans.push(Span::styled(
                    format!(" [REGEX ERR: {}] ", err),
                    Style::default()
                        .bg(Color::Red)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ));
            } else if app.search_mode == SearchMode::Regex && !app.search_query.is_empty() {
                spans.push(Span::styled(
                    " [VALID REGEX] ",
                    Style::default()
                        .bg(Color::Green)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                ));
            }

            spans.push(Span::styled(
                " (Tab: Toggle Fuzzy/Regex, Enter: Confirm, Esc: Clear)",
                Style::default().fg(Color::DarkGray),
            ));

            Line::from(spans)
        }
        InputMode::Normal => {
            let mut spans = vec![
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
                    " LVL: ",
                    Style::default()
                        .bg(Color::Magenta)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {} ", app.level_filter.label()),
                    Style::default()
                        .bg(level_bg)
                        .fg(level_fg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
            ];

            if !app.search_query.is_empty() {
                let (mode_tag, tag_bg) = match app.search_mode {
                    SearchMode::Fuzzy => (" FZF: ", Color::Yellow),
                    SearchMode::Regex => (" REGEX: ", Color::Cyan),
                };

                spans.extend(vec![
                    Span::styled(
                        mode_tag,
                        Style::default()
                            .bg(tag_bg)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" {} ", app.search_query),
                        Style::default()
                            .bg(Color::Rgb(50, 50, 50))
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" "),
                ]);
            }

            spans.extend(vec![
                Span::styled(
                    " SHOWN: ",
                    Style::default()
                        .bg(Color::DarkGray)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {}/{} ", app.visible_count(), app.all_records.len()),
                    Style::default().bg(Color::Rgb(50, 50, 50)).fg(Color::White),
                ),
            ]);

            if let Some((msg, time)) = &app.status_message {
                if time.elapsed().as_secs() < 3 {
                    spans.extend(vec![
                        Span::raw(" "),
                        Span::styled(
                            format!(" [{}] ", msg),
                            Style::default()
                                .bg(Color::Green)
                                .fg(Color::Black)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]);
                }
            }

            spans.push(Span::styled(
                format!("  [CPU: {:.1}% | MEM: {}MB]", cpu, mem),
                Style::default().fg(Color::Green),
            ));

            Line::from(spans)
        }
    };

    let line2 = Line::from(vec![
        Span::styled(
            "NAV: ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("j/k", Style::default().fg(Color::White)),
        Span::styled(" Up/Down  ", Style::default().fg(Color::DarkGray)),
        Span::styled("g/G", Style::default().fg(Color::White)),
        Span::styled(" Top/Bot  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "COPY: ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("y", Style::default().fg(Color::Green)),
        Span::styled(" Summary  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Y", Style::default().fg(Color::Green)),
        Span::styled(" XML  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "FILTER: ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("1-3", Style::default().fg(Color::Yellow)),
        Span::styled(" Lvl ", Style::default().fg(Color::DarkGray)),
        Span::styled("/", Style::default().fg(Color::Yellow)),
        Span::styled(" Search  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "VIEW: ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Space", Style::default().fg(Color::White)),
        Span::styled(" Pane  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Tab", Style::default().fg(Color::White)),
        Span::styled(" Mode  ", Style::default().fg(Color::DarkGray)),
        Span::styled("q", Style::default().fg(Color::Red)),
        Span::styled(" Quit", Style::default().fg(Color::DarkGray)),
    ]);

    let footer = Paragraph::new(vec![line1, line2]);
    f.render_widget(footer, outer_chunks[1]);
}
