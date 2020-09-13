use async_trait::async_trait;
use reqwest::Client;

use crate::plugin::Plugin;
use crate::Error;

pub struct GoogleSearchPlugin {
    http_client: Client,
}

impl GoogleSearchPlugin {
    pub fn new() -> Result<Self, Error> {
        Ok(GoogleSearchPlugin {
            http_client: Client::new(),
        })
    }
}

#[async_trait]
impl Plugin for GoogleSearchPlugin {
    fn new() -> Result<Self, Error> {
        GoogleSearchPlugin::new()
    }

    async fn handle_room_message(&self) -> Result<(), Error> {
        Ok(())
    }
}
