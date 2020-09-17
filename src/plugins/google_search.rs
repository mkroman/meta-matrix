use async_trait::async_trait;
use log::debug;
use matrix_sdk::events::room::message::TextMessageEventContent;
use matrix_sdk_common::identifiers::{RoomId, UserId};
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

    async fn on_room_text_message(
        &self,
        _user: &UserId,
        _room: &RoomId,
        message: &TextMessageEventContent,
    ) {
        if message.body.starts_with(".g ") {
            debug!("Google search: {}", message.body);
        }
    }
}
