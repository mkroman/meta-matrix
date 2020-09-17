use async_trait::async_trait;
use lazy_static::lazy_static;
use log::error;
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
use scraper::{ElementRef, Html, Selector};
use url::Url;

use crate::plugin::Plugin;
use crate::Error;

lazy_static! {
    pub static ref HEADER_LINK_SELECTOR: Selector = Selector::parse(".r > a").unwrap();
    pub static ref SUB_HEADER_SELECTOR: Selector = Selector::parse("h3").unwrap();
    pub static ref SEARCH_BODY_SELECTOR: Selector = Selector::parse("div#search div#rso").unwrap();
    pub static ref SEARCH_RESULT_SELECTOR: Selector = Selector::parse("div.g .rc").unwrap();
}

pub struct GoogleSearchPlugin {
    client: Client,
    http_client: reqwest::Client,
}

#[derive(thiserror::Error, Debug)]
pub enum GoogleSearchError {
    #[error("could not extract search body")]
    NoSearchBody,
    #[error("no search results")]
    NoSearchResults,
    #[error("missing expected element")]
    MissingElement(&'static str),
    #[error("http client error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("unable to parse result url: {0}")]
    UrlParseError(#[from] url::ParseError),
}

#[derive(Debug, Clone)]
struct SearchResult {
    title: String,
    url: Url,
}

impl SearchResult {
    fn from_element(element: &ElementRef) -> Result<SearchResult, GoogleSearchError> {
        let header_link = element
            .select(&HEADER_LINK_SELECTOR)
            .next()
            .ok_or_else(|| GoogleSearchError::MissingElement("header_link"))?;

        let header_title = header_link
            .select(&SUB_HEADER_SELECTOR)
            .next()
            .ok_or_else(|| GoogleSearchError::MissingElement("sub_header"))?
            .text()
            .collect::<Vec<_>>()
            .join("");

        let header_link_href = header_link
            .value()
            .attr("href")
            .ok_or_else(|| GoogleSearchError::MissingElement("href_attr"))?;

        Ok(SearchResult {
            title: header_title,
            url: header_link_href.parse()?,
        })
    }
}

impl GoogleSearchPlugin {
    async fn search(&self, query: &str) -> Result<Option<SearchResult>, GoogleSearchError> {
        let req = self
            .http_client
            .get("https://www.google.dk/search")
            .query(&[("q", query), ("hl", "en")])
            .send()
            .await?;
        let body = req.text().await?;

        Self::parse_search_page(&body)
    }

    fn parse_search_page(body: &str) -> Result<Option<SearchResult>, GoogleSearchError> {
        let document = Html::parse_document(body);
        let search_body = document
            .select(&SEARCH_BODY_SELECTOR)
            .next()
            .ok_or(GoogleSearchError::NoSearchBody)?;
        let result = search_body
            .select(&SEARCH_RESULT_SELECTOR)
            .next()
            .ok_or(GoogleSearchError::NoSearchResults)?;

        SearchResult::from_element(&result).map(|x| Some(x))
    }
}

#[async_trait]
impl Plugin for GoogleSearchPlugin {
    fn new(client: Client) -> Result<Self, Error> {
        let http_client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:80.0) Gecko/20100101 Firefox/80.0")
            .gzip(true)
            .build()
            .map_err(Error::ReqwestBuildError)?;

        Ok(GoogleSearchPlugin {
            client,
            http_client,
        })
    }

    async fn on_room_text_message(
        &self,
        _user: &UserId,
        room: &RoomId,
        message: &TextMessageEventContent,
    ) {
        if message.body.starts_with(".g ") {
            let result = self.search(&message.body[3..]).await;
            let message = match result {
                Ok(Some(result)) => format!("{} - {}", result.title, result.url),
                Ok(None) => format!("No results"),
                Err(err) => format!("Error: {}", err),
            };

            let content = AnyMessageEventContent::RoomMessage(MessageEventContent::Text(
                TextMessageEventContent {
                    body: message.clone(),
                    formatted: Some(FormattedBody {
                        body: message,
                        format: MessageFormat::Html,
                    }),
                    relates_to: None,
                },
            ));

            self.client.room_send(room, content, None).await.unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search() {
        assert!(GoogleSearchPlugin::parse_search_page(include_str!(
            "../../test/google-search-page.html"
        ))
        .is_ok());
    }
}
