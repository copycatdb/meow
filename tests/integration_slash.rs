//! Integration tests for slash commands against a real SQL Server.
//! Requires SQL Server running on localhost:1433 with sa/TestPass123!

use std::time::Instant;

/// Helper: connect to SQL Server and run a query, returning (columns, rows).
async fn run_query(
    client: &mut claw::TcpClient,
    sql: &str,
) -> Result<(Vec<String>, Vec<Vec<String>>), Box<dyn std::error::Error>> {
    let stream = client.execute(sql, &[]).await?;
    let mut stream = stream;
    let cols = stream.columns().await?;
    let columns: Vec<String> = match cols {
        Some(c) => c.iter().map(|col| col.name().to_string()).collect(),
        None => Vec::new(),
    };
    let rows = stream.into_first_result().await?;
    let result_rows: Vec<Vec<String>> = rows
        .into_iter()
        .map(|row| row.into_iter().map(|val| format!("{:?}", val)).collect())
        .collect();
    Ok((columns, result_rows))
}

async fn connect() -> Result<claw::TcpClient, Box<dyn std::error::Error>> {
    let mut config = claw::Config::new();
    config.host("localhost");
    config.port(1433);
    config.authentication(claw::AuthMethod::sql_server("sa", "TestPass123!"));
    config.database("master");
    config.trust_cert();
    Ok(claw::connect(config).await?)
}

#[tokio::test]
async fn test_slash_d_list_all() {
    let Ok(mut client) = connect().await else {
        eprintln!("Skipping: SQL Server not available");
        return;
    };

    let cmd = meow::commands::parse("\\d").unwrap();
    let action = meow::commands::to_action(&cmd, "", "", "");
    if let meow::commands::CommandAction::ExecuteSql(sql) = action {
        let (cols, _rows) = run_query(&mut client, &sql).await.unwrap();
        assert!(cols.contains(&"TABLE_SCHEMA".to_string()));
        assert!(cols.contains(&"TABLE_NAME".to_string()));
        assert!(cols.contains(&"TABLE_TYPE".to_string()));
    }
}

#[tokio::test]
async fn test_slash_dt_list_tables() {
    let Ok(mut client) = connect().await else {
        return;
    };
    // Create a test table
    let _ = run_query(
        &mut client,
        "IF OBJECT_ID('dbo.__meow_test', 'U') IS NOT NULL DROP TABLE dbo.__meow_test",
    )
    .await;
    let _ = run_query(
        &mut client,
        "CREATE TABLE dbo.__meow_test (id INT PRIMARY KEY, name NVARCHAR(100))",
    )
    .await;

    let cmd = meow::commands::parse("\\dt").unwrap();
    if let meow::commands::CommandAction::ExecuteSql(sql) =
        meow::commands::to_action(&cmd, "", "", "")
    {
        let (cols, rows) = run_query(&mut client, &sql).await.unwrap();
        assert!(cols.contains(&"TABLE_NAME".to_string()));
        assert!(rows.len() > 0);
    }

    let _ = run_query(&mut client, "DROP TABLE dbo.__meow_test").await;
}

#[tokio::test]
async fn test_slash_d_describe_table() {
    let Ok(mut client) = connect().await else {
        return;
    };
    let _ = run_query(
        &mut client,
        "IF OBJECT_ID('dbo.__meow_test2', 'U') IS NOT NULL DROP TABLE dbo.__meow_test2",
    )
    .await;
    let _ = run_query(
        &mut client,
        "CREATE TABLE dbo.__meow_test2 (id INT NOT NULL, name NVARCHAR(50) NULL)",
    )
    .await;

    let cmd = meow::commands::parse("\\d __meow_test2").unwrap();
    if let meow::commands::CommandAction::ExecuteSql(sql) =
        meow::commands::to_action(&cmd, "", "", "")
    {
        let (cols, rows) = run_query(&mut client, &sql).await.unwrap();
        assert!(cols.contains(&"COLUMN_NAME".to_string()));
        assert!(cols.contains(&"DATA_TYPE".to_string()));
        assert_eq!(rows.len(), 2);
    }

    let _ = run_query(&mut client, "DROP TABLE dbo.__meow_test2").await;
}

#[tokio::test]
async fn test_slash_dn_list_databases() {
    let Ok(mut client) = connect().await else {
        return;
    };
    let cmd = meow::commands::parse("\\dn").unwrap();
    if let meow::commands::CommandAction::ExecuteSql(sql) =
        meow::commands::to_action(&cmd, "", "", "")
    {
        let (cols, rows) = run_query(&mut client, &sql).await.unwrap();
        assert!(cols.contains(&"name".to_string()));
        assert!(rows.len() > 0);
    }
}

#[tokio::test]
async fn test_slash_ds_list_schemas() {
    let Ok(mut client) = connect().await else {
        return;
    };
    let cmd = meow::commands::parse("\\ds").unwrap();
    if let meow::commands::CommandAction::ExecuteSql(sql) =
        meow::commands::to_action(&cmd, "", "", "")
    {
        let (cols, rows) = run_query(&mut client, &sql).await.unwrap();
        assert!(cols.contains(&"name".to_string()));
        assert!(rows.len() > 0);
    }
}
