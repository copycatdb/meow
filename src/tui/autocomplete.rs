//! SQL keyword autocomplete state and matching logic.

/// Comprehensive T-SQL keywords for autocomplete.
const SQL_KEYWORDS: &[&str] = &[
    "ALL",
    "ALTER",
    "AND",
    "ANY",
    "AS",
    "ASC",
    "AVG",
    "BEGIN",
    "BETWEEN",
    "BIGINT",
    "BINARY",
    "BIT",
    "BY",
    "CASE",
    "CAST",
    "CATCH",
    "CHARINDEX",
    "CHECK",
    "CLUSTERED",
    "COALESCE",
    "COMMIT",
    "CONSTRAINT",
    "CONVERT",
    "COUNT",
    "CREATE",
    "CROSS",
    "CTE",
    "DATABASE",
    "DATE",
    "DATEADD",
    "DATEDIFF",
    "DATETIME",
    "DATETIME2",
    "DATETIMEOFFSET",
    "DECIMAL",
    "DECLARE",
    "DEFAULT",
    "DELAY",
    "DELETE",
    "DELETED",
    "DENSE_RANK",
    "DENY",
    "DESC",
    "DISTINCT",
    "DROP",
    "ELSE",
    "END",
    "EXEC",
    "EXECUTE",
    "EXISTS",
    "FETCH",
    "FLOAT",
    "FOREIGN",
    "FORMAT",
    "FROM",
    "FUNCTION",
    "GEOGRAPHY",
    "GEOMETRY",
    "GETDATE",
    "GO",
    "GRANT",
    "GROUP",
    "HAVING",
    "HIERARCHYID",
    "IDENTITY",
    "IF",
    "IMAGE",
    "IN",
    "INDEX",
    "INFORMATION_SCHEMA",
    "INNER",
    "INSERT",
    "INSERTED",
    "INT",
    "INTO",
    "IS",
    "ISNULL",
    "JOIN",
    "KEY",
    "LEFT",
    "LEN",
    "LIKE",
    "LOWER",
    "LTRIM",
    "MAX",
    "MERGE",
    "MIN",
    "MONEY",
    "NEXT",
    "NOT",
    "NTEXT",
    "NULL",
    "NULLIF",
    "NUMERIC",
    "NVARCHAR",
    "OFFSET",
    "ON",
    "ONLY",
    "OR",
    "ORDER",
    "OUTER",
    "OUTPUT",
    "OVER",
    "PARTITION",
    "PRIMARY",
    "PRINT",
    "PROCEDURE",
    "RAISERROR",
    "RANK",
    "REAL",
    "REFERENCES",
    "REPLACE",
    "REVOKE",
    "RIGHT",
    "ROLLBACK",
    "ROW_NUMBER",
    "ROWS",
    "ROWVERSION",
    "RTRIM",
    "SCHEMA",
    "SELECT",
    "SET",
    "SMALLINT",
    "SOME",
    "STRING_AGG",
    "STUFF",
    "SUBSTRING",
    "SUM",
    "SYSDATETIME",
    "TABLE",
    "TEXT",
    "THEN",
    "THROW",
    "TIME",
    "TINYINT",
    "TOP",
    "TRANSACTION",
    "TRIGGER",
    "TRIM",
    "TRUNCATE",
    "TRY",
    "UNION",
    "UNIQUE",
    "UNIQUEIDENTIFIER",
    "UPDATE",
    "UPPER",
    "USE",
    "VALUES",
    "VARBINARY",
    "VARCHAR",
    "VIEW",
    "WAITFOR",
    "WHEN",
    "WHERE",
    "WHILE",
    "WITH",
    "XML",
    // System procs/views (lowercase by convention)
    "sp_columns",
    "sp_help",
    "sp_who",
    "sys",
];

/// Autocomplete popup state.
#[derive(Debug, Clone)]
pub struct Autocomplete {
    /// Whether the popup is currently visible.
    pub active: bool,
    /// Current list of matching suggestions.
    pub suggestions: Vec<&'static str>,
    /// Currently selected index in suggestions.
    pub selected: usize,
    /// The prefix being matched (the partial word the user typed).
    pub prefix: String,
}

impl Default for Autocomplete {
    fn default() -> Self {
        Self {
            active: false,
            suggestions: Vec::new(),
            selected: 0,
            prefix: String::new(),
        }
    }
}

impl Autocomplete {
    /// Update suggestions based on the current word at cursor.
    /// Call this after every keystroke in the editor.
    pub fn update(&mut self, lines: &[String], cursor_row: usize, cursor_col: usize) {
        let prefix = extract_current_word(lines, cursor_row, cursor_col);
        if prefix.len() < 2 {
            self.dismiss();
            return;
        }
        let upper = prefix.to_ascii_uppercase();
        let matches: Vec<&'static str> = SQL_KEYWORDS
            .iter()
            .filter(|kw| kw.to_ascii_uppercase().starts_with(&upper))
            .copied()
            .collect();
        if matches.is_empty() {
            self.dismiss();
        } else {
            self.prefix = prefix;
            self.suggestions = matches;
            self.selected = self.selected.min(self.suggestions.len().saturating_sub(1));
            self.active = true;
        }
    }

    /// Dismiss the autocomplete popup.
    pub fn dismiss(&mut self) {
        self.active = false;
        self.suggestions.clear();
        self.selected = 0;
        self.prefix.clear();
    }

    /// Move selection up.
    pub fn prev(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected = self
                .selected
                .checked_sub(1)
                .unwrap_or(self.suggestions.len() - 1);
        }
    }

    /// Move selection down.
    pub fn next(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected = (self.selected + 1) % self.suggestions.len();
        }
    }

    /// Get the currently selected suggestion, if any.
    pub fn selected_keyword(&self) -> Option<&'static str> {
        self.suggestions.get(self.selected).copied()
    }
}

/// Extract the current word being typed at the cursor position.
/// Scans backward from cursor to find the word start.
fn extract_current_word(lines: &[String], row: usize, col: usize) -> String {
    if row >= lines.len() {
        return String::new();
    }
    let line = &lines[row];
    let bytes = line.as_bytes();
    let col = col.min(bytes.len());
    let mut start = col;
    while start > 0 {
        let ch = bytes[start - 1];
        if ch.is_ascii_alphanumeric() || ch == b'_' {
            start -= 1;
        } else {
            break;
        }
    }
    line[start..col].to_string()
}
