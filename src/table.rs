use std::io;
use std::sync::mpsc;
use std::thread;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Cell, Paragraph, Row, Table},
    Frame, Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind,
            DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use crate::classifier::{ClassifiedLine, LogLevel, value_to_string};
use crate::config::Config;

// --- Column enum ---

#[derive(Debug, Clone, PartialEq)]
pub enum Column {
    Time,
    Level,
    Message,
    Service,
    Label,
    TraceId,
}

impl Column {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "time"     => Some(Column::Time),
            "level"    => Some(Column::Level),
            "message"  => Some(Column::Message),
            "service"  => Some(Column::Service),
            "label"    => Some(Column::Label),
            "trace_id" => Some(Column::TraceId),
            _          => None,
        }
    }

    pub fn header(&self) -> &str {
        match self {
            Column::Time     => "TIME",
            Column::Level    => "LEVEL",
            Column::Message  => "MESSAGE",
            Column::Service  => "SERVICE",
            Column::Label    => "LABEL",
            Column::TraceId  => "TRACE-ID",
        }
    }

    /// Fixed minimum width for this column. Message has no fixed width (gets remainder).
    pub fn fixed_width(&self) -> Option<u16> {
        match self {
            Column::Time     => Some(8),
            Column::Level    => Some(7),
            Column::Message  => None,  // takes remainder
            Column::Service  => Some(12),
            Column::Label    => Some(10),
            Column::TraceId  => Some(14),
        }
    }
}

// --- ColWidths ---

pub struct ColWidths {
    pub widths: Vec<(Column, u16)>,
}

impl ColWidths {
    /// Compute column widths for a given terminal width.
    /// Fixed columns use their minimum. Message column takes the remainder.
    /// A separator char is counted between each column.
    pub fn compute(columns: &[Column], terminal_width: u16) -> Self {
        let separator_total = columns.len().saturating_sub(1) as u16;
        let fixed_total: u16 = columns.iter()
            .filter_map(|c| c.fixed_width())
            .sum();
        let msg_width = terminal_width
            .saturating_sub(fixed_total)
            .saturating_sub(separator_total)
            .max(20);

        let widths = columns.iter().map(|c| {
            let w = c.fixed_width().unwrap_or(msg_width);
            (c.clone(), w)
        }).collect();

        ColWidths { widths }
    }

    pub fn width_of(&self, col: &Column) -> u16 {
        self.widths.iter()
            .find(|(c, _)| c == col)
            .map(|(_, w)| *w)
            .unwrap_or(0)
    }
}

// --- App state ---

pub struct App {
    pub rows: Vec<ClassifiedLine>,
    pub scroll_offset: usize,
    pub selected: usize,
    pub expanded: Option<usize>,
    pub paused: bool,
    pub new_count: usize,
    pub col_widths: ColWidths,
    pub columns: Vec<Column>,
    pub show_extras_in_detail: bool,
    pub visible_height: u16,
}

impl App {
    pub fn new(config: &Config, show_extras: bool, terminal_width: u16, visible_height: u16) -> Self {
        let columns: Vec<Column> = config.table.columns.iter()
            .filter_map(|s| Column::from_str(s))
            .collect();
        let col_widths = ColWidths::compute(&columns, terminal_width);
        App {
            rows: Vec::new(),
            scroll_offset: 0,
            selected: 0,
            expanded: None,
            paused: false,
            new_count: 0,
            col_widths,
            columns,
            show_extras_in_detail: show_extras,
            visible_height,
        }
    }

    pub fn push_row(&mut self, row: ClassifiedLine) {
        self.rows.push(row);
        if !self.paused {
            self.selected = self.rows.len() - 1;
            self.adjust_scroll_offset();
        } else {
            self.new_count += 1;
        }
    }

    pub fn scroll_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.paused = true;
            if self.selected < self.scroll_offset {
                self.scroll_offset = self.selected;
            }
        }
    }

    pub fn scroll_down(&mut self) {
        if self.selected + 1 < self.rows.len() {
            self.selected += 1;
            self.adjust_scroll_offset();
        }
    }

    pub fn jump_to_end(&mut self) {
        self.paused = false;
        self.new_count = 0;
        if !self.rows.is_empty() {
            self.selected = self.rows.len() - 1;
            self.adjust_scroll_offset();
        }
    }

    pub fn toggle_expand(&mut self) {
        match self.expanded {
            Some(i) if i == self.selected => self.expanded = None,
            _ => self.expanded = Some(self.selected),
        }
    }

    pub fn toggle_pause(&mut self) {
        if self.paused {
            self.jump_to_end();
        } else {
            self.paused = true;
        }
    }

    fn adjust_scroll_offset(&mut self) {
        if self.selected >= self.scroll_offset + self.visible_height as usize {
            self.scroll_offset = self.selected + 1 - self.visible_height as usize;
        }
    }
}

/// Returns ratatui fg+bg Style for a log level badge.
fn level_style(lvl: &LogLevel) -> Style {
    match lvl {
        LogLevel::Error => Style::default().fg(Color::LightRed).bg(Color::Red),
        LogLevel::Warn  => Style::default().fg(Color::Yellow).bg(Color::Rgb(100, 70, 0)),
        LogLevel::Info  => Style::default().fg(Color::LightGreen).bg(Color::Green),
        LogLevel::Debug => Style::default().fg(Color::LightBlue).bg(Color::Blue),
        LogLevel::Trace => Style::default().fg(Color::DarkGray).bg(Color::Black),
        LogLevel::Unknown(_) => Style::default(),
    }
}

fn level_text(lvl: &LogLevel) -> &'static str {
    match lvl {
        LogLevel::Error   => " ERR ",
        LogLevel::Warn    => " WRN ",
        LogLevel::Info    => " INF ",
        LogLevel::Debug   => " DBG ",
        LogLevel::Trace   => " TRC ",
        LogLevel::Unknown(_) => " ??? ",
    }
}

/// Extract value for a given column from a ClassifiedLine.
fn cell_value(col: &Column, row: &ClassifiedLine) -> String {
    match col {
        Column::Time => row.timestamp.as_deref()
            .map(crate::renderer::shorten_timestamp)
            .unwrap_or_default(),
        Column::Level => String::new(), // rendered specially as a styled cell
        Column::Message => row.message.clone().unwrap_or_default(),
        Column::Service => extras_get(row, "service"),
        Column::Label   => extras_get(row, "label"),
        Column::TraceId => row.trace_id.clone()
            .unwrap_or_else(|| extras_get(row, "trace_id")),
    }
}

fn extras_get(row: &ClassifiedLine, key: &str) -> String {
    row.extras.iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| value_to_string(v))
        .unwrap_or_else(|| "—".to_string())
}

/// Truncate a string to `max_chars`, appending "…" if cut.
fn truncate(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_chars {
        s.to_string()
    } else {
        let cut: String = chars[..max_chars.saturating_sub(1)].iter().collect();
        format!("{}…", cut)
    }
}

fn col_constraints(app: &App) -> Vec<Constraint> {
    app.col_widths.widths.iter()
        .map(|(_, w)| Constraint::Length(*w))
        .collect()
}

pub fn render_ui(f: &mut Frame, app: &App) {
    let size = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Min(0),    // rows
            Constraint::Length(1), // status bar
        ])
        .split(size);

    render_header(f, app, chunks[0]);
    render_rows(f, app, chunks[1]);
    render_status(f, app, chunks[2]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let header_style = Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD);
    let cells: Vec<Cell> = app.columns.iter().map(|col| {
        Cell::from(col.header()).style(header_style)
    }).collect();
    let header = Row::new(cells).height(1);
    let widths = col_constraints(app);
    let table = Table::new(vec![header], widths)
        .block(Block::default());
    f.render_widget(table, area);
}

fn render_rows(f: &mut Frame, app: &App, area: Rect) {
    let widths = col_constraints(app);
    let visible_end = (app.scroll_offset + area.height as usize).min(app.rows.len());
    let visible_rows = &app.rows[app.scroll_offset..visible_end];

    let rows: Vec<Row> = visible_rows.iter().enumerate().map(|(rel_idx, row)| {
        let abs_idx = app.scroll_offset + rel_idx;
        let is_selected = abs_idx == app.selected;
        let is_expanded = app.expanded == Some(abs_idx);

        let base_style = if is_selected {
            Style::default().bg(Color::Rgb(30, 41, 59))
        } else {
            Style::default()
        };

        let cells: Vec<Cell> = app.columns.iter().map(|col| {
            if *col == Column::Level {
                let (text, style) = match &row.level {
                    Some(lvl) => (level_text(lvl), level_style(lvl)),
                    None => (" ??? ", Style::default()),
                };
                Cell::from(text).style(style)
            } else {
                let max_w = app.col_widths.width_of(col) as usize;
                let raw = cell_value(col, row);
                let text = if *col == Column::Message && !is_expanded {
                    truncate(&raw, max_w)
                } else {
                    raw
                };
                let cell_style = if *col == Column::Message {
                    match &row.level {
                        Some(LogLevel::Error) => base_style.fg(Color::LightRed),
                        _ => base_style.fg(Color::White),
                    }
                } else {
                    base_style.fg(Color::DarkGray)
                };
                Cell::from(text).style(cell_style)
            }
        }).collect();

        let mut table_row = Row::new(cells).style(base_style);

        if is_expanded {
            let detail = build_detail_lines(row, app.show_extras_in_detail);
            let detail_height = 1 + detail.len() as u16;
            table_row = table_row.height(detail_height);
        }

        table_row
    }).collect();

    let table = Table::new(rows, widths);
    f.render_widget(table, area);
}

fn build_detail_lines(row: &ClassifiedLine, show_extras: bool) -> Vec<String> {
    let mut lines = Vec::new();

    if let Some(ref msg) = row.message {
        lines.push(format!("  msg: {}", msg));
    }
    if let Some(ref tid) = row.trace_id {
        lines.push(format!("  trace_id: {}", tid));
    }
    if let Some(ref c) = row.caller {
        lines.push(format!("  caller: {}", c));
    }
    if show_extras {
        for (k, v) in &row.extras {
            lines.push(format!("  {}: {}", k, value_to_string(v)));
        }
    }
    for cont in &row.continuation_lines {
        lines.push(format!("    {}", cont));
    }
    lines
}

fn render_status(f: &mut Frame, app: &App, area: Rect) {
    let live_indicator = if app.paused {
        Span::styled("⏸ PAUSED", Style::default().fg(Color::Yellow))
    } else {
        Span::styled("● LIVE", Style::default().fg(Color::LightGreen))
    };

    let new_notice = if app.new_count > 0 {
        format!(" ↓ {} new  ", app.new_count)
    } else {
        String::new()
    };

    let row_info = format!("row {}/{}", app.selected + 1, app.rows.len());

    let left = format!(
        "↑↓/wheel scroll · Enter expand · Space pause · End latest · q quit{}",
        new_notice
    );

    let status = Line::from(vec![
        Span::styled(left, Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled(row_info, Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        live_indicator,
    ]);

    f.render_widget(Paragraph::new(status), area);
}

pub enum AppEvent {
    LogLine(ClassifiedLine),
    RawLine(String),
    Eof,
    Term(Event),
}

/// Handle a terminal event. Returns false if the app should quit.
pub fn handle_event(app: &mut App, event: Event) -> bool {
    match event {
        Event::Key(KeyEvent { code, modifiers, .. }) => match code {
            KeyCode::Char('q') | KeyCode::Char('Q') => return false,
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => return false,
            KeyCode::Up   => app.scroll_up(),
            KeyCode::Down => app.scroll_down(),
            KeyCode::End  => app.jump_to_end(),
            KeyCode::Char('G') => app.jump_to_end(),
            KeyCode::Enter => app.toggle_expand(),
            KeyCode::Char(' ') => app.toggle_pause(),
            _ => {}
        },
        Event::Mouse(MouseEvent { kind, .. }) => match kind {
            MouseEventKind::ScrollUp   => app.scroll_up(),
            MouseEventKind::ScrollDown => app.scroll_down(),
            _ => {}
        },
        _ => {}
    }
    true
}

pub fn run_table_mode(config: &Config, show_extras: bool) -> io::Result<()> {
    use crate::reader::LineReader;
    use crate::parser::{parse_line, ParseResult};
    use crate::classifier::classify;

    // --- Terminal setup ---
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let size = terminal.size()?;
    // Reserve 2 rows: 1 header + 1 status bar
    let visible_height = size.height.saturating_sub(2);

    let mut app = App::new(config, show_extras, size.width, visible_height);

    let (tx, rx) = mpsc::channel::<AppEvent>();

    // Stdin reader thread
    let tx_log = tx.clone();
    let cfg = config.clone();
    thread::spawn(move || {
        let stdin = io::stdin();
        let reader = LineReader::new(stdin.lock(), &cfg.multiline);
        for logical_line in reader {
            let ev = match parse_line(&logical_line.main, logical_line.continuations) {
                ParseResult::Json(parsed) => {
                    AppEvent::LogLine(classify(parsed, &cfg))
                }
                ParseResult::Raw { line, .. } => AppEvent::RawLine(line),
            };
            if tx_log.send(ev).is_err() {
                break;
            }
        }
        let _ = tx_log.send(AppEvent::Eof);
    });

    // Terminal event thread
    let tx_term = tx;
    thread::spawn(move || {
        loop {
            match event::poll(std::time::Duration::from_millis(50)) {
                Ok(true) => {
                    if let Ok(ev) = event::read() {
                        if tx_term.send(AppEvent::Term(ev)).is_err() {
                            break;
                        }
                    }
                }
                Ok(false) => {}
                Err(_) => break,
            }
        }
    });

    // Main render loop
    let mut running = true;
    while running {
        terminal.draw(|f| render_ui(f, &app))?;

        match rx.recv_timeout(std::time::Duration::from_millis(16)) {
            Ok(AppEvent::LogLine(line)) => app.push_row(line),
            Ok(AppEvent::RawLine(raw)) => {
                app.push_row(ClassifiedLine {
                    level: None,
                    timestamp: None,
                    message: Some(raw),
                    trace_id: None,
                    caller: None,
                    extras: vec![],
                    continuation_lines: vec![],
                });
            }
            Ok(AppEvent::Eof) => { /* stdin closed; keep showing until user quits */ }
            Ok(AppEvent::Term(ev)) => {
                if !handle_event(&mut app, ev) {
                    running = false;
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => running = false,
        }
    }

    // --- Terminal teardown ---
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_from_str_known_values() {
        assert_eq!(Column::from_str("time"), Some(Column::Time));
        assert_eq!(Column::from_str("level"), Some(Column::Level));
        assert_eq!(Column::from_str("message"), Some(Column::Message));
        assert_eq!(Column::from_str("service"), Some(Column::Service));
        assert_eq!(Column::from_str("label"), Some(Column::Label));
        assert_eq!(Column::from_str("trace_id"), Some(Column::TraceId));
        assert_eq!(Column::from_str("unknown"), None);
    }

    #[test]
    fn col_widths_message_gets_remainder() {
        let columns = vec![
            Column::Time,
            Column::Level,
            Column::Message,
            Column::Service,
        ];
        // time=8, level=7, service=12, separators=4 → fixed=31, message=100-31=69
        let cw = ColWidths::compute(&columns, 100);
        let msg_width = cw.width_of(&Column::Message);
        assert!(msg_width >= 20, "message column got {}", msg_width);
        let total: u16 = cw.widths.iter().map(|(_, w)| w).sum();
        assert!(total <= 100);
    }

    #[test]
    fn app_push_row_auto_scrolls_when_not_paused() {
        let config = Config::default();
        let mut app = App::new(&config, false, 200, 20);
        for i in 0..5 {
            app.push_row(make_row(format!("msg {}", i)));
        }
        assert_eq!(app.selected, 4);
        assert!(!app.paused);
        assert_eq!(app.new_count, 0);
    }

    #[test]
    fn app_push_row_buffers_when_paused() {
        let config = Config::default();
        let mut app = App::new(&config, false, 200, 20);
        app.paused = true;
        app.push_row(make_row("hello".into()));
        app.push_row(make_row("world".into()));
        assert_eq!(app.new_count, 2);
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn app_jump_to_end_clears_new_count() {
        let config = Config::default();
        let mut app = App::new(&config, false, 200, 20);
        app.paused = true;
        app.push_row(make_row("a".into()));
        app.push_row(make_row("b".into()));
        assert_eq!(app.new_count, 2);
        app.jump_to_end();
        assert_eq!(app.new_count, 0);
        assert!(!app.paused);
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn app_scroll_up_sets_paused() {
        let config = Config::default();
        let mut app = App::new(&config, false, 200, 20);
        for i in 0..5 {
            app.push_row(make_row(format!("msg {}", i)));
        }
        assert!(!app.paused);
        app.scroll_up();
        assert!(app.paused);
        assert_eq!(app.selected, 3);
    }

    #[test]
    fn app_toggle_expand_sets_and_clears() {
        let config = Config::default();
        let mut app = App::new(&config, false, 200, 20);
        app.push_row(make_row("hello".into()));
        app.toggle_expand();
        assert_eq!(app.expanded, Some(0));
        app.toggle_expand();
        assert_eq!(app.expanded, None);
    }

    fn make_row(msg: String) -> ClassifiedLine {
        ClassifiedLine {
            level: None,
            timestamp: None,
            message: Some(msg),
            trace_id: None,
            caller: None,
            extras: vec![],
            continuation_lines: vec![],
        }
    }
}
