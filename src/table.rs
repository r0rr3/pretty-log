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

// --- KMP Searcher ---

struct KmpSearcher {
    pattern_lower: Vec<u8>,
    failure: Vec<usize>,
}

impl KmpSearcher {
    fn new(query: &str) -> Self {
        let pattern_lower: Vec<u8> = query.to_lowercase().into_bytes();
        let m = pattern_lower.len();
        let mut failure = vec![0usize; m.max(1)];
        if m > 1 {
            let mut k = 0usize;
            for i in 1..m {
                while k > 0 && pattern_lower[k] != pattern_lower[i] {
                    k = failure[k - 1];
                }
                if pattern_lower[k] == pattern_lower[i] {
                    k += 1;
                }
                failure[i] = k;
            }
        }
        KmpSearcher { pattern_lower, failure }
    }

    fn matches_text(&self, text: &str) -> bool {
        if self.pattern_lower.is_empty() {
            return false;
        }
        let text_lower: Vec<u8> = text.to_lowercase().into_bytes();
        let n = text_lower.len();
        let m = self.pattern_lower.len();
        if m > n {
            return false;
        }
        let mut k = 0usize;
        for i in 0..n {
            while k > 0 && self.pattern_lower[k] != text_lower[i] {
                k = self.failure[k - 1];
            }
            if self.pattern_lower[k] == text_lower[i] {
                k += 1;
            }
            if k == m {
                return true;
            }
        }
        false
    }
}

// --- Search State ---

pub struct SearchState {
    pub typing: bool,
    pub query: String,
    pub matches: Vec<usize>,
    pub current: usize,
}

impl SearchState {
    fn new() -> Self {
        SearchState {
            typing: false,
            query: String::new(),
            matches: Vec::new(),
            current: 0,
        }
    }

    pub fn is_active(&self) -> bool {
        self.typing || !self.query.is_empty()
    }

    fn update_matches(&mut self, rows: &[TableRowData]) {
        self.matches.clear();
        if self.query.is_empty() {
            self.current = 0;
            return;
        }
        let searcher = KmpSearcher::new(&self.query);
        for (i, row) in rows.iter().enumerate() {
            if row_matches_searcher(row, &searcher) {
                self.matches.push(i);
            }
        }
        if !self.matches.is_empty() && self.current >= self.matches.len() {
            self.current = self.matches.len() - 1;
        }
    }
}

fn row_matches_searcher(row: &TableRowData, searcher: &KmpSearcher) -> bool {
    if let Some(ref msg) = row.line.message {
        if searcher.matches_text(msg) {
            return true;
        }
    }
    for (k, v) in &row.detail_pairs {
        if searcher.matches_text(k) || searcher.matches_text(v) {
            return true;
        }
    }
    false
}

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
            "time"    => Some(Column::Time),
            "level"   => Some(Column::Level),
            "message" => Some(Column::Message),
            _         => None,
        }
    }

    pub fn header(&self) -> &str {
        match self {
            Column::Time    => "TIME",
            Column::Level   => "LEVEL",
            Column::Message => "MESSAGE",
        }
    }

    /// Fixed minimum width for this column. Message has no fixed width (gets remainder).
    pub fn fixed_width(&self) -> Option<u16> {
        match self {
            Column::Time    => Some(20),
            Column::Level   => Some(5),
            Column::Message => None,
        }
    }
}

// --- ColWidths ---

pub struct ColWidths {
    pub widths: Vec<(Column, u16)>,
}

impl ColWidths {
    pub fn compute(columns: &[Column], terminal_width: u16) -> Self {
        let separator_total = columns.len().saturating_sub(1) as u16;
        let fixed_total: u16 = columns.iter().filter_map(|c| c.fixed_width()).sum();
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
    pub search: SearchState,
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
            search: SearchState::new(),
        }
    }

    pub fn push_row(&mut self, row: ClassifiedLine) {
        let detail_pairs = collect_detail_pairs(&row);
        self.rows.push(TableRowData { line: row, detail_pairs });
        if !self.paused {
            self.selected = self.rows.len() - 1;
            self.adjust_scroll_to_show_last();
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
            self.adjust_scroll_to_show_last();
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

    pub fn search_next(&mut self) {
        if self.search.matches.is_empty() {
            return;
        }
        self.search.current = (self.search.current + 1) % self.search.matches.len();
        self.selected = self.search.matches[self.search.current];
        self.paused = true;
        self.ensure_selected_visible();
    }

    pub fn search_prev(&mut self) {
        if self.search.matches.is_empty() {
            return;
        }
        self.search.current = if self.search.current == 0 {
            self.search.matches.len() - 1
        } else {
            self.search.current - 1
        };
        self.selected = self.search.matches[self.search.current];
        self.paused = true;
        self.ensure_selected_visible();
    }

    /// Adjust scroll so that `selected` is at the bottom of the visible area.
    fn adjust_scroll_to_show_last(&mut self) {
        if self.rows.is_empty() {
            self.scroll_offset = 0;
            return;
        }
        self.scroll_offset = self.scroll_offset_for_bottom(self.selected);
    }

    /// Compute scroll_offset such that `last_row` appears at the bottom of the visible area.
    fn scroll_offset_for_bottom(&self, last_row: usize) -> usize {
        let vh = self.visible_height as usize;
        if vh == 0 {
            return last_row;
        }
        let mut h = 0usize;
        let mut idx = last_row;
        loop {
            let rh = self.row_height(idx) as usize;
            if h + rh > vh {
                // idx row no longer fits — start from idx+1
                return idx + 1;
            }
            h += rh;
            if idx == 0 {
                break;
            }
            idx -= 1;
        }
        0
    }

    fn ensure_selected_visible(&mut self) {
        if self.rows.is_empty() {
            self.scroll_offset = 0;
            return;
        }
        // If selected is above the scroll window, scroll up
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
            return;
        }
        // Check whether selected fits within the current window
        let mut h = 0usize;
        let mut fits = false;
        for i in self.scroll_offset..self.rows.len() {
            let rh = self.row_height(i) as usize;
            if i == self.selected {
                // Check if this row starts within the visible area
                if h + rh <= self.visible_height as usize {
                    fits = true;
                }
                break;
            }
            h += rh;
            if h >= self.visible_height as usize {
                break;
            }
        }
        if !fits {
            self.scroll_offset = self.scroll_offset_for_bottom(self.selected);
        }
    }

    pub fn row_height(&self, idx: usize) -> u16 {
        let msg_w = self.col_widths.width_of(&Column::Message).max(16) as usize;
        let msg_lines = match &self.rows[idx].line.message {
            Some(s) if !s.is_empty() => wrap_line_count(s.chars().count(), msg_w),
            _ => 1,
        };
        let detail_lines = compact_line_count(&self.rows[idx].detail_pairs, msg_w);
        (msg_lines + detail_lines) as u16
    }

    fn row_at_screen_y(&self, y: u16) -> Option<usize> {
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

fn wrap_line_count(char_count: usize, width: usize) -> usize {
    if width == 0 || char_count == 0 {
        return 1;
    }
    (char_count + width - 1) / width
}

/// Wrap `text` into `Vec<Line>` with search highlighting applied.
fn wrap_and_highlight(
    text: &str,
    width: usize,
    base_style: Style,
    search: &SearchState,
) -> Vec<Line<'static>> {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return vec![Line::from(Span::styled(String::new(), base_style))];
    }

    let do_highlight = !search.query.is_empty() && search.is_active();
    let highlight_style = Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD);

    let mut lines = vec![];
    let mut offset = 0;
    loop {
        let end = (offset + width).min(chars.len());
        let chunk: String = chars[offset..end].iter().collect();

        if do_highlight {
            let spans = highlight_text_spans(&chunk, &search.query, base_style, highlight_style);
            lines.push(Line::from(spans));
        } else {
            lines.push(Line::from(Span::styled(chunk, base_style)));
        }

        if end >= chars.len() {
            break;
        }
        offset = end;
    }
    lines
}

/// Build spans from `text` with case-insensitive `query` highlighted.
fn highlight_text_spans(
    text: &str,
    query: &str,
    normal: Style,
    highlight: Style,
) -> Vec<Span<'static>> {
    if query.is_empty() {
        return vec![Span::styled(text.to_string(), normal)];
    }
    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();
    let qlen = query_lower.len();

    let mut spans: Vec<Span<'static>> = vec![];
    let mut last_byte = 0usize;
    let mut search_from = 0usize;

    while search_from < text_lower.len() {
        match text_lower[search_from..].find(&query_lower) {
            None => break,
            Some(rel) => {
                let start = search_from + rel;
                let end = start + qlen;
                // Guard UTF-8 boundaries
                if !text.is_char_boundary(start) || !text.is_char_boundary(end) {
                    search_from += 1;
                    continue;
                }
                if start > last_byte {
                    spans.push(Span::styled(text[last_byte..start].to_string(), normal));
                }
                spans.push(Span::styled(text[start..end].to_string(), highlight));
                last_byte = end;
                search_from = end;
                if search_from >= text_lower.len() {
                    break;
                }
            }
        }
    }
    if last_byte < text.len() {
        spans.push(Span::styled(text[last_byte..].to_string(), normal));
    }
    if spans.is_empty() {
        spans.push(Span::styled(text.to_string(), normal));
    }
    spans
}

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
        LogLevel::Error      => "ERROR",
        LogLevel::Warn       => "WARN ",
        LogLevel::Info       => "INFO ",
        LogLevel::Debug      => "DEBUG",
        LogLevel::Trace      => "TRACE",
        LogLevel::Unknown(_) => "?????",
    }
}

fn cell_value(col: &Column, row: &ClassifiedLine) -> String {
    match col {
        Column::Time    => row.timestamp.as_deref()
            .map(crate::renderer::shorten_timestamp)
            .unwrap_or_default(),
        Column::Level   => String::new(),
        Column::Message => row.message.clone().unwrap_or_default(),
    }
}

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
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
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
    let table = Table::new(vec![header], widths).block(Block::default());
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
        let row     = &app.rows[*abs_idx].line;
        let detail_pairs = &app.rows[*abs_idx].detail_pairs;
        let is_selected = *abs_idx == app.selected;
        let is_match = !app.search.query.is_empty()
            && app.search.matches.contains(abs_idx);

        let base_style = if is_selected {
            Style::default().bg(Color::Rgb(30, 41, 59))
        } else if is_match {
            Style::default().bg(Color::Rgb(20, 30, 20))
        } else {
            Style::default()
        };

        let cells: Vec<Cell> = app.columns.iter().map(|col| {
            if *col == Column::Level {
                let (text, style) = match &row.level {
                    Some(lvl) => (level_text(lvl), level_style(lvl)),
                    None      => ("?????", Style::default()),
                };
                Cell::from(text).style(style)
            } else if *col == Column::Message {
                let msg_w = app.col_widths.width_of(&Column::Message).max(16) as usize;
                let raw   = cell_value(col, row);

                let is_error_keyword = app.highlight_errors && contains_error_keyword(&raw);
                let message_style = if is_error_keyword {
                    Style::default().fg(Color::LightRed)
                } else {
                    match &row.level {
                        Some(LogLevel::Error) => Style::default().fg(Color::LightRed),
                        _ => Style::default().fg(Color::White),
                    }
                };

                let mut lines = wrap_and_highlight(&raw, msg_w, message_style, &app.search);

                let key_style   = Style::default().fg(Color::Yellow);
                let value_style = Style::default().fg(Color::DarkGray);
                lines.extend(build_compact_detail_lines(
                    detail_pairs,
                    msg_w.max(16),
                    key_style,
                    value_style,
                ));

                let text = Text::from(lines);
                Cell::from(text).style(base_style)
            } else {
                // Time column — truncate to column width
                let max_w = app.col_widths.width_of(col) as usize;
                let raw   = cell_value(col, row);
                let text  = truncate(&raw, max_w);
                Cell::from(text).style(base_style.fg(Color::DarkGray))
            }
        }).collect();

        let detail_line_count = compact_line_count(
            detail_pairs,
            app.col_widths.width_of(&Column::Message).max(16) as usize,
        );
        let msg_lines = match &row.message {
            Some(s) if !s.is_empty() => {
                let w = app.col_widths.width_of(&Column::Message).max(16) as usize;
                wrap_line_count(s.chars().count(), w)
            }
            _ => 1,
        };
        let row_h = (msg_lines + detail_line_count) as u16;

        Row::new(cells).style(base_style).height(row_h)
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
        let token       = format!("{}: {}", k, v);
        let token_width = token.chars().count();
        let sep_width   = if current.is_empty() { 0 } else { 1 };
        let candidate   = current_width + sep_width + token_width;
        if !current.is_empty() && candidate > max_width {
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
    let mut lines        = 1usize;
    let mut current_width = 0usize;
    for (k, v) in detail_pairs {
        let token_width = format!("{}: {}", k, v).chars().count();
        let sep_width   = if current_width == 0 { 0 } else { 1 };
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
        || msg.contains("err")  || msg.contains("Err")
}

fn render_status(f: &mut Frame, app: &App, area: Rect) {
    if app.search.typing || !app.search.query.is_empty() {
        render_search_bar(f, app, area);
    } else {
        render_normal_status(f, app, area);
    }
}

fn render_normal_status(f: &mut Frame, app: &App, area: Rect) {
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
        "↑↓/wheel scroll · g/Home top · G/End latest · / search · q quit{}",
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

fn render_search_bar(f: &mut Frame, app: &App, area: Rect) {
    let dim   = Style::default().fg(Color::DarkGray);
    let query_style = Style::default().fg(Color::White).add_modifier(Modifier::BOLD);
    let match_style = Style::default().fg(Color::Yellow);

    let match_info = if app.search.query.is_empty() {
        String::new()
    } else if app.search.matches.is_empty() {
        "  no matches".to_string()
    } else {
        format!("  {}/{}", app.search.current + 1, app.search.matches.len())
    };

    let cursor = if app.search.typing { "█" } else { "" };

    let mut spans = vec![
        Span::styled("/", match_style),
        Span::styled(app.search.query.clone(), query_style),
        Span::styled(cursor, query_style),
        Span::styled(match_info, match_style),
    ];

    if !app.search.typing && !app.search.query.is_empty() {
        spans.push(Span::styled("  n next · N prev · / edit · Esc clear", dim));
    } else {
        spans.push(Span::styled("  Enter confirm · Esc cancel", dim));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);
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

            // --- Search typing mode ---
            if app.search.typing {
                match code {
                    KeyCode::Esc => {
                        app.search.typing = false;
                        app.search.query.clear();
                        app.search.matches.clear();
                        app.search.current = 0;
                    }
                    KeyCode::Enter => {
                        app.search.typing = false;
                        // Jump to first match if any
                        if !app.search.matches.is_empty() {
                            app.search.current = 0;
                            app.selected = app.search.matches[0];
                            app.paused = true;
                            app.ensure_selected_visible();
                        }
                    }
                    KeyCode::Backspace => {
                        app.search.query.pop();
                        app.search.update_matches(&app.rows);
                    }
                    KeyCode::Char(c) => {
                        app.search.query.push(c);
                        app.search.update_matches(&app.rows);
                        // Jump to nearest match
                        if !app.search.matches.is_empty() {
                            let from = app.selected;
                            let i = app.search.matches.partition_point(|&x| x < from);
                            app.search.current = if i < app.search.matches.len() { i } else { 0 };
                            app.selected = app.search.matches[app.search.current];
                            app.paused = true;
                            app.ensure_selected_visible();
                        }
                    }
                    _ => {}
                }
                return true;
            }

            // --- Normal mode ---
            match code {
                KeyCode::Char('q') if modifiers.contains(KeyModifiers::SUPER) => return false,
                KeyCode::Char('q') | KeyCode::Char('Q') => return false,
                KeyCode::Up        => app.scroll_up(),
                KeyCode::Down      => app.scroll_down(),
                KeyCode::Home      => app.jump_to_start(),
                KeyCode::End       => app.jump_to_end(),
                KeyCode::Char('g') => app.jump_to_start(),
                KeyCode::Char('G') => app.jump_to_end(),
                KeyCode::Char(' ') => app.toggle_pause(),
                KeyCode::Char('/') => {
                    app.search.typing = true;
                }
                KeyCode::Char('n') => app.search_next(),
                KeyCode::Char('N') => app.search_prev(),
                KeyCode::Esc => {
                    if app.search.is_active() {
                        app.search.query.clear();
                        app.search.matches.clear();
                        app.search.current = 0;
                        app.search.typing = false;
                    }
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
    use crate::reader::LogicalLine;
    use crate::parser::{parse_line, ParseResult};
    use crate::classifier::classify;
    use regex::Regex;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend  = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let size = terminal.size()?;
    let visible_height = size.height.saturating_sub(2);

    let mut app = App::new(config, show_extras, size.width, visible_height);

    let (tx_log, rx_log) = mpsc::channel::<AppEvent>();

    // Thread 1: raw stdin reader → sends raw lines to raw_rx.
    // Runs independently so crossterm and the assembler never share the stdin handle.
    let (raw_tx, raw_rx) = mpsc::channel::<Option<String>>();
    thread::spawn(move || {
        use io::BufRead;
        let stdin  = io::stdin();
        let locked = stdin.lock();
        let reader = io::BufReader::new(locked);
        for line_result in reader.lines() {
            match line_result {
                Ok(line) => { if raw_tx.send(Some(line)).is_err() { break; } }
                Err(_)   => break,
            }
        }
        let _ = raw_tx.send(None); // signal EOF
    });

    // Thread 2: logical-line assembler with 100 ms timeout flush.
    // The timeout ensures that the last (pending) line is always emitted
    // within 100 ms even if no subsequent line arrives (tail -f style streaming).
    let tx_logs = tx_log.clone();
    let cfg = config.clone();
    thread::spawn(move || {
        let continuation_re: Option<Regex> = if cfg.multiline.enabled {
            Regex::new(&cfg.multiline.continuation_pattern).ok()
        } else {
            None
        };
        let multiline_enabled = cfg.multiline.enabled;

        let is_cont = |line: &str| -> bool {
            if !multiline_enabled { return false; }
            match &continuation_re {
                Some(re) => re.is_match(line),
                None     => !line.trim_start().starts_with('{'),
            }
        };

        let make_event = |p: LogicalLine| -> AppEvent {
            match parse_line(&p.main, p.continuations) {
                ParseResult::Json(parsed) => AppEvent::LogLine(classify(parsed, &cfg)),
                ParseResult::Raw { line, .. } => AppEvent::RawLine(line),
            }
        };

        let mut pending: Option<LogicalLine> = None;
        let flush_timeout = std::time::Duration::from_millis(100);

        loop {
            match raw_rx.recv_timeout(flush_timeout) {
                // New raw line arrived
                Ok(Some(line)) => {
                    if line.is_empty() { continue; }
                    if is_cont(&line) {
                        if let Some(ref mut p) = pending {
                            p.continuations.push(line);
                        } else {
                            pending = Some(LogicalLine { main: line, continuations: vec![] });
                        }
                    } else {
                        let prev = pending.replace(LogicalLine { main: line, continuations: vec![] });
                        if let Some(p) = prev {
                            if tx_logs.send(make_event(p)).is_err() { break; }
                        }
                    }
                }
                // EOF or raw reader disconnected — flush pending and exit
                Ok(None) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                    if let Some(p) = pending.take() {
                        let _ = tx_logs.send(make_event(p));
                    }
                    let _ = tx_logs.send(AppEvent::Eof);
                    break;
                }
                // Timeout: no new line in 100 ms → flush pending immediately
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if let Some(p) = pending.take() {
                        if tx_logs.send(make_event(p)).is_err() { break; }
                    }
                }
            }
        }
    });

    let result = (|| {
        let mut running = true;
        while running {
            terminal.draw(|f| render_ui(f, &app))?;

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
                Err(_)    => {}
            }

            let mut processed = 0usize;
            while processed < 256 {
                match rx_log.try_recv() {
                    Ok(AppEvent::LogLine(line)) => app.push_row(line),
                    Ok(AppEvent::RawLine(raw))  => app.push_row(ClassifiedLine {
                        level: None, timestamp: None, message: Some(raw),
                        trace_id: None, caller: None, extras: vec![],
                        continuation_lines: vec![],
                    }),
                    Ok(AppEvent::Eof) => {}
                    Err(mpsc::TryRecvError::Empty)        => break,
                    Err(mpsc::TryRecvError::Disconnected) => { running = false; break; }
                }
                processed += 1;
            }

            if processed == 0 {
                match rx_log.recv_timeout(std::time::Duration::from_millis(16)) {
                    Ok(AppEvent::LogLine(line)) => app.push_row(line),
                    Ok(AppEvent::RawLine(raw))  => app.push_row(ClassifiedLine {
                        level: None, timestamp: None, message: Some(raw),
                        trace_id: None, caller: None, extras: vec![],
                        continuation_lines: vec![],
                    }),
                    Ok(AppEvent::Eof) => {}
                    Err(mpsc::RecvTimeoutError::Timeout)      => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => running = false,
                }
            }
        }
        Ok(())
    })();

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
        assert_eq!(Column::from_str("time"),    Some(Column::Time));
        assert_eq!(Column::from_str("level"),   Some(Column::Level));
        assert_eq!(Column::from_str("message"), Some(Column::Message));
        assert_eq!(Column::from_str("service"), None);
        assert_eq!(Column::from_str("unknown"), None);
    }

    #[test]
    fn col_widths_message_gets_remainder() {
        let columns = vec![Column::Time, Column::Level, Column::Message];
        // time=20, level=5, separators=2 -> fixed=27; message=73
        let cw = ColWidths::compute(&columns, 100);
        let msg_width = cw.width_of(&Column::Message);
        assert!(msg_width >= 20, "message column got {}", msg_width);
        let total: u16 = cw.widths.iter().map(|(_, w)| w).sum();
        assert!(total <= 100);
    }

    #[test]
    fn time_column_width_is_20() {
        assert_eq!(Column::Time.fixed_width(), Some(20));
    }

    #[test]
    fn level_column_width_is_5() {
        assert_eq!(Column::Level.fixed_width(), Some(5));
    }

    #[test]
    fn wrap_line_count_basic() {
        assert_eq!(wrap_line_count(60, 60), 1);
        assert_eq!(wrap_line_count(61, 60), 2);
        assert_eq!(wrap_line_count(120, 60), 2);
        assert_eq!(wrap_line_count(121, 60), 3);
        assert_eq!(wrap_line_count(0, 60), 1);
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

    #[test]
    fn scroll_offset_for_bottom_positions_last_row_at_bottom() {
        let config = Config::default();
        let mut app = App::new(&config, false, 200, 5);
        for i in 0..10 {
            app.push_row(make_row(format!("msg {}", i)));
        }
        // All rows are 1-line tall; with visible_height=5, last 5 rows should show
        assert_eq!(app.scroll_offset, 5);
        assert_eq!(app.selected, 9);
    }

    #[test]
    fn kmp_searcher_finds_substring() {
        let s = KmpSearcher::new("error");
        assert!(s.matches_text("request error occurred"));
        assert!(s.matches_text("ERROR message")); // case-insensitive
        assert!(!s.matches_text("everything is fine"));
    }

    #[test]
    fn search_state_update_matches() {
        let rows = vec![
            make_table_row("database connection error"),
            make_table_row("user login success"),
            make_table_row("query timeout error"),
        ];
        let mut search = SearchState::new();
        search.query = "error".to_string();
        search.update_matches(&rows);
        assert_eq!(search.matches, vec![0, 2]);
    }

    #[test]
    fn search_next_wraps_around() {
        let config = Config::default();
        let mut app = App::new(&config, false, 200, 20);
        for msg in &["alpha error", "beta ok", "gamma error"] {
            app.push_row(make_row(msg.to_string()));
        }
        app.search.query = "error".to_string();
        app.search.update_matches(&app.rows);
        app.search.current = 1; // at second match (row 2)
        app.search_next();
        // Should wrap to first match (row 0)
        assert_eq!(app.search.current, 0);
        assert_eq!(app.selected, 0);
    }

    fn make_row(msg: String) -> ClassifiedLine {
        ClassifiedLine {
            level: None, timestamp: None, message: Some(msg),
            trace_id: None, caller: None, extras: vec![],
            continuation_lines: vec![],
        }
    }

    fn make_table_row(msg: &str) -> TableRowData {
        TableRowData {
            line: make_row(msg.to_string()),
            detail_pairs: vec![],
        }
    }
}
