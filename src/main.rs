use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Context;
use async_trait::async_trait;
use lazy_static::lazy_static;
use log::debug;
use matrix_sdk::{
    events::{
        room::message::{
            AudioInfo, AudioMessageEventContent, FormattedBody, MessageEventContent, MessageFormat,
            TextMessageEventContent,
        },
        AnyMessageEventContent, SyncMessageEvent,
    },
    Client, ClientConfig, EventEmitter, SyncRoom, SyncSettings,
};
use rink_core::{ast, btc, currency, date, gnu_units};
use rink_core::{CURRENCY_FILE, DATES_FILE, DEFAULT_FILE};
use serde::Deserialize;
use thiserror::Error;
use url::Url;

lazy_static! {
    pub static ref RINK: Arc<Mutex<rink_core::Context>> =
        Arc::new(Mutex::new(load_rink().unwrap()));
}

#[derive(Error, Debug)]
enum Error {
    #[error("matrix error")]
    MatrixError(#[from] matrix_sdk::Error),
    #[error("error when parsing config file")]
    ConfigError(#[from] toml::de::Error),
}

#[derive(Debug, Deserialize)]
struct Config {
    matrix: MatrixConfig,
}

#[derive(Debug, Deserialize)]
struct MatrixConfig {
    homeserver: String,
    username: String,
    password: String,
    rooms: Vec<String>,
}

struct RinkMessageHandler {
    /// This clone of the `Client` will send requests to the server,
    /// while the other keeps us in sync with the server using `sync_forever`.
    client: Client,
}

impl RinkMessageHandler {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

fn oneline_eval(input: &str) -> Result<String, String> {
    let ctx = RINK.clone();
    let mut ctx = ctx.lock().unwrap();

    rink_core::one_line(&mut ctx, input)
}

#[async_trait]
impl EventEmitter for RinkMessageHandler {
    async fn on_room_message(&self, room: SyncRoom, event: &SyncMessageEvent<MessageEventContent>) {
        if let SyncRoom::Joined(room) = room {
            let msg_body = if let SyncMessageEvent {
                content: MessageEventContent::Text(TextMessageEventContent { body: msg_body, .. }),
                ..
            } = event
            {
                msg_body.clone()
            } else {
                String::new()
            };

            if msg_body.starts_with(".c ") {
                let input = &msg_body[3..];
                let result = oneline_eval(input);

                let response = match result {
                    Ok(result) => format!("{} = {}", input.trim(), result),
                    Err(err) => format!("Error: {}", err),
                };

                let content = AnyMessageEventContent::RoomMessage(MessageEventContent::Text(
                    TextMessageEventContent {
                        body: response.to_string(),
                        formatted: None,
                        relates_to: None,
                    },
                ));

                // we clone here to hold the lock for as little time as possible.
                let room_id = room.read().await.room_id.clone();

                self.client
                    // send our message to the room we found the "!party" command in
                    // the last parameter is an optional Uuid which we don't care about.
                    .room_send(&room_id, content, None)
                    .await
                    .unwrap();
            } else if (msg_body == "!flag") {
                // we clone here to hold the lock for as little time as possible.
                let room_id = room.read().await.room_id.clone();

                let content = AnyMessageEventContent::RoomMessage(MessageEventContent::Text(
                    TextMessageEventContent {
                        body: "🥳 🎉 SÅ ER DER FLAG!!!!!!!!!!!!!!11111111".to_string(),
                        formatted: Some(FormattedBody {
                            body: "<h1>🥳 🎉 SÅ ER DER FLAG!!!!!!!!!!!!!!11111111</h1>".to_string(),
                            format: MessageFormat::Html,
                        }),
                        relates_to: None,
                    },
                ));

                self.client
                    // send our message to the room we found the "!party" command in
                    // the last parameter is an optional Uuid which we don't care about.
                    .room_send(&room_id, content, None)
                    .await
                    .unwrap();

                // Send the flag wav
                let content = AnyMessageEventContent::RoomMessage(MessageEventContent::Audio(
                    AudioMessageEventContent {
                        body: "flag.mp3".to_string(),
                        info: Some(Box::new(AudioInfo {
                            mimetype: Some("audio/mpeg".to_string()),
                            size: Some(2644885u32.into()),
                            duration: None,
                        })),
                        url: Some(
                            "mxc://mozilla.modular.im/262f04ef155106ac40fb141b0625670b8d530d0b"
                                .to_string(),
                        ),
                        file: None,
                    },
                ));

                self.client
                    // send our message to the room we found the "!party" command in
                    // the last parameter is an optional Uuid which we don't care about.
                    .room_send(&room_id, content, None)
                    .await
                    .unwrap();
            }
        }
    }
}

fn load_rink() -> Result<rink_core::Context, anyhow::Error> {
    let mut ctx = rink_core::Context::new();

    // Load units from the included definitions.units
    let units = {
        let mut iter = gnu_units::TokenIterator::new(&*DEFAULT_FILE.unwrap()).peekable();
        gnu_units::parse(&mut iter)
    };

    // Load dates from the included datepatterns.txt
    let dates = date::parse_datefile(&*DATES_FILE);

    // Cache ECB's daily currency values
    let ecb = cached(
        "currency.xml",
        currency::URL,
        Duration::from_secs(23 * 60 * 60),
    )
    .and_then(currency::parse);

    // Cache blockchain.info's BTC market price
    let btc = cached("btc.json", btc::URL, Duration::from_secs(3 * 60 * 60))
        .and_then(|mut file| {
            let mut buf = String::new();
            match file.read_to_string(&mut buf) {
                Ok(_size) => Ok(buf),
                Err(e) => Err(e.to_string()),
            }
        })
        .and_then(btc::parse);

    // Load currency units from the included currency.units
    let currency_defs = {
        let mut iter = gnu_units::TokenIterator::new(&*CURRENCY_FILE).peekable();
        gnu_units::parse(&mut iter)
    };

    let currency = {
        let mut defs = vec![];
        if let Ok(mut ecb) = ecb {
            defs.append(&mut ecb.defs)
        } else if let Err(e) = ecb {
            println!("Failed to load ECB currency data: {}", e);
        }
        if let Ok(mut btc) = btc {
            defs.append(&mut btc.defs)
        } else if let Err(e) = btc {
            println!("Failed to load BTC currency data: {}", e);
        }
        let mut currency_defs = currency_defs;
        defs.append(&mut currency_defs.defs);
        ast::Defs { defs }
    };

    ctx.load(units);
    ctx.load_dates(dates);
    ctx.load(currency);

    Ok(ctx)
}

fn cached(file: &str, url: &str, expiration: Duration) -> Result<File, String> {
    use std::fmt::Display;
    use std::time::SystemTime;

    fn ts<T: Display>(x: T) -> String {
        x.to_string()
    }
    let mut path = std::env::current_dir().unwrap();
    let mut tmppath = path.clone();
    path.push(file);
    let tmpfile = format!("{}.part", file);
    tmppath.push(tmpfile);

    File::open(path.clone())
        .map_err(ts)
        .and_then(|f| {
            let stats = f.metadata().map_err(ts)?;
            let mtime = stats.modified().map_err(ts)?;
            let now = SystemTime::now();
            let elapsed = now.duration_since(mtime).map_err(ts)?;
            if elapsed > expiration {
                Err("File is out of date".to_string())
            } else {
                Ok(f)
            }
        })
        .or_else(|_| {
            fs::create_dir_all(path.parent().unwrap()).map_err(|x| x.to_string())?;
            let mut f = File::create(tmppath.clone()).map_err(|x| x.to_string())?;

            let client = reqwest::blocking::Client::builder()
                .gzip(true)
                .timeout(Duration::from_secs(2))
                .build()
                .map_err(|err| format!("Failed to create http client: {}", err))?;

            client
                .get(url)
                .send()
                .map_err(|err| format!("Request failed: {}", err))?
                .copy_to(&mut f)
                .map_err(|err| format!("Request failed: {}", err))?;

            f.sync_all().map_err(|x| format!("{}", x))?;
            drop(f);
            fs::rename(tmppath.clone(), path.clone()).map_err(|x| x.to_string())?;
            File::open(path.clone()).map_err(|x| x.to_string())
        })
        // If the request fails then try to reuse the already cached file
        .or_else(|_| File::open(path.clone()).map_err(ts))
}

/// Loads the file at the given `path` and parses it as a TOML file according to `Config`
fn load_config<P: AsRef<Path>>(path: P) -> Result<Config, anyhow::Error> {
    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;

    Ok(config)
}

async fn login_and_sync(
    homeserver_url: &str,
    username: &str,
    password: &str,
) -> Result<(), anyhow::Error> {
    let client_config = ClientConfig::new();
    let homeserver_url = Url::parse(&homeserver_url)
        .with_context(|| format!("failed to parse homeserver url: {}", homeserver_url))?;
    let mut client = Client::new_with_config(homeserver_url, client_config)?;

    client
        .login(username, password, None, Some("rust-sdk"))
        .await?;

    // Sync to skip old messages
    client.sync(SyncSettings::default()).await?;

    client
        .add_event_emitter(Box::new(RinkMessageHandler::new(client.clone())))
        .await;

    // Sync forever with our stored token
    let settings = SyncSettings::default().token(client.sync_token().await.unwrap());
    client.sync_forever(settings, |_| async {}).await;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Load the config
    let config_path = "config.toml";
    let config = load_config(config_path)
        .with_context(|| format!("failed to load config file `{}'", config_path))?;

    debug!(
        "Logging in as {} on homeserver {}",
        config.matrix.username, config.matrix.homeserver
    );

    login_and_sync(
        &config.matrix.homeserver,
        &config.matrix.username,
        &config.matrix.password,
    )
    .await?;

    Ok(())
}
