//! This is a module that contains a high-level Matrix client

use matrix_sdk::{Client, ClientConfig, SyncSettings};
use url::Url;

use crate::{Config, Error};

pub struct MatrixClient {
    /// The inner, slightly lower-level Matrix client
    inner: Client,
    config: Config,
}

impl MatrixClient {
    /// Creates a new MatrixClient with a given `config`
    pub fn with_config(config: Config) -> Result<MatrixClient, Error> {
        let client_config = ClientConfig::new();

        let homeserver_url =
            Url::parse(&config.matrix.homeserver).map_err(Error::HomeserverParseError)?;

        let client = Client::new_with_config(homeserver_url, client_config)?;

        Ok(MatrixClient {
            inner: client,
            config,
        })
    }

    pub async fn login_and_sync(&self) -> Result<(), Error> {
        let client = &self.inner;

        client
            .login(
                &self.config.matrix.username,
                &self.config.matrix.password,
                None,
                Some("rust-sdk"),
            )
            .await?;

        // Sync to skip old messages
        client.sync(SyncSettings::default()).await?;

        // Sync forever with our stored token
        let settings = SyncSettings::default().token(client.sync_token().await.unwrap());
        client.sync_forever(settings, |_| async {}).await;

        Ok(())
    }
}
