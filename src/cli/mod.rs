//! Non-interactive CLI mode for scripting and piped input.

use crate::Args;
use crate::db;
use std::io::{self, BufRead, Write};

/// Run meow in CLI mode.
pub async fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let (host, port) = args.parse_server();
    let user = args.user.as_deref().unwrap_or("sa");
    let password = args.password.as_deref().unwrap_or("");

    let mut client =
        db::connect(&host, port, user, password, &args.database, args.trust_cert).await?;

    // Determine SQL source
    let sql = if let Some(ref input_file) = args.input {
        std::fs::read_to_string(input_file)?
    } else if !std::io::stdin().is_terminal() {
        // Read from stdin pipe
        let mut buf = String::new();
        io::stdin().lock().read_to_string(&mut buf)?;
        buf
    } else {
        // Interactive CLI mode — read line by line
        return run_interactive(&mut client, &args).await;
    };

    // Execute and output
    execute_and_print(&mut client, &sql, &args).await?;
    Ok(())
}

/// Run interactive CLI (line-by-line REPL).
async fn run_interactive(
    client: &mut db::ConnectionHandle,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("meow> ");
        stdout.flush()?;

        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            break; // EOF
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.eq_ignore_ascii_case("quit") || trimmed.eq_ignore_ascii_case("exit") {
            break;
        }

        execute_and_print(client, trimmed, args).await.ok();
    }

    Ok(())
}

/// Execute a SQL statement and print results.
async fn execute_and_print(
    client: &mut db::ConnectionHandle,
    sql: &str,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = db::query::execute_query(client, sql).await?;

    let output: Box<dyn Write> = if let Some(ref path) = args.output {
        Box::new(std::fs::File::create(path)?)
    } else {
        Box::new(io::stdout())
    };
    let mut writer = io::BufWriter::new(output);

    match args.format.as_str() {
        "csv" => print_csv(&mut writer, &result)?,
        "json" => print_json(&mut writer, &result)?,
        _ => print_table(&mut writer, &result)?,
    }

    Ok(())
}

/// Print results as an ASCII table.
fn print_table(
    writer: &mut dyn Write,
    result: &crate::app::QueryResult,
) -> Result<(), Box<dyn std::error::Error>> {
    if result.columns.is_empty() {
        return Ok(());
    }

    // Calculate column widths
    let widths: Vec<usize> = result
        .columns
        .iter()
        .enumerate()
        .map(|(i, col)| {
            let max_data = result
                .rows
                .iter()
                .map(|r| r.get(i).map(|s| s.len()).unwrap_or(0))
                .max()
                .unwrap_or(0);
            col.len().max(max_data)
        })
        .collect();

    // Header
    let header: Vec<String> = result
        .columns
        .iter()
        .zip(&widths)
        .map(|(c, w)| format!("{:<width$}", c, width = w))
        .collect();
    writeln!(writer, "{}", header.join(" | "))?;

    // Separator
    let sep: Vec<String> = widths.iter().map(|w| "-".repeat(*w)).collect();
    writeln!(writer, "{}", sep.join("-+-"))?;

    // Data rows
    for row in &result.rows {
        let cells: Vec<String> = row
            .iter()
            .zip(&widths)
            .map(|(val, w)| format!("{:<width$}", val, width = w))
            .collect();
        writeln!(writer, "{}", cells.join(" | "))?;
    }

    writeln!(
        writer,
        "\n({} rows, {}ms)",
        result.rows.len(),
        result.elapsed_ms
    )?;

    Ok(())
}

/// Print results as CSV.
fn print_csv(
    writer: &mut dyn Write,
    result: &crate::app::QueryResult,
) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(writer, "{}", result.columns.join(","))?;
    for row in &result.rows {
        let escaped: Vec<String> = row
            .iter()
            .map(|v| {
                if v.contains(',') || v.contains('"') || v.contains('\n') {
                    format!("\"{}\"", v.replace('"', "\"\""))
                } else {
                    v.clone()
                }
            })
            .collect();
        writeln!(writer, "{}", escaped.join(","))?;
    }
    Ok(())
}

/// Print results as JSON.
fn print_json(
    writer: &mut dyn Write,
    result: &crate::app::QueryResult,
) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(writer, "[")?;
    for (i, row) in result.rows.iter().enumerate() {
        write!(writer, "  {{")?;
        for (j, (col, val)) in result.columns.iter().zip(row).enumerate() {
            write!(
                writer,
                "\"{}\": \"{}\"",
                col,
                val.replace('\\', "\\\\").replace('"', "\\\"")
            )?;
            if j + 1 < result.columns.len() {
                write!(writer, ", ")?;
            }
        }
        write!(writer, "}}")?;
        if i + 1 < result.rows.len() {
            writeln!(writer, ",")?;
        } else {
            writeln!(writer)?;
        }
    }
    writeln!(writer, "]")?;
    Ok(())
}

/// Helper trait — re-export for stdin detection.
use std::io::IsTerminal;
use std::io::Read;
