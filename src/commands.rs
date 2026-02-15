//! Slash command parser and SQL generation for psql-style commands.

/// Parsed slash command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlashCommand {
    /// `\d` — list all tables and views.
    ListAll,
    /// `\d <table>` — describe a table's columns.
    Describe(String),
    /// `\dt` — list tables only.
    ListTables,
    /// `\dv` — list views only.
    ListViews,
    /// `\di` — list indexes.
    ListIndexes,
    /// `\df` — list procedures and functions.
    ListFunctions,
    /// `\ds` — list schemas.
    ListSchemas,
    /// `\dn` — list databases.
    ListDatabases,
    /// `\c <db>` — switch database.
    UseDatabase(String),
    /// `\conninfo` — show connection info.
    ConnInfo,
    /// `\x` — toggle expanded display.
    ToggleExpanded,
    /// `\timing` — toggle query timing display.
    ToggleTiming,
    /// `\?` — show help.
    Help,
    /// `\q` — quit.
    Quit,
}

/// Result of handling a slash command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandAction {
    /// Execute this SQL and display results.
    ExecuteSql(String),
    /// Display a message in the results pane (columns + rows).
    DisplayMessage {
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    /// Toggle expanded mode.
    ToggleExpanded,
    /// Toggle timing mode.
    ToggleTiming,
    /// Quit the application.
    Quit,
}

/// Parse input text into a slash command. Returns `None` if not a slash command.
pub fn parse(input: &str) -> Option<SlashCommand> {
    let trimmed = input.trim();
    if !trimmed.starts_with('\\') {
        return None;
    }

    let parts: Vec<&str> = trimmed.splitn(2, char::is_whitespace).collect();
    let cmd = parts[0];
    let arg = parts.get(1).map(|s| s.trim()).filter(|s| !s.is_empty());

    match cmd {
        "\\d" => match arg {
            Some(table) => Some(SlashCommand::Describe(table.to_string())),
            None => Some(SlashCommand::ListAll),
        },
        "\\dt" => Some(SlashCommand::ListTables),
        "\\dv" => Some(SlashCommand::ListViews),
        "\\di" => Some(SlashCommand::ListIndexes),
        "\\df" => Some(SlashCommand::ListFunctions),
        "\\ds" => Some(SlashCommand::ListSchemas),
        "\\dn" => Some(SlashCommand::ListDatabases),
        "\\c" => arg.map(|db| SlashCommand::UseDatabase(db.to_string())),
        "\\conninfo" => Some(SlashCommand::ConnInfo),
        "\\x" => Some(SlashCommand::ToggleExpanded),
        "\\timing" => Some(SlashCommand::ToggleTiming),
        "\\?" => Some(SlashCommand::Help),
        "\\q" => Some(SlashCommand::Quit),
        _ => None,
    }
}

/// Generate the action for a slash command.
pub fn to_action(cmd: &SlashCommand, conn_info: &str, database: &str, user: &str) -> CommandAction {
    match cmd {
        SlashCommand::ListAll => CommandAction::ExecuteSql(
            "SELECT TABLE_SCHEMA, TABLE_NAME, TABLE_TYPE FROM INFORMATION_SCHEMA.TABLES ORDER BY TABLE_SCHEMA, TABLE_NAME".to_string(),
        ),
        SlashCommand::Describe(table) => CommandAction::ExecuteSql(format!(
            "SELECT COLUMN_NAME, DATA_TYPE, CHARACTER_MAXIMUM_LENGTH, IS_NULLABLE, COLUMN_DEFAULT FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_NAME = '{}' ORDER BY ORDINAL_POSITION",
            table.replace('\'', "''")
        )),
        SlashCommand::ListTables => CommandAction::ExecuteSql(
            "SELECT TABLE_SCHEMA, TABLE_NAME, TABLE_TYPE FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_TYPE = 'BASE TABLE' ORDER BY TABLE_SCHEMA, TABLE_NAME".to_string(),
        ),
        SlashCommand::ListViews => CommandAction::ExecuteSql(
            "SELECT TABLE_SCHEMA, TABLE_NAME, TABLE_TYPE FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_TYPE = 'VIEW' ORDER BY TABLE_SCHEMA, TABLE_NAME".to_string(),
        ),
        SlashCommand::ListIndexes => CommandAction::ExecuteSql(
            "SELECT t.name AS table_name, i.name AS index_name, i.type_desc, i.is_unique, i.is_primary_key FROM sys.indexes i JOIN sys.tables t ON i.object_id = t.object_id WHERE i.name IS NOT NULL ORDER BY t.name, i.name".to_string(),
        ),
        SlashCommand::ListFunctions => CommandAction::ExecuteSql(
            "SELECT ROUTINE_SCHEMA, ROUTINE_NAME, ROUTINE_TYPE FROM INFORMATION_SCHEMA.ROUTINES ORDER BY ROUTINE_SCHEMA, ROUTINE_NAME".to_string(),
        ),
        SlashCommand::ListSchemas => CommandAction::ExecuteSql(
            "SELECT schema_id, name FROM sys.schemas WHERE principal_id = 1 ORDER BY name".to_string(),
        ),
        SlashCommand::ListDatabases => CommandAction::ExecuteSql(
            "SELECT name, state_desc, recovery_model_desc FROM sys.databases ORDER BY name".to_string(),
        ),
        SlashCommand::UseDatabase(db) => CommandAction::ExecuteSql(format!("USE {}", db)),
        SlashCommand::ConnInfo => CommandAction::DisplayMessage {
            columns: vec!["Property".to_string(), "Value".to_string()],
            rows: vec![
                vec!["Server".to_string(), conn_info.to_string()],
                vec!["Database".to_string(), database.to_string()],
                vec!["User".to_string(), user.to_string()],
            ],
        },
        SlashCommand::ToggleExpanded => CommandAction::ToggleExpanded,
        SlashCommand::ToggleTiming => CommandAction::ToggleTiming,
        SlashCommand::Help => CommandAction::DisplayMessage {
            columns: vec!["Command".to_string(), "Description".to_string()],
            rows: vec![
                vec!["\\d".to_string(), "List all tables and views".to_string()],
                vec!["\\d <table>".to_string(), "Describe table columns".to_string()],
                vec!["\\dt".to_string(), "List tables only".to_string()],
                vec!["\\dv".to_string(), "List views only".to_string()],
                vec!["\\di".to_string(), "List indexes".to_string()],
                vec!["\\df".to_string(), "List procedures and functions".to_string()],
                vec!["\\ds".to_string(), "List schemas".to_string()],
                vec!["\\dn".to_string(), "List databases".to_string()],
                vec!["\\c <db>".to_string(), "Switch database".to_string()],
                vec!["\\conninfo".to_string(), "Show connection info".to_string()],
                vec!["\\x".to_string(), "Toggle expanded display".to_string()],
                vec!["\\timing".to_string(), "Toggle query timing display".to_string()],
                vec!["\\?".to_string(), "Show this help".to_string()],
                vec!["\\q".to_string(), "Quit".to_string()],
            ],
        },
        SlashCommand::Quit => CommandAction::Quit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_list_all() {
        assert_eq!(parse("\\d"), Some(SlashCommand::ListAll));
    }

    #[test]
    fn test_parse_describe() {
        assert_eq!(
            parse("\\d foo"),
            Some(SlashCommand::Describe("foo".to_string()))
        );
    }

    #[test]
    fn test_parse_describe_with_whitespace() {
        assert_eq!(
            parse("  \\d  bar  "),
            Some(SlashCommand::Describe("bar".to_string()))
        );
    }

    #[test]
    fn test_parse_list_tables() {
        assert_eq!(parse("\\dt"), Some(SlashCommand::ListTables));
    }

    #[test]
    fn test_parse_list_views() {
        assert_eq!(parse("\\dv"), Some(SlashCommand::ListViews));
    }

    #[test]
    fn test_parse_list_indexes() {
        assert_eq!(parse("\\di"), Some(SlashCommand::ListIndexes));
    }

    #[test]
    fn test_parse_list_functions() {
        assert_eq!(parse("\\df"), Some(SlashCommand::ListFunctions));
    }

    #[test]
    fn test_parse_list_schemas() {
        assert_eq!(parse("\\ds"), Some(SlashCommand::ListSchemas));
    }

    #[test]
    fn test_parse_list_databases() {
        assert_eq!(parse("\\dn"), Some(SlashCommand::ListDatabases));
    }

    #[test]
    fn test_parse_use_database() {
        assert_eq!(
            parse("\\c mydb"),
            Some(SlashCommand::UseDatabase("mydb".to_string()))
        );
    }

    #[test]
    fn test_parse_use_database_no_arg() {
        assert_eq!(parse("\\c"), None);
    }

    #[test]
    fn test_parse_conninfo() {
        assert_eq!(parse("\\conninfo"), Some(SlashCommand::ConnInfo));
    }

    #[test]
    fn test_parse_toggle_expanded() {
        assert_eq!(parse("\\x"), Some(SlashCommand::ToggleExpanded));
    }

    #[test]
    fn test_parse_toggle_timing() {
        assert_eq!(parse("\\timing"), Some(SlashCommand::ToggleTiming));
    }

    #[test]
    fn test_parse_help() {
        assert_eq!(parse("\\?"), Some(SlashCommand::Help));
    }

    #[test]
    fn test_parse_quit() {
        assert_eq!(parse("\\q"), Some(SlashCommand::Quit));
    }

    #[test]
    fn test_parse_not_slash_command() {
        assert_eq!(parse("SELECT 1"), None);
    }

    #[test]
    fn test_parse_unknown_command() {
        assert_eq!(parse("\\zzz"), None);
    }

    #[test]
    fn test_to_action_list_all_sql() {
        let cmd = SlashCommand::ListAll;
        let action = to_action(&cmd, "", "", "");
        match action {
            CommandAction::ExecuteSql(sql) => {
                assert!(sql.contains("INFORMATION_SCHEMA.TABLES"));
                assert!(!sql.contains("WHERE"));
            }
            _ => panic!("expected ExecuteSql"),
        }
    }

    #[test]
    fn test_to_action_describe_sql() {
        let action = to_action(&SlashCommand::Describe("users".to_string()), "", "", "");
        match action {
            CommandAction::ExecuteSql(sql) => {
                assert!(sql.contains("INFORMATION_SCHEMA.COLUMNS"));
                assert!(sql.contains("'users'"));
            }
            _ => panic!("expected ExecuteSql"),
        }
    }

    #[test]
    fn test_to_action_describe_sql_injection() {
        let action = to_action(&SlashCommand::Describe("a'b".to_string()), "", "", "");
        match action {
            CommandAction::ExecuteSql(sql) => {
                assert!(sql.contains("'a''b'"));
            }
            _ => panic!("expected ExecuteSql"),
        }
    }

    #[test]
    fn test_to_action_conninfo() {
        let action = to_action(&SlashCommand::ConnInfo, "localhost:1433", "mydb", "sa");
        match action {
            CommandAction::DisplayMessage { columns, rows } => {
                assert_eq!(columns, vec!["Property", "Value"]);
                assert_eq!(rows[0], vec!["Server", "localhost:1433"]);
                assert_eq!(rows[1], vec!["Database", "mydb"]);
                assert_eq!(rows[2], vec!["User", "sa"]);
            }
            _ => panic!("expected DisplayMessage"),
        }
    }

    #[test]
    fn test_to_action_help() {
        let action = to_action(&SlashCommand::Help, "", "", "");
        match action {
            CommandAction::DisplayMessage { columns, rows } => {
                assert_eq!(columns[0], "Command");
                assert!(rows.len() >= 13);
            }
            _ => panic!("expected DisplayMessage"),
        }
    }
}
