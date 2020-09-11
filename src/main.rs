use anyhow::Context;
use log::debug;

mod client;
mod config;
mod error;

use client::MatrixClient;
pub use config::Config;
pub use error::Error;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Load the config
    let config_path = "config.toml";
    let config = config::load(config_path)
        .with_context(|| format!("failed to load config file `{}'", config_path))?;

    debug!(
        "Logging in as {} on homeserver {}",
        &config.matrix.username, &config.matrix.homeserver
    );

    let client = MatrixClient::with_config(config)?;

    client.login_and_sync().await?;

    Ok(())
}
