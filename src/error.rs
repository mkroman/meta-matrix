use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    /// An error occurred when parsing the config file
    #[error("unable to parse config")]
    ConfigError(#[from] toml::de::Error),

    /// An I/O error
    #[error("An I/O error occurred")]
    IoError(#[from] io::Error),

    #[error("matrix error")]
    MatrixError(#[from] matrix_sdk::Error),

    #[error("unable to parse homeserver url")]
    HomeserverParseError(#[from] url::ParseError),
}
