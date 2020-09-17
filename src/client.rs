//! This is a module that contains a high-level Matrix client

use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use async_trait::async_trait;
use matrix_sdk::{
    events::{room::message::MessageEventContent, SyncMessageEvent},
    Client, ClientConfig, EventEmitter, JsonStore, SyncRoom, SyncSettings,
};
use url::Url;

use crate::plugin::PluginRegistry;
use crate::plugins;
use crate::{Config, Error};

#[derive(Clone)]
pub struct MatrixClient {
    /// The inner, slightly lower level Matrix client
    inner: Arc<RwLock<Client>>,
    /// The parsed config file
    config: Arc<Mutex<Config>>,
    /// The plugin registry
    plugin_registry: Arc<RwLock<PluginRegistry>>,
}

struct PluginEventDispatcher {
    client: MatrixClient,
}

impl PluginEventDispatcher {
    pub fn new(client: MatrixClient) -> PluginEventDispatcher {
        PluginEventDispatcher { client }
    }
}

#[async_trait]
impl EventEmitter for PluginEventDispatcher {
    async fn on_room_message(&self, room: SyncRoom, event: &SyncMessageEvent<MessageEventContent>) {
        if let SyncRoom::Joined(room) = room {
            let user_id = &event.sender;
            let room_id = room.read().await.room_id.clone();

            // TODO: Figure out how to use dynamic dispatch on the Plugin trait instead of this
            // nonsense
            match &event.content {
                MessageEventContent::Audio(content) => {
                    for plugin in self.client.plugin_registry.read().await.plugins().iter() {
                        plugin
                            .on_room_audio_message(user_id, &room_id, content)
                            .await;
                    }
                }
                MessageEventContent::Emote(content) => {
                    for plugin in self.client.plugin_registry.read().await.plugins().iter() {
                        plugin
                            .on_room_emote_message(user_id, &room_id, content)
                            .await;
                    }
                }
                MessageEventContent::File(content) => {
                    for plugin in self.client.plugin_registry.read().await.plugins().iter() {
                        plugin
                            .on_room_file_message(user_id, &room_id, content)
                            .await;
                    }
                }
                MessageEventContent::Image(content) => {
                    for plugin in self.client.plugin_registry.read().await.plugins().iter() {
                        plugin
                            .on_room_image_message(user_id, &room_id, content)
                            .await;
                    }
                }
                MessageEventContent::Location(content) => {
                    for plugin in self.client.plugin_registry.read().await.plugins().iter() {
                        plugin
                            .on_room_location_message(user_id, &room_id, content)
                            .await;
                    }
                }
                MessageEventContent::Notice(content) => {
                    for plugin in self.client.plugin_registry.read().await.plugins().iter() {
                        plugin
                            .on_room_notice_message(user_id, &room_id, content)
                            .await;
                    }
                }
                MessageEventContent::ServerNotice(content) => {
                    for plugin in self.client.plugin_registry.read().await.plugins().iter() {
                        plugin
                            .on_room_server_notice_message(user_id, &room_id, content)
                            .await;
                    }
                }
                MessageEventContent::Text(content) => {
                    println!("Received text message: {:?}", content);

                    for plugin in self.client.plugin_registry.read().await.plugins().iter() {
                        plugin
                            .on_room_text_message(user_id, &room_id, content)
                            .await;
                    }
                }
                MessageEventContent::Video(content) => {
                    println!("Received video message: {:?}", content);

                    for plugin in self.client.plugin_registry.read().await.plugins().iter() {
                        plugin
                            .on_room_video_message(user_id, &room_id, content)
                            .await;
                    }
                }
                _ => {}
            }
        }
    }
}

impl MatrixClient {
    /// Creates a new MatrixClient with a given parsed `config`
    pub fn with_config(config: Config) -> Result<MatrixClient, Error> {
        let store = JsonStore::open("matrix_state").unwrap();
        let client_config = ClientConfig::new()
            .state_store(Box::new(store))
            .store_path("/home/mk/Projects/meta-matrix/test");

        let homeserver_url =
            Url::parse(&config.matrix.homeserver).map_err(Error::HomeserverParseError)?;

        let client = Client::new_with_config(homeserver_url, client_config)?;

        Ok(MatrixClient {
            inner: Arc::new(RwLock::new(client.clone())),
            config: Arc::new(Mutex::new(config)),
            plugin_registry: Arc::new(RwLock::new(PluginRegistry::new(client))),
        })
    }

    /// Authenticates with the homeserver
    pub async fn login(&self) -> Result<(), Error> {
        let mut client = self.inner.write().await;
        let config = self.config.lock().await;

        client
            .login(
                &config.matrix.username,
                &config.matrix.password,
                Some("development"),
                Some("meta-matrix"),
            )
            .await?;

        // Sync to skip old messages
        client.sync(SyncSettings::default()).await?;

        client
            .add_event_emitter(Box::new(PluginEventDispatcher::new(self.clone())))
            .await;

        Ok(())
    }

    /// Continually `sync`s with the homeserver for new updates until an error occurs
    pub async fn poll(&self) -> Result<(), Error> {
        let client = self.inner.read().await;

        // Sync forever with our stored token
        let settings = SyncSettings::default().token(client.sync_token().await.unwrap());
        client.sync_forever(settings, |_| async {}).await;

        Ok(())
    }

    /// Initializes the plugin registry
    pub async fn init_plugins(&mut self) -> Result<(), Error> {
        let mut registry = self.plugin_registry.write().await;

        registry.register::<plugins::google_search::GoogleSearchPlugin>()?;
        registry.register::<plugins::choices::ChoicesPlugin>()?;

        Ok(())
    }
}
