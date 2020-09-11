//! This is the module for the user configurations

use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::Error;

/// The root config struct
#[derive(Debug, Deserialize)]
pub struct Config {
    /// The matrix-specific configuration
    pub matrix: MatrixConfig,
}

/// The matrix-specific configuration
#[derive(Debug, Deserialize)]
pub struct MatrixConfig {
    /// The homeserver URL to connect to
    pub homeserver: String,
    /// The username to use when authenticating
    pub username: String,
    /// The password to use when authenticating
    pub password: String,
    /// A list of rooms to join
    ///
    /// NOTE: this has to be room ids and not aliases
    pub rooms: Vec<String>,
}

pub fn load<P: AsRef<Path>>(path: P) -> Result<Config, Error> {
    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;

    Ok(config)
}
