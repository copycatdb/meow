//! Application state machine for the TUI.

use crate::db;
use crate::tui::autocomplete::Autocomplete;

/// Which pane currently has focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPane {
    /// The SQL editor pane.
    Editor,
    /// The results table pane.
    Results,
    /// The object browser sidebar.
    Sidebar,
}

/// A node in the object browser tree.
#[derive(Debug, Clone)]
pub struct ObjectNode {
    /// Display label.
    pub name: String,
    /// Depth in the tree (0 = database, 1 = schema, 2 = table).
    pub depth: u8,
    /// Whether this node is expanded.
    pub expanded: bool,
    /// Children (lazy-loaded).
    pub children: Vec<ObjectNode>,
}

/// A single result set from a query.
#[derive(Debug, Clone, Default)]
pub struct ResultSet {
    /// Column headers.
    pub columns: Vec<String>,
    /// Row data as strings.
    pub rows: Vec<Vec<String>>,
}

/// Query result data ready for display.
#[derive(Debug, Clone, Default)]
pub struct QueryResult {
    /// All result sets from the query.
    pub result_sets: Vec<ResultSet>,
    /// How long the query took, in milliseconds.
    pub elapsed_ms: u128,
    /// Optional error message.
    pub error: Option<String>,
}

impl QueryResult {
    /// Get columns of the current (or first) result set.
    pub fn columns(&self) -> &[String] {
        self.result_sets
            .first()
            .map(|rs| rs.columns.as_slice())
            .unwrap_or(&[])
    }

    /// Get rows of a specific result set.
    pub fn rows_for(&self, index: usize) -> &[Vec<String>] {
        self.result_sets
            .get(index)
            .map(|rs| rs.rows.as_slice())
            .unwrap_or(&[])
    }

    /// Get columns of a specific result set.
    pub fn columns_for(&self, index: usize) -> &[String] {
        self.result_sets
            .get(index)
            .map(|rs| rs.columns.as_slice())
            .unwrap_or(&[])
    }

    /// Total row count across all result sets.
    pub fn total_rows(&self) -> usize {
        self.result_sets.iter().map(|rs| rs.rows.len()).sum()
    }

    /// Helper to create a single-resultset QueryResult.
    pub fn single(columns: Vec<String>, rows: Vec<Vec<String>>, elapsed_ms: u128) -> Self {
        Self {
            result_sets: vec![ResultSet { columns, rows }],
            elapsed_ms,
            error: None,
        }
    }
}

/// The main application state.
pub struct App {
    /// Which pane has focus.
    pub focus: FocusPane,
    /// Whether the sidebar is visible.
    pub sidebar_visible: bool,
    /// The SQL editor text area.
    pub editor: tui_textarea::TextArea<'static>,
    /// Current query results.
    pub result: QueryResult,
    /// Object browser tree.
    pub objects: Vec<ObjectNode>,
    /// Scroll offset in the results table (rows).
    pub result_scroll: usize,
    /// Horizontal scroll offset in the results table (columns).
    pub result_col_scroll: usize,
    /// Sidebar scroll offset.
    pub sidebar_scroll: usize,
    /// Connection info string for the status bar.
    pub connection_info: String,
    /// Current database name.
    pub current_database: String,
    /// Whether the app should quit.
    pub should_quit: bool,
    /// Whether a query is currently running.
    pub query_running: bool,
    /// Query history.
    pub history: Vec<String>,
    /// Current position in history (-1 = current editor content).
    pub history_index: Option<usize>,
    /// Show help overlay.
    pub show_help: bool,
    /// Autocomplete state.
    pub autocomplete: Autocomplete,
    /// Which result set is currently displayed (for multi-resultset queries).
    pub current_result_set: usize,
    /// Expanded display mode (vertical record layout).
    pub expanded_mode: bool,
    /// Show query timing in results.
    pub show_timing: bool,
    /// Username used for the connection.
    pub user: String,
}

impl App {
    /// Create a new App with default state.
    pub fn new(host: &str, port: u16, database: &str, user: &str) -> Self {
        let mut editor = tui_textarea::TextArea::default();
        editor.set_cursor_line_style(ratatui::style::Style::default());
        editor.set_line_number_style(
            ratatui::style::Style::default().fg(ratatui::style::Color::DarkGray),
        );

        Self {
            focus: FocusPane::Editor,
            sidebar_visible: true,
            editor,
            result: QueryResult::default(),
            objects: Vec::new(),
            result_scroll: 0,
            result_col_scroll: 0,
            sidebar_scroll: 0,
            connection_info: format!("{}:{}", host, port),
            current_database: database.to_string(),
            should_quit: false,
            query_running: false,
            history: Vec::new(),
            history_index: None,
            show_help: false,
            autocomplete: Autocomplete::default(),
            current_result_set: 0,
            expanded_mode: false,
            show_timing: false,
            user: user.to_string(),
        }
    }

    /// Cycle focus to the next pane.
    pub fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            FocusPane::Editor => FocusPane::Results,
            FocusPane::Results => {
                if self.sidebar_visible {
                    FocusPane::Sidebar
                } else {
                    FocusPane::Editor
                }
            }
            FocusPane::Sidebar => FocusPane::Editor,
        };
    }

    /// Toggle sidebar visibility.
    pub fn toggle_sidebar(&mut self) {
        self.sidebar_visible = !self.sidebar_visible;
        if !self.sidebar_visible && self.focus == FocusPane::Sidebar {
            self.focus = FocusPane::Editor;
        }
    }

    /// Get the current editor content as a string.
    pub fn get_editor_text(&self) -> String {
        self.editor.lines().join("\n")
    }

    /// Clear the editor.
    pub fn clear_editor(&mut self) {
        self.editor = tui_textarea::TextArea::default();
        self.editor
            .set_cursor_line_style(ratatui::style::Style::default());
        self.editor.set_line_number_style(
            ratatui::style::Style::default().fg(ratatui::style::Color::DarkGray),
        );
    }

    /// Push current query to history and reset index.
    pub fn push_history(&mut self) {
        let text = self.get_editor_text();
        if !text.trim().is_empty() {
            self.history.push(text);
        }
        self.history_index = None;
    }

    /// Navigate history backward.
    pub fn history_prev(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let idx = match self.history_index {
            None => self.history.len().saturating_sub(1),
            Some(i) => i.saturating_sub(1),
        };
        self.history_index = Some(idx);
        self.set_editor_text(&self.history[idx].clone());
    }

    /// Navigate history forward.
    pub fn history_next(&mut self) {
        if let Some(idx) = self.history_index {
            if idx + 1 < self.history.len() {
                let new_idx = idx + 1;
                self.history_index = Some(new_idx);
                self.set_editor_text(&self.history[new_idx].clone());
            } else {
                self.history_index = None;
                self.clear_editor();
            }
        }
    }

    /// Set editor text content.
    fn set_editor_text(&mut self, text: &str) {
        let lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();
        let lines = if lines.is_empty() {
            vec!["".to_string()]
        } else {
            lines
        };
        self.editor = tui_textarea::TextArea::new(lines);
        self.editor
            .set_cursor_line_style(ratatui::style::Style::default());
        self.editor.set_line_number_style(
            ratatui::style::Style::default().fg(ratatui::style::Color::DarkGray),
        );
    }

    /// Scroll results down.
    pub fn scroll_results_down(&mut self) {
        let row_count = self.result.rows_for(self.current_result_set).len();
        if self.result_scroll + 1 < row_count {
            self.result_scroll += 1;
        }
    }

    /// Scroll results up.
    pub fn scroll_results_up(&mut self) {
        self.result_scroll = self.result_scroll.saturating_sub(1);
    }

    /// Scroll results right (horizontal).
    pub fn scroll_results_right(&mut self) {
        let col_count = self.result.columns_for(self.current_result_set).len();
        if col_count > 0 && self.result_col_scroll + 1 < col_count {
            self.result_col_scroll += 1;
        }
    }

    /// Scroll results left (horizontal).
    pub fn scroll_results_left(&mut self) {
        self.result_col_scroll = self.result_col_scroll.saturating_sub(1);
    }

    /// Scroll sidebar down.
    pub fn scroll_sidebar_down(&mut self) {
        self.sidebar_scroll += 1;
    }

    /// Scroll sidebar up.
    pub fn scroll_sidebar_up(&mut self) {
        self.sidebar_scroll = self.sidebar_scroll.saturating_sub(1);
    }

    /// Navigate to the next result set.
    pub fn next_result_set(&mut self) {
        if self.current_result_set + 1 < self.result.result_sets.len() {
            self.current_result_set += 1;
            self.result_scroll = 0;
            self.result_col_scroll = 0;
        }
    }

    /// Navigate to the previous result set.
    pub fn prev_result_set(&mut self) {
        if self.current_result_set > 0 {
            self.current_result_set -= 1;
            self.result_scroll = 0;
            self.result_col_scroll = 0;
        }
    }

    /// Toggle expand/collapse on the selected sidebar node.
    pub fn toggle_sidebar_node(&mut self) {
        if let Some(node) = get_flat_node_mut(&mut self.objects, self.sidebar_scroll) {
            node.expanded = !node.expanded;
        }
    }

    /// Build the object tree from a database connection.
    pub async fn load_objects(&mut self, client: &mut db::ConnectionHandle) {
        match db::query::fetch_object_tree(client).await {
            Ok(objects) => self.objects = objects,
            Err(e) => {
                self.result.error = Some(format!("Failed to load objects: {}", e));
            }
        }
    }
}

/// Get a mutable reference to the node at the given flat index in the tree.
fn get_flat_node_mut(nodes: &mut [ObjectNode], target: usize) -> Option<&mut ObjectNode> {
    let mut idx = 0;
    get_flat_node_mut_inner(nodes, target, &mut idx)
}

fn get_flat_node_mut_inner<'a>(
    nodes: &'a mut [ObjectNode],
    target: usize,
    idx: &mut usize,
) -> Option<&'a mut ObjectNode> {
    for node in nodes.iter_mut() {
        if *idx == target {
            return Some(node);
        }
        *idx += 1;
        if node.expanded
            && let Some(found) = get_flat_node_mut_inner(&mut node.children, target, idx)
        {
            return Some(found);
        }
    }
    None
}

/// Flatten the object tree for display, returning (depth, name, expanded, has_children).
pub fn flatten_tree(nodes: &[ObjectNode]) -> Vec<(u8, String, bool, bool)> {
    let mut out = Vec::new();
    flatten_tree_inner(nodes, &mut out);
    out
}

fn flatten_tree_inner(nodes: &[ObjectNode], out: &mut Vec<(u8, String, bool, bool)>) {
    for node in nodes {
        out.push((
            node.depth,
            node.name.clone(),
            node.expanded,
            !node.children.is_empty(),
        ));
        if node.expanded {
            flatten_tree_inner(&node.children, out);
        }
    }
}
