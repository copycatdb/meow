//! Query execution and result formatting.

use crate::app::{ObjectNode, QueryResult};
use crate::db::ConnectionHandle;
use std::time::Instant;
use tabby::SqlValue;

/// Execute a SQL query and return structured results.
pub async fn execute_query(
    client: &mut ConnectionHandle,
    sql: &str,
) -> Result<QueryResult, Box<dyn std::error::Error>> {
    let start = Instant::now();

    let stream = client.execute(sql, &[]).await?;

    // Get columns
    let mut stream = stream;
    let cols = stream.columns().await?;
    let columns: Vec<String> = match cols {
        Some(c) => c.iter().map(|col| col.name().to_string()).collect(),
        None => Vec::new(),
    };

    // Collect rows
    let result_rows = stream.into_first_result().await?;
    let rows: Vec<Vec<String>> = result_rows
        .into_iter()
        .map(|row| row.into_iter().map(|val| format_sql_value(&val)).collect())
        .collect();

    let elapsed_ms = start.elapsed().as_millis();

    Ok(QueryResult {
        columns,
        rows,
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
        other => format!("{:?}", other),
    }
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
