use std::env;

use anyhow::Context;
use log::debug;

mod client;
mod config;
mod error;

use client::MatrixClient;
pub use config::Config;
pub use error::Error;

/// Returns the path to the config file to use
///
/// It will return the env variable `META_MATRIX_CONFIG_FILE` if defined, otherwise just
/// `config.toml` in cwd
fn config_file_path() -> String {
    match env::var("META_MATRIX_CONFIG_FILE") {
        Ok(path) => path.to_string(),
        Err(_) => "config.toml".to_string(),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Load the config
    let config_path = config_file_path();
    let config = config::load(&config_path)
        .with_context(|| format!("failed to load config file `{}'", config_path))?;

    debug!(
        "Logging in as {} on homeserver {}",
        &config.matrix.username, &config.matrix.homeserver
    );

    let client = MatrixClient::with_config(config)?;

    client.login().await?;
    client.poll().await?;

    Ok(())
}
