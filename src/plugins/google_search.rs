use async_trait::async_trait;
use log::debug;
use matrix_sdk::{
    events::{
        room::message::{
            FormattedBody, MessageEventContent, MessageFormat, TextMessageEventContent,
        },
        AnyMessageEventContent,
    },
    Client,
};
use matrix_sdk_common::identifiers::{RoomId, UserId};

use crate::plugin::Plugin;
use crate::Error;

pub struct GoogleSearchPlugin {
    client: Client,
    http_client: reqwest::Client,
}

impl GoogleSearchPlugin {}

#[async_trait]
impl Plugin for GoogleSearchPlugin {
    fn new(client: Client) -> Result<Self, Error> {
        Ok(GoogleSearchPlugin {
            client,
            http_client: reqwest::Client::new(),
        })
    }

    async fn on_room_text_message(
        &self,
        _user: &UserId,
        room: &RoomId,
        message: &TextMessageEventContent,
    ) {
        if message.body.starts_with(".g ") {
            let msg = AnyMessageEventContent::RoomMessage(MessageEventContent::Text(
                TextMessageEventContent {
                    body: "test".to_string(),
                    formatted: Some(FormattedBody {
                        body: "test".to_string(),
                        format: MessageFormat::Html,
                    }),
                    relates_to: None,
                },
            ));

            self.client
                // send our message to the room we found the "!party" command in
                // the last parameter is an optional Uuid which we don't care about.
                .room_send(room, msg, None)
                .await
                .unwrap();
        }
    }
}
