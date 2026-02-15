//! TUI setup, teardown, and main event loop.

pub mod autocomplete;
pub mod editor;
pub mod results;
pub mod sidebar;
pub mod statusbar;
pub mod ui;

use crate::Args;
use crate::app::{App, FocusPane};
use crate::commands;
use crate::db;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use std::io;

/// Run the TUI application.
pub async fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let (host, port) = args.parse_server();
    let user = args.user.as_deref().unwrap_or("sa");
    let password = args.password.as_deref().unwrap_or("");

    // Connect to SQL Server
    let mut client =
        db::connect(&host, port, user, password, &args.database, args.trust_cert).await?;

    // Initialize app state
    let mut app = App::new(&host, port, &args.database, user);

    // Load object tree
    app.load_objects(&mut client).await;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main event loop
    let result = run_loop(&mut terminal, &mut app, &mut client).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

/// The main TUI event loop.
async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    client: &mut db::ConnectionHandle,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        // Draw UI
        terminal.draw(|frame| ui::draw(frame, app))?;

        // Poll for events with a timeout so we can do async work
        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && handle_key(key, app, client).await?
        {
            break;
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

/// Handle a key event. Returns true if the app should exit.
async fn handle_key(
    key: KeyEvent,
    app: &mut App,
    client: &mut db::ConnectionHandle,
) -> Result<bool, Box<dyn std::error::Error>> {
    // Global keys
    match (key.modifiers, key.code) {
        // Ctrl+Q — quit
        (KeyModifiers::CONTROL, KeyCode::Char('q')) => return Ok(true),
        // F1 — toggle help
        (_, KeyCode::F(1)) => {
            app.show_help = !app.show_help;
            return Ok(false);
        }
        // Tab — cycle focus
        (KeyModifiers::NONE, KeyCode::Tab) => {
            app.cycle_focus();
            return Ok(false);
        }
        // Ctrl+D — toggle sidebar
        (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
            app.toggle_sidebar();
            return Ok(false);
        }
        // Ctrl+L — clear editor
        (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
            app.clear_editor();
            return Ok(false);
        }
        // Ctrl+Enter or F5 — execute query
        (KeyModifiers::CONTROL, KeyCode::Enter) | (_, KeyCode::F(5)) => {
            let sql = app.get_editor_text();
            if !sql.trim().is_empty() {
                app.push_history();
                // Check for slash commands
                if let Some(cmd) = commands::parse(&sql) {
                    let action = commands::to_action(
                        &cmd,
                        &app.connection_info,
                        &app.current_database,
                        &app.user,
                    );
                    match action {
                        commands::CommandAction::ExecuteSql(query) => {
                            app.query_running = true;
                            match db::query::execute_query(client, &query).await {
                                Ok(result) => {
                                    // If it was a USE command, update current database
                                    if let commands::SlashCommand::UseDatabase(ref db_name) = cmd {
                                        app.current_database = db_name.clone();
                                    }
                                    app.result = result;
                                    app.result_scroll = 0;
                                    app.result_col_scroll = 0;
                                }
                                Err(e) => {
                                    app.result = crate::app::QueryResult {
                                        error: Some(e.to_string()),
                                        ..Default::default()
                                    };
                                }
                            }
                            app.query_running = false;
                        }
                        commands::CommandAction::DisplayMessage { columns, rows } => {
                            app.result = crate::app::QueryResult {
                                columns,
                                rows,
                                elapsed_ms: 0,
                                error: None,
                            };
                            app.result_scroll = 0;
                            app.result_col_scroll = 0;
                        }
                        commands::CommandAction::ToggleExpanded => {
                            app.expanded_mode = !app.expanded_mode;
                            let state = if app.expanded_mode { "ON" } else { "OFF" };
                            app.result = crate::app::QueryResult {
                                columns: vec!["Status".to_string()],
                                rows: vec![vec![format!("Expanded display is {}", state)]],
                                elapsed_ms: 0,
                                error: None,
                            };
                        }
                        commands::CommandAction::ToggleTiming => {
                            app.show_timing = !app.show_timing;
                            let state = if app.show_timing { "ON" } else { "OFF" };
                            app.result = crate::app::QueryResult {
                                columns: vec!["Status".to_string()],
                                rows: vec![vec![format!("Timing is {}", state)]],
                                elapsed_ms: 0,
                                error: None,
                            };
                        }
                        commands::CommandAction::Quit => return Ok(true),
                    }
                } else {
                    app.query_running = true;
                    match db::query::execute_query(client, &sql).await {
                        Ok(result) => {
                            app.result = result;
                            app.result_scroll = 0;
                            app.result_col_scroll = 0;
                        }
                        Err(e) => {
                            app.result = crate::app::QueryResult {
                                error: Some(e.to_string()),
                                ..Default::default()
                            };
                        }
                    }
                    app.query_running = false;
                }
            }
            return Ok(false);
        }
        _ => {}
    }

    // Pane-specific keys
    match app.focus {
        FocusPane::Editor => {
            // If autocomplete is active, intercept navigation keys
            if app.autocomplete.active {
                match key.code {
                    KeyCode::Esc => {
                        app.autocomplete.dismiss();
                        return Ok(false);
                    }
                    KeyCode::Up => {
                        app.autocomplete.prev();
                        return Ok(false);
                    }
                    KeyCode::Down => {
                        app.autocomplete.next();
                        return Ok(false);
                    }
                    KeyCode::Tab | KeyCode::Enter => {
                        // Accept selected suggestion
                        if let Some(keyword) = app.autocomplete.selected_keyword() {
                            let prefix_len = app.autocomplete.prefix.len();
                            // Delete the prefix characters by sending backspaces
                            for _ in 0..prefix_len {
                                app.editor
                                    .input(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
                            }
                            // Insert the keyword character by character
                            for ch in keyword.chars() {
                                app.editor
                                    .input(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
                            }
                        }
                        app.autocomplete.dismiss();
                        return Ok(false);
                    }
                    _ => {
                        // Pass through to editor, then update autocomplete below
                    }
                }
            }
            // Let tui-textarea handle input
            app.editor.input(key);
            // Update autocomplete after keystroke
            let cursor = app.editor.cursor();
            let lines: Vec<String> = app.editor.lines().iter().map(|s| s.to_string()).collect();
            app.autocomplete.update(&lines, cursor.0, cursor.1);
        }
        FocusPane::Results => match key.code {
            KeyCode::Up => app.scroll_results_up(),
            KeyCode::Down => app.scroll_results_down(),
            KeyCode::Left => app.scroll_results_left(),
            KeyCode::Right => app.scroll_results_right(),
            _ => {}
        },
        FocusPane::Sidebar => match key.code {
            KeyCode::Up => app.scroll_sidebar_up(),
            KeyCode::Down => app.scroll_sidebar_down(),
            KeyCode::Enter => app.toggle_sidebar_node(),
            _ => {}
        },
    }

    Ok(false)
}
