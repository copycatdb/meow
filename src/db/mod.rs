//! Database connection management and query execution.

pub mod query;

use claw::{AuthMethod, Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

/// A handle wrapping the tabby client.
pub type ConnectionHandle = Client<Compat<TcpStream>>;

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

    let addr = config.get_addr();
    let tcp = TcpStream::connect(&addr).await?;
    tcp.set_nodelay(true)?;

    let client = Client::connect(config, tcp.compat_write()).await?;
    Ok(client)
}
