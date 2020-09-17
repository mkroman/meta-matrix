use async_trait::async_trait;
use lazy_static::lazy_static;
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
use rand::seq::IteratorRandom;
use regex::Regex;

use crate::plugin::Plugin;
use crate::Error;

lazy_static! {
    pub static ref CHOICES_REQUEST: Regex =
        Regex::new(r"(?i)^meta: (.*?) eller (.*?)\\?$").unwrap();
}

pub struct ChoicesPlugin {
    client: Client,
}

#[async_trait]
impl Plugin for ChoicesPlugin {
    fn new(client: Client) -> Result<Self, Error> {
        Ok(ChoicesPlugin { client })
    }

    async fn on_room_text_message(
        &self,
        user: &UserId,
        room: &RoomId,
        message: &TextMessageEventContent,
    ) {
        if let Some(captures) = CHOICES_REQUEST.captures(&message.body) {
            match (captures.get(1), captures.get(2)) {
                (Some(options), Some(last)) => {
                    let mut choices: Vec<_> = options.as_str().split(", ").collect();
                    choices.push(last.as_str());

                    let choice = choices
                        .iter()
                        .choose(&mut rand::thread_rng())
                        .unwrap_or_else(|| &"something went wrong");

                    let plain_message = format!("{}: {}", user.localpart(), choice);
                    let html_message = format!(
                        "<a href=\"https://matrix.to/#/{}\">{}</a>: {}",
                        user.as_str(),
                        user.localpart(),
                        choice
                    );

                    let content = AnyMessageEventContent::RoomMessage(MessageEventContent::Text(
                        TextMessageEventContent {
                            body: plain_message,
                            formatted: Some(FormattedBody {
                                body: html_message,
                                format: MessageFormat::Html,
                            }),
                            relates_to: None,
                        },
                    ));

                    self.client.room_send(room, content, None).await.unwrap();
                }
                _ => {}
            }
        }
    }
}
