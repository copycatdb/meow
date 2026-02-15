//! Query execution and result formatting.

use crate::app::{ObjectNode, QueryResult, ResultSet};
use crate::db::ConnectionHandle;
use claw::{ResultItem, SqlValue};
use futures_util::TryStreamExt;
use std::time::Instant;

/// Execute a SQL query and return structured results.
pub async fn execute_query(
    client: &mut ConnectionHandle,
    sql: &str,
) -> Result<QueryResult, Box<dyn std::error::Error>> {
    let start = Instant::now();

    let mut stream = client.execute(sql, &[]).await?;

    let mut result_sets = Vec::new();
    let mut current_columns: Vec<String> = Vec::new();
    let mut current_rows: Vec<Vec<String>> = Vec::new();

    while let Some(item) = stream.try_next().await? {
        match item {
            ResultItem::Metadata(schema) => {
                // Save previous resultset if it had rows or columns
                if !current_columns.is_empty() || !current_rows.is_empty() {
                    result_sets.push(ResultSet {
                        columns: std::mem::take(&mut current_columns),
                        rows: std::mem::take(&mut current_rows),
                    });
                }
                current_columns = schema
                    .columns()
                    .iter()
                    .map(|c| c.name().to_string())
                    .collect();
            }
            ResultItem::Row(row) => {
                // If we haven't seen metadata yet, get columns from the row
                if current_columns.is_empty() {
                    current_columns = row.columns().iter().map(|c| c.name().to_string()).collect();
                }
                let vals: Vec<String> = row.into_iter().map(|val| format_sql_value(&val)).collect();
                current_rows.push(vals);
            }
            ResultItem::Message(_) => {} // skip info messages
        }
    }

    // Don't forget the last resultset
    if !current_columns.is_empty() || !current_rows.is_empty() {
        result_sets.push(ResultSet {
            columns: current_columns,
            rows: current_rows,
        });
    }

    let elapsed_ms = start.elapsed().as_millis();

    Ok(QueryResult {
        result_sets,
        elapsed_ms,
        error: None,
    })
}

/// Format a SqlValue into a display string.
fn format_sql_value(val: &SqlValue<'_>) -> String {
    match val {
        SqlValue::U8(Some(n)) => n.to_string(),
        SqlValue::U8(None) => "NULL".to_string(),
        SqlValue::I16(Some(n)) => n.to_string(),
        SqlValue::I16(None) => "NULL".to_string(),
        SqlValue::I32(Some(n)) => n.to_string(),
        SqlValue::I32(None) => "NULL".to_string(),
        SqlValue::I64(Some(n)) => n.to_string(),
        SqlValue::I64(None) => "NULL".to_string(),
        SqlValue::F32(Some(n)) => n.to_string(),
        SqlValue::F32(None) => "NULL".to_string(),
        SqlValue::F64(Some(n)) => n.to_string(),
        SqlValue::F64(None) => "NULL".to_string(),
        SqlValue::Bit(Some(b)) => b.to_string(),
        SqlValue::Bit(None) => "NULL".to_string(),
        SqlValue::String(Some(s)) => s.to_string(),
        SqlValue::String(None) => "NULL".to_string(),
        SqlValue::Guid(Some(g)) => g.to_string(),
        SqlValue::Guid(None) => "NULL".to_string(),
        SqlValue::Binary(Some(b)) => format!("0x{}", hex_encode(b)),
        SqlValue::Binary(None) => "NULL".to_string(),
        SqlValue::Numeric(Some(n)) => format!("{}", n),
        SqlValue::Numeric(None) => "NULL".to_string(),
        SqlValue::Xml(Some(x)) => format!("{:?}", x),
        SqlValue::Xml(None) => "NULL".to_string(),
        SqlValue::DateTime(Some(dt)) => {
            // Days since 1900-01-01, seconds_fragments in 1/300s
            let unix_days = -25567i64 + dt.days() as i64;
            let (year, month, day) = days_to_ymd(unix_days);
            let total_secs = dt.seconds_fragments() as f64 / 300.0;
            let hours = (total_secs / 3600.0) as u32;
            let mins = ((total_secs % 3600.0) / 60.0) as u32;
            let secs = (total_secs % 60.0) as u32;
            format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                year, month, day, hours, mins, secs
            )
        }
        SqlValue::DateTime(None) => "NULL".to_string(),
        SqlValue::SmallDateTime(Some(dt)) => {
            let unix_days = -25567i64 + dt.days() as i64;
            let (year, month, day) = days_to_ymd(unix_days);
            let total_secs = dt.seconds_fragments() as f64 / 300.0;
            let hours = (total_secs / 3600.0) as u32;
            let mins = ((total_secs % 3600.0) / 60.0) as u32;
            format!(
                "{:04}-{:02}-{:02} {:02}:{:02}",
                year, month, day, hours, mins
            )
        }
        SqlValue::SmallDateTime(None) => "NULL".to_string(),
        SqlValue::Date(Some(d)) => {
            let (year, month, day) = days_to_ymd(d.days() as i64 - 719163);
            format!("{:04}-{:02}-{:02}", year, month, day)
        }
        SqlValue::Date(None) => "NULL".to_string(),
        SqlValue::Time(Some(t)) => {
            let nanos = t.increments() as f64 * 10f64.powi(9 - t.scale() as i32);
            let total_secs = (nanos / 1_000_000_000.0) as u64;
            let hours = total_secs / 3600;
            let mins = (total_secs % 3600) / 60;
            let secs = total_secs % 60;
            let frac = (nanos % 1_000_000_000.0) as u64;
            if frac > 0 {
                format!("{:02}:{:02}:{:02}.{:07}", hours, mins, secs, frac / 100)
            } else {
                format!("{:02}:{:02}:{:02}", hours, mins, secs)
            }
        }
        SqlValue::Time(None) => "NULL".to_string(),
        SqlValue::DateTime2(Some(dt2)) => {
            let (year, month, day) = days_to_ymd(dt2.date().days() as i64 - 719163);
            let t = dt2.time();
            let nanos = t.increments() as f64 * 10f64.powi(9 - t.scale() as i32);
            let total_secs = (nanos / 1_000_000_000.0) as u64;
            let hours = total_secs / 3600;
            let mins = (total_secs % 3600) / 60;
            let secs = total_secs % 60;
            let frac = (nanos % 1_000_000_000.0) as u64;
            if frac > 0 {
                format!(
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:07}",
                    year,
                    month,
                    day,
                    hours,
                    mins,
                    secs,
                    frac / 100
                )
            } else {
                format!(
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                    year, month, day, hours, mins, secs
                )
            }
        }
        SqlValue::DateTime2(None) => "NULL".to_string(),
        SqlValue::DateTimeOffset(Some(dto)) => {
            let dt2 = dto.datetime2();
            let (year, month, day) = days_to_ymd(dt2.date().days() as i64 - 719163);
            let t = dt2.time();
            let nanos = t.increments() as f64 * 10f64.powi(9 - t.scale() as i32);
            let total_secs = (nanos / 1_000_000_000.0) as u64;
            let hours = total_secs / 3600;
            let mins = (total_secs % 3600) / 60;
            let secs = total_secs % 60;
            let offset_mins = dto.offset();
            let sign = if offset_mins >= 0 { '+' } else { '-' };
            let abs_offset = offset_mins.unsigned_abs();
            format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02} {}{:02}:{:02}",
                year,
                month,
                day,
                hours,
                mins,
                secs,
                sign,
                abs_offset / 60,
                abs_offset % 60
            )
        }
        SqlValue::DateTimeOffset(None) => "NULL".to_string(),
        other => format!("{:?}", other),
    }
}

/// Convert days since Unix epoch (1970-01-01) to (year, month, day).
/// Uses Howard Hinnant's civil calendar algorithm.
fn days_to_ymd(z: i64) -> (i64, u32, u32) {
    let z = z + 719468; // shift to 0000-03-01 epoch
    let era = if z >= 0 {
        z / 146097
    } else {
        (z - 146096) / 146097
    };
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m as u32, d as u32)
}

/// Simple hex encoding for binary data.
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02X}", b)).collect()
}

/// Fetch the object tree (databases → schemas → tables) from SQL Server.
pub async fn fetch_object_tree(
    client: &mut ConnectionHandle,
) -> Result<Vec<ObjectNode>, Box<dyn std::error::Error>> {
    // Get databases
    let stream = client
        .execute("SELECT name FROM sys.databases ORDER BY name", &[])
        .await?;
    let db_rows = stream.into_first_result().await?;

    let mut databases = Vec::new();
    for row in &db_rows {
        let db_name: &str = row.get(0usize).unwrap_or("?");
        databases.push(ObjectNode {
            name: db_name.to_string(),
            depth: 0,
            expanded: false,
            children: Vec::new(),
        });
    }

    // For the current database, pre-load schemas and tables
    if let Some(db) = databases.iter_mut().find(|d| d.name == "master") {
        load_schemas_and_tables(client, db).await.ok();
    }

    Ok(databases)
}

/// Load schemas and tables for a specific database node.
pub async fn load_schemas_and_tables(
    client: &mut ConnectionHandle,
    db_node: &mut ObjectNode,
) -> Result<(), Box<dyn std::error::Error>> {
    let sql = format!(
        "SELECT TABLE_SCHEMA, TABLE_NAME FROM {}.INFORMATION_SCHEMA.TABLES ORDER BY TABLE_SCHEMA, TABLE_NAME",
        db_node.name
    );
    let stream = client.execute(&sql, &[]).await?;
    let rows = stream.into_first_result().await?;

    // Group by schema
    let mut schemas: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for row in &rows {
        let schema: &str = row.get(0usize).unwrap_or("dbo");
        let table: &str = row.get(1usize).unwrap_or("?");
        schemas
            .entry(schema.to_string())
            .or_default()
            .push(table.to_string());
    }

    db_node.children = schemas
        .into_iter()
        .map(|(schema, tables)| ObjectNode {
            name: schema,
            depth: 1,
            expanded: false,
            children: tables
                .into_iter()
                .map(|t| ObjectNode {
                    name: t,
                    depth: 2,
                    expanded: false,
                    children: Vec::new(),
                })
                .collect(),
        })
        .collect();

    Ok(())
}
