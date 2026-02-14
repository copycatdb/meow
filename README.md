# 🐱 meow

A beautiful TUI client for Microsoft SQL Server, powered by [tabby](https://github.com/copycatdb/tabby).

> Think `pgcli` meets `lazygit` — fast, cross-platform, single binary.

Part of the [CopyCat](https://github.com/copycatdb) ecosystem.

![screenshot placeholder](https://via.placeholder.com/800x500?text=meow+TUI+screenshot+coming+soon)

## Installation

Build from source (requires Rust 1.85+):

```bash
git clone https://github.com/copycatdb/meow.git
cd meow
cargo build --release
# Binary at ./target/release/meow
```

## Usage

### TUI Mode (default)

```bash
meow -S localhost,1433 -U sa -P yourpassword --trust-cert
```

This launches the interactive TUI with three panes: object browser, SQL editor, and results.

### CLI Mode

```bash
# Interactive REPL
meow --cli -S localhost,1433 -U sa -P yourpassword --trust-cert

# Pipe a query
echo "SELECT 1 AS test" | meow -S localhost,1433 -U sa -P yourpassword --trust-cert

# Execute from file
meow -S localhost,1433 -U sa -P yourpassword --trust-cert -i query.sql

# Output as CSV
meow -S localhost,1433 -U sa -P yourpassword --trust-cert -i query.sql --format csv

# Output as JSON
echo "SELECT name FROM sys.databases" | meow -S localhost,1433 -U sa -P yourpassword --trust-cert --format json
```

## Options

| Flag | Description | Default |
|------|-------------|---------|
| `-S, --server` | Server address (`host,port`) | `localhost,1433` |
| `-U, --user` | SQL login username | — |
| `-P, --password` | SQL login password | — |
| `-d, --database` | Initial database | `master` |
| `--trust-cert` | Trust server certificate | off |
| `--cli` | Non-interactive CLI mode | off |
| `-i, --input` | Execute SQL from file | — |
| `-o, --output` | Write results to file | — |
| `--format` | Output format: `table`, `csv`, `json` | `table` |

## Key Bindings

| Key | Action |
|-----|--------|
| `Ctrl+Enter` / `F5` | Execute query |
| `Tab` | Cycle focus: Editor → Results → Sidebar |
| `Ctrl+D` | Toggle sidebar (object browser) |
| `Ctrl+L` | Clear editor |
| `Ctrl+Q` | Quit |
| `F1` | Toggle help overlay |
| `↑/↓` | Scroll results (when focused) |
| `Enter` | Expand/collapse sidebar node |

## Architecture

```
src/
├── main.rs          — entry point, CLI args, mode dispatch
├── app.rs           — App state machine
├── tui/
│   ├── mod.rs       — TUI setup/teardown, event loop
│   ├── ui.rs        — layout and rendering
│   ├── editor.rs    — SQL editor pane
│   ├── results.rs   — result grid/table pane
│   ├── sidebar.rs   — object browser
│   └── statusbar.rs — connection info, timing
├── db/
│   ├── mod.rs       — connection management
│   └── query.rs     — query execution, result formatting
└── cli/
    └── mod.rs       — non-interactive CLI mode
```

## License

MIT
