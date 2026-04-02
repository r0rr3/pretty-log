use crate::classifier::ClassifiedLine;
use crate::config::Config;
use std::io;

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

/// Placeholder for table mode entry point (implemented in Tasks 6-7).
pub fn run_table_mode(_config: &Config, _show_extras: bool) -> io::Result<()> {
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
