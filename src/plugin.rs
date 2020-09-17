use async_trait::async_trait;
use log::{debug, error};
use matrix_sdk::{
    events::room::message::{
        AudioMessageEventContent, EmoteMessageEventContent, FileMessageEventContent,
        ImageMessageEventContent, LocationMessageEventContent, NoticeMessageEventContent,
        ServerNoticeMessageEventContent, TextMessageEventContent, VideoMessageEventContent,
    },
    Client,
};
use matrix_sdk_common::identifiers::{RoomId, UserId};

use crate::Error;

pub struct PluginRegistry {
    client: Client,
    plugins: Vec<Box<dyn Plugin>>,
}

#[async_trait]
pub trait Plugin: Send + Sync {
    /// Returns the full Rust module path of the plugin
    fn module_path(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Returns the type name fo the plugin
    fn name(&self) -> &'static str {
        self.module_path()
            .split("::")
            .last()
            .expect("could not extract module name")
    }

    fn new(client: Client) -> Result<Self, Error>
    where
        Self: Sized;

    /// Called when a audio message is received in a room
    async fn on_room_audio_message(
        &self,
        _user: &UserId,
        _room: &RoomId,
        _message: &AudioMessageEventContent,
    ) {
    }

    /// Called when a emote message is received in a room
    async fn on_room_emote_message(
        &self,
        _user: &UserId,
        _room: &RoomId,
        _message: &EmoteMessageEventContent,
    ) {
    }

    /// Called when a file message is received in a room
    async fn on_room_file_message(
        &self,
        _user: &UserId,
        _room: &RoomId,
        _message: &FileMessageEventContent,
    ) {
    }

    /// Called when a image message is received in a room
    async fn on_room_image_message(
        &self,
        _user: &UserId,
        _room: &RoomId,
        _message: &ImageMessageEventContent,
    ) {
    }

    /// Called when a location message is received in a room
    async fn on_room_location_message(
        &self,
        _user: &UserId,
        _room: &RoomId,
        _message: &LocationMessageEventContent,
    ) {
    }

    /// Called when a notice message is received in a room
    async fn on_room_notice_message(
        &self,
        _user: &UserId,
        _room: &RoomId,
        _message: &NoticeMessageEventContent,
    ) {
    }

    /// Called when a server notice message is received in a room
    async fn on_room_server_notice_message(
        &self,
        _user: &UserId,
        _room: &RoomId,
        _message: &ServerNoticeMessageEventContent,
    ) {
    }

    /// Called when a text message is received in a room
    async fn on_room_text_message(
        &self,
        _user: &UserId,
        _room: &RoomId,
        _message: &TextMessageEventContent,
    ) {
    }

    /// Called when a video message is received in a room
    async fn on_room_video_message(
        &self,
        _user: &UserId,
        _room: &RoomId,
        _message: &VideoMessageEventContent,
    ) {
    }
}

impl PluginRegistry {
    /// Constructs and returns a new plugin registry
    pub fn new(client: Client) -> PluginRegistry {
        PluginRegistry {
            client,
            plugins: vec![],
        }
    }

    /// Instantiates the given trait and adds it to the registry
    pub fn register<P: Plugin + 'static>(&mut self) -> Result<(), Error> {
        debug!("Registering plugin {}", std::any::type_name::<P>());

        let plugin = P::new(self.client.clone());

        match plugin {
            Ok(plugin) => {
                debug!("Registered plugin {}", plugin.name());

                self.plugins.push(Box::new(plugin));
            }
            Err(err) => error!("Failed to register plugin: {}", err),
        }

        Ok(())
    }

    /// Returns a ref slice of all the plugins
    pub fn plugins(&self) -> &[Box<dyn Plugin>] {
        self.plugins.as_ref()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct TestPlugin {
        client: Client,
    }

    #[async_trait]
    impl Plugin for TestPlugin {
        fn new(client: Client) -> Result<TestPlugin, Error> {
            Ok(TestPlugin { client })
        }

        async fn on_room_text_message(
            &self,
            _user: &UserId,
            _room: &RoomId,
            _message: &TextMessageEventContent,
        ) {
        }
    }

    #[test]
    fn test_register_plugin() {
        let url: url::Url = "http://example.com".parse().unwrap();
        let client = Client::new(url).unwrap();
        let mut registry = PluginRegistry::new(client);

        registry.register::<TestPlugin>().unwrap();
    }
}
