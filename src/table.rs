use std::io;
use std::sync::mpsc;
use std::thread;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Cell, Paragraph, Row, Table},
    Frame, Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
            DisableMouseCapture},
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
}

impl Column {
    #[cfg(test)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "time"     => Some(Column::Time),
            "level"    => Some(Column::Level),
            "message"  => Some(Column::Message),
            _          => None,
        }
    }

    pub fn header(&self) -> &str {
        match self {
            Column::Time     => "TIME",
            Column::Level    => "LEVEL",
            Column::Message  => "MESSAGE",
        }
    }

    /// Fixed minimum width for this column. Message has no fixed width (gets remainder).
    pub fn fixed_width(&self) -> Option<u16> {
        match self {
            Column::Time     => Some(8),
            Column::Level    => Some(7),
            Column::Message  => None,  // takes remainder
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
    pub rows: Vec<TableRowData>,
    pub scroll_offset: usize,
    pub selected: usize,
    pub paused: bool,
    pub new_count: usize,
    pub col_widths: ColWidths,
    pub columns: Vec<Column>,
    pub visible_height: u16,
    pub highlight_errors: bool,
}

pub struct TableRowData {
    pub line: ClassifiedLine,
    pub detail_pairs: Vec<(String, String)>,
}

impl App {
    pub fn new(config: &Config, show_extras: bool, terminal_width: u16, visible_height: u16) -> Self {
        let _ = show_extras;
        let _ = config;
        let columns: Vec<Column> = vec![Column::Time, Column::Level, Column::Message];
        let col_widths = ColWidths::compute(&columns, terminal_width);
        App {
            rows: Vec::new(),
            scroll_offset: 0,
            selected: 0,
            paused: false,
            new_count: 0,
            col_widths,
            columns,
            visible_height,
            highlight_errors: config.highlight_errors,
        }
    }

    pub fn push_row(&mut self, row: ClassifiedLine) {
        let detail_pairs = collect_detail_pairs(&row);
        self.rows.push(TableRowData { line: row, detail_pairs });
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
            self.ensure_selected_visible();
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

    pub fn jump_to_start(&mut self) {
        self.paused = true;
        self.new_count = 0;
        self.selected = 0;
        self.scroll_offset = 0;
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

    fn ensure_selected_visible(&mut self) {
        if self.rows.is_empty() {
            self.scroll_offset = 0;
            return;
        }
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
            return;
        }
        while self.scroll_offset < self.selected {
            let used_height: u16 = (self.scroll_offset..=self.selected)
                .map(|idx| self.row_height(idx))
                .sum();
            if used_height <= self.visible_height {
                break;
            }
            self.scroll_offset += 1;
        }
    }

    fn row_height(&self, idx: usize) -> u16 {
        let max_w = self.col_widths.width_of(&Column::Message).max(16) as usize;
        1 + compact_line_count(&self.rows[idx].detail_pairs, max_w) as u16
    }

    fn row_at_screen_y(&self, y: u16) -> Option<usize> {
        // Layout: row 0 header, [1..] data rows, last row status bar.
        if y == 0 || y > self.visible_height {
            return None;
        }
        let target = y - 1;
        let mut cursor_y: u16 = 0;
        let mut idx = self.scroll_offset;
        while idx < self.rows.len() && cursor_y < self.visible_height {
            let h = self.row_height(idx);
            if target < cursor_y + h {
                return Some(idx);
            }
            cursor_y = cursor_y.saturating_add(h);
            idx += 1;
        }
        None
    }

    pub fn click_select_or_expand(&mut self, y: u16) {
        if let Some(idx) = self.row_at_screen_y(y) {
            self.paused = true;
            self.selected = idx;
            self.ensure_selected_visible();
        }
    }
}

/// Returns ratatui fg+bg Style for a log level badge.
fn level_style(lvl: &LogLevel) -> Style {
    match lvl {
        LogLevel::Error => Style::default().fg(Color::LightRed),
        LogLevel::Warn  => Style::default().fg(Color::Yellow),
        LogLevel::Info  => Style::default().fg(Color::LightGreen),
        LogLevel::Debug => Style::default().fg(Color::LightBlue),
        LogLevel::Trace => Style::default().fg(Color::DarkGray),
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
    }
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
    let mut visible_indices: Vec<usize> = Vec::new();
    let mut consumed_height: u16 = 0;
    let mut idx = app.scroll_offset;
    while idx < app.rows.len() {
        let h = app.row_height(idx);
        if consumed_height + h > area.height {
            break;
        }
        visible_indices.push(idx);
        consumed_height += h;
        idx += 1;
    }

    let rows: Vec<Row> = visible_indices.iter().map(|abs_idx| {
        let row = &app.rows[*abs_idx].line;
        let detail_pairs = &app.rows[*abs_idx].detail_pairs;
        let is_selected = *abs_idx == app.selected;

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
                if *col == Column::Message {
                    let key_style = Style::default().fg(Color::Yellow);
                    let value_style = Style::default().fg(Color::DarkGray);
                    let is_error_keyword = app.highlight_errors
                        && contains_error_keyword(&raw);
                    let message_style = if is_error_keyword {
                        Style::default().fg(Color::LightRed)
                    } else {
                        match &row.level {
                            Some(LogLevel::Error) => Style::default().fg(Color::LightRed),
                            _ => Style::default().fg(Color::White),
                        }
                    };
                    let mut lines = vec![Line::from(Span::styled(raw, message_style))];
                    lines.extend(build_compact_detail_lines(
                        detail_pairs,
                        max_w.max(16),
                        key_style,
                        value_style,
                    ));
                    let text = Text::from(lines);
                    Cell::from(text).style(base_style)
                } else {
                    let text = truncate(&raw, max_w);
                    let cell_style = base_style.fg(Color::DarkGray);
                    Cell::from(text).style(cell_style)
                }
            }
        }).collect();

        let mut table_row = Row::new(cells).style(base_style);
        let detail_line_count = compact_line_count(detail_pairs, app.col_widths.width_of(&Column::Message).max(16) as usize);
        table_row = table_row.height(1 + detail_line_count as u16);

        table_row
    }).collect();

    let table = Table::new(rows, widths);
    f.render_widget(table, area);
}

fn collect_detail_pairs(row: &ClassifiedLine) -> Vec<(String, String)> {
    let mut pairs: Vec<(String, String)> = Vec::new();
    if let Some(ref tid) = row.trace_id {
        pairs.push(("trace_id".to_string(), tid.clone()));
    }
    if let Some(ref c) = row.caller {
        pairs.push(("caller".to_string(), c.clone()));
    }
    for (k, v) in &row.extras {
        pairs.push((k.clone(), value_to_string(v)));
    }
    for (i, cont) in row.continuation_lines.iter().enumerate() {
        pairs.push((format!("continuation_{:02}", i + 1), cont.clone()));
    }
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    pairs
}

fn build_compact_detail_lines(
    detail_pairs: &[(String, String)],
    max_width: usize,
    key_style: Style,
    value_style: Style,
) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current: Vec<(String, String, usize)> = Vec::new();
    let mut current_width: usize = 0;

    for (k, v) in detail_pairs {
        let token = format!("{}: {}", k, v);
        let token_width = token.chars().count();
        let sep_width = if current.is_empty() { 0 } else { 1 };
        let candidate_width = current_width + sep_width + token_width;
        if !current.is_empty() && candidate_width > max_width {
            lines.push(pairs_to_line(&current, key_style, value_style));
            current.clear();
            current_width = 0;
        }
        if !current.is_empty() {
            current_width += 1;
        }
        current_width += token_width;
        current.push((k.clone(), v.clone(), token_width));
    }
    if !current.is_empty() {
        lines.push(pairs_to_line(&current, key_style, value_style));
    }
    lines
}

fn pairs_to_line(
    pairs: &[(String, String, usize)],
    key_style: Style,
    value_style: Style,
) -> Line<'static> {
    let mut spans = vec![];
    for (idx, (k, v, _)) in pairs.iter().enumerate() {
        if idx > 0 {
            spans.push(Span::raw(" "));
        }
        spans.push(Span::styled(format!("{}:", k), key_style));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(v.clone(), value_style));
    }
    Line::from(spans)
}

fn compact_line_count(detail_pairs: &[(String, String)], max_width: usize) -> usize {
    if detail_pairs.is_empty() {
        return 0;
    }
    let mut lines = 1usize;
    let mut current_width = 0usize;
    for (k, v) in detail_pairs {
        let token_width = format!("{}: {}", k, v).chars().count();
        let sep_width = if current_width == 0 { 0 } else { 1 };
        if current_width != 0 && current_width + sep_width + token_width > max_width {
            lines += 1;
            current_width = token_width;
        } else {
            current_width += sep_width + token_width;
        }
    }
    lines
}

fn contains_error_keyword(msg: &str) -> bool {
    msg.contains("error") || msg.contains("Error") || msg.contains("ERROR")
        || msg.contains("err") || msg.contains("Err")
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

    let row_info = if app.rows.is_empty() {
        "row 0/0".to_string()
    } else {
        format!("row {}/{}", app.selected + 1, app.rows.len())
    };

    let left = format!(
        "↑↓/wheel scroll · g/Home top · G/End latest · q quit{}",
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
}

/// Handle a terminal event. Returns false if the app should quit.
pub fn handle_event(app: &mut App, event: Event) -> bool {
    match event {
        Event::Key(KeyEvent { code, modifiers, kind, .. }) => {
            if kind != KeyEventKind::Press && kind != KeyEventKind::Repeat {
                return true;
            }
            match code {
                KeyCode::Char('q') if modifiers.contains(KeyModifiers::SUPER) => return false,
                KeyCode::Char('q') | KeyCode::Char('Q') => return false,
                KeyCode::Up => {
                    app.scroll_up();
                }
                KeyCode::Down => {
                    app.scroll_down();
                }
                KeyCode::Home => {
                    app.jump_to_start();
                }
                KeyCode::End => {
                    app.jump_to_end();
                }
                KeyCode::Char('g') => {
                    app.jump_to_start();
                }
                KeyCode::Char('G') => {
                    app.jump_to_end();
                }
                KeyCode::Char(' ') => {
                    app.toggle_pause();
                }
                _ => {}
            }
        }
        Event::Mouse(MouseEvent { kind, row, .. }) => match kind {
            MouseEventKind::ScrollUp   => app.scroll_up(),
            MouseEventKind::ScrollDown => app.scroll_down(),
            MouseEventKind::Down(MouseButton::Left) => app.click_select_or_expand(row),
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
    // Keep terminal selection/copy behavior available by default.
    // Capturing mouse in alternate screen often prevents drag-to-copy in terminal emulators.
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let size = terminal.size()?;
    // Reserve 2 rows: 1 header + 1 status bar
    let visible_height = size.height.saturating_sub(2);

    let mut app = App::new(config, show_extras, size.width, visible_height);

    let (tx_log, rx_log) = mpsc::channel::<AppEvent>();

    // Stdin reader thread
    let tx_logs = tx_log.clone();
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
            if tx_logs.send(ev).is_err() {
                break;
            }
        }
        let _ = tx_logs.send(AppEvent::Eof);
    });

    // Main render loop — extracted so teardown always runs
    let result = (|| {
        let mut running = true;
        while running {
            terminal.draw(|f| render_ui(f, &app))?;

            // Poll keyboard/mouse in the main thread.
            // On Windows consoles this is more reliable than reading events in a worker thread.
            match event::poll(std::time::Duration::from_millis(1)) {
                Ok(true) => {
                    if let Ok(ev) = event::read() {
                        if !handle_event(&mut app, ev) {
                            running = false;
                            continue;
                        }
                    }
                }
                Ok(false) => {}
                Err(_) => {}
            }

            // Then process a bounded batch of log events each frame.
            let mut processed_logs = 0usize;
            while processed_logs < 256 {
                match rx_log.try_recv() {
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
                    Ok(AppEvent::Eof) => {}
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        running = false;
                        break;
                    }
                }
                processed_logs += 1;
            }

            // If there were no pending logs, block briefly to avoid a tight loop.
            if processed_logs == 0 {
                match rx_log.recv_timeout(std::time::Duration::from_millis(16)) {
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
                    Ok(AppEvent::Eof) => {}
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => running = false,
                }
            }
        }
        Ok(())
    })();

    // Always restore terminal, regardless of result
    let _ = disable_raw_mode();
    let _ = execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    );
    let _ = terminal.show_cursor();

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_from_str_known_values() {
        assert_eq!(Column::from_str("time"), Some(Column::Time));
        assert_eq!(Column::from_str("level"), Some(Column::Level));
        assert_eq!(Column::from_str("message"), Some(Column::Message));
        assert_eq!(Column::from_str("service"), None);
        assert_eq!(Column::from_str("label"), None);
        assert_eq!(Column::from_str("trace_id"), None);
        assert_eq!(Column::from_str("unknown"), None);
    }

    #[test]
    fn col_widths_message_gets_remainder() {
        let columns = vec![
            Column::Time,
            Column::Level,
            Column::Message,
        ];
        // time=8, level=7, separators=2 -> fixed=15, message=85
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
    fn app_click_select_sets_selected_row() {
        let config = Config::default();
        let mut app = App::new(&config, false, 200, 20);
        app.push_row(make_row("hello".into()));
        app.click_select_or_expand(1);
        assert_eq!(app.selected, 0);
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
