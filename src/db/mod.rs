//! Database connection management and query execution.

pub mod query;

use claw::{AuthMethod, Config, TcpClient};

/// A handle wrapping the claw client.
pub type ConnectionHandle = TcpClient;

/// Connect to SQL Server using the given parameters.
pub async fn connect(
    host: &str,
    port: u16,
    user: &str,
    password: &str,
    database: &str,
    trust_cert: bool,
) -> Result<ConnectionHandle, Box<dyn std::error::Error>> {
    let mut config = Config::new();
    config.host(host);
    config.port(port);
    config.authentication(AuthMethod::sql_server(user, password));
    config.database(database);

    if trust_cert {
        config.trust_cert();
    }

    let client = claw::connect(config).await?;
    Ok(client)
}
