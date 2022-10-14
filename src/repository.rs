#[cfg(feature = "json")]
use {serde_derive::Deserialize, serde_json};

#[cfg(any(feature = "xivapi", feature = "xivapi_blocking"))]
use reqwest;

use log::*;
use std::collections::HashMap;
use std::sync::Arc;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum LogMessageRepositoryError {
    #[error("Message not found")]
    NotFound,
    #[cfg(any(feature = "json", feature = "xivapi", feature = "xivapi_blocking"))]
    #[error("Invalid json input string")]
    InvalidJsonInput(#[from] serde_json::Error),
    #[cfg(any(feature = "xivapi", feature = "xivapi_blocking"))]
    #[error("A network error occurred")]
    Network(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, LogMessageRepositoryError>;

#[derive(Debug, Clone)]
pub struct EmoteData {
    pub name: String,
    pub en: LogMessagePair,
    pub ja: LogMessagePair,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(Deserialize))]
pub struct LogMessagePair {
    pub targeted: String,
    pub untargeted: String,
}

type MessagesMap = HashMap<String, Arc<EmoteData>>;

#[derive(Debug, Clone)]
pub struct LogMessageRepository {
    messages: MessagesMap,
    #[cfg(any(feature = "xivapi"))]
    client: reqwest::Client,
    #[cfg(any(feature = "xivapi_blocking"))]
    client_blocking: reqwest::blocking::Client,
    #[cfg(any(feature = "xivapi", feature = "xivapi_blocking"))]
    query: Vec<(String, String)>,
}

impl LogMessageRepository {
    #[cfg(feature = "json")]
    pub fn from_json(json: &str) -> Result<LogMessageRepository> {
        let messages = serde_json::from_str::<Vec<LogMessageData>>(json)
            .map_err(LogMessageRepositoryError::InvalidJsonInput)?
            .into_iter()
            .fold(HashMap::new(), |mut map, data| {
                let value = Arc::new(EmoteData {
                    name: data.name,
                    en: LogMessagePair {
                        targeted: data.en.targeted,
                        untargeted: data.en.untargeted,
                    },
                    ja: LogMessagePair {
                        targeted: data.ja.targeted,
                        untargeted: data.ja.untargeted,
                    },
                });
                for command in data.commands {
                    trace!("{} => {}", command, value.name);
                    map.insert(command, value.clone());
                }
                map
            });
        Ok(LogMessageRepository {
            messages,
            #[cfg(feature = "xivapi")]
            client: reqwest::Client::new(),
            #[cfg(feature = "xivapi_blocking")]
            client_blocking: reqwest::blocking::Client::new(),
            #[cfg(any(feature = "xivapi", feature = "xivapi_blocking"))]
            query: Vec::with_capacity(3),
        })
    }

    #[cfg(any(feature = "xivapi", feature = "xivapi_blocking"))]
    fn prep_query(api_key: Option<String>) -> Vec<(String, String)> {
        let mut query = Vec::with_capacity(3);
        query.push(("snake_case".to_string(), "1".to_string()));
        query.push((
            "columns".to_string(),
            "LogMessageTargeted,LogMessageUntargeted,Name,TextCommand".to_string(),
        ));
        if let Some(key) = api_key {
            trace!("adding xivapi private key");
            query.push(("private_key".to_string(), key));
        }
        query
    }

    #[cfg(feature = "xivapi")]
    pub async fn from_xivapi(api_key: Option<String>) -> Result<LogMessageRepository> {
        let client = reqwest::Client::new();
        let query = Self::prep_query(api_key);
        Ok(LogMessageRepository {
            messages: Self::load_xivapi(&client, &query).await?,
            client,
            #[cfg(feature = "xivapi_blocking")]
            client_blocking: reqwest::blocking::Client::new(),
            query,
        })
    }

    #[cfg(feature = "xivapi_blocking")]
    pub fn from_xivapi_blocking(api_key: Option<String>) -> Result<LogMessageRepository> {
        let client = reqwest::blocking::Client::new();
        let query = Self::prep_query(api_key);
        Ok(LogMessageRepository {
            messages: Self::load_xivapi_blocking(&client, &query)?,
            #[cfg(feature = "xivapi")]
            client: reqwest::Client::new(),
            client_blocking: client,
            query,
        })
    }

    #[cfg(any(feature = "xivapi", feature = "xivapi_blocking"))]
    fn parse_xivapi(data: self::xivapi::Response) -> MessagesMap {
        data.results
            .into_iter()
            .fold::<MessagesMap, _>(HashMap::new(), |mut map, result| {
                if let self::xivapi::EmoteData {
                    log_message_targeted: Some(targeted),
                    log_message_untargeted: Some(untargeted),
                    text_command: Some(text_command),
                    name: Some(name),
                } = result
                {
                    let data = Arc::new(EmoteData {
                        name,
                        en: LogMessagePair {
                            targeted: targeted.text_en,
                            untargeted: untargeted.text_en,
                        },
                        ja: LogMessagePair {
                            targeted: targeted.text_ja,
                            untargeted: untargeted.text_ja,
                        },
                    });
                    [
                        text_command.alias_en,
                        text_command.alias_ja,
                        text_command.command_en,
                        text_command.command_ja,
                    ]
                    .into_iter()
                    .flatten()
                    .for_each(|cmd| {
                        trace!("{} => {}", cmd, data.name);
                        map.insert(cmd, data.clone());
                    })
                } else {
                    trace!("ignoring invalid emote data ({:?})", result);
                }
                map
            })
    }

    #[cfg(feature = "xivapi")]
    async fn load_xivapi(
        client: &reqwest::Client,
        query: &[(String, String)],
    ) -> Result<MessagesMap> {
        let res = client
            .get("https://xivapi.com/emote")
            .query(&query)
            .send()
            .await?;
        let data: self::xivapi::Response = serde_json::from_str(res.text().await?.as_str())?;

        Ok(Self::parse_xivapi(data))
    }

    #[cfg(feature = "xivapi_blocking")]
    fn load_xivapi_blocking(
        client: &reqwest::blocking::Client,
        query: &[(String, String)],
    ) -> Result<MessagesMap> {
        let res = client
            .get("https://xivapi.com/emote")
            .query(&query)
            .send()?;
        let data: self::xivapi::Response = serde_json::from_str(res.text()?.as_str())?;

        Ok(Self::parse_xivapi(data))
    }

    #[cfg(feature = "xivapi")]
    pub async fn reload_messages(&mut self) -> Result<()> {
        self.messages = Self::load_xivapi(&self.client, &self.query).await?;
        Ok(())
    }

    #[cfg(feature = "xivapi_blocking")]
    pub fn reload_messages_blocking(&mut self) -> Result<()> {
        self.messages = Self::load_xivapi_blocking(&self.client_blocking, &self.query)?;
        Ok(())
    }

    #[cfg(feature = "xivapi")]
    pub fn set_xivapi_query(&mut self, query: Vec<(String, String)>) {
        self.query = query;
    }

    pub fn targeted(&self, name: &str, language: Language) -> Result<&str> {
        self.messages
            .get(name)
            .map(|data| match language {
                Language::En => data.en.targeted.as_str(),
                Language::Ja => data.ja.targeted.as_str(),
            })
            .ok_or(LogMessageRepositoryError::NotFound)
    }

    pub fn untargeted(&self, name: &str, language: Language) -> Result<&str> {
        self.messages
            .get(name)
            .map(|data| match language {
                Language::En => data.en.untargeted.as_str(),
                Language::Ja => data.ja.untargeted.as_str(),
            })
            .ok_or(LogMessageRepositoryError::NotFound)
    }

    pub fn messages(&self, name: &str) -> Result<&Arc<EmoteData>> {
        self.messages
            .get(name)
            .ok_or(LogMessageRepositoryError::NotFound)
    }

    pub fn all_messages(&self) -> Vec<&Arc<EmoteData>> {
        self.messages.values().collect()
    }

    pub fn contains_emote(&self, name: &str) -> bool {
        self.messages.contains_key(name)
    }

    pub fn emote_list(&self) -> impl Iterator<Item = &String> {
        self.messages.keys()
    }
}

#[cfg(any(feature = "xivapi", feature = "xivapi_blocking"))]
mod xivapi {
    use serde_derive::Deserialize;

    #[derive(Debug, Clone, Deserialize)]
    pub struct Response {
        pub results: Vec<EmoteData>,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct EmoteData {
        pub log_message_targeted: Option<LogMessageData>,
        pub log_message_untargeted: Option<LogMessageData>,
        pub text_command: Option<TextCommand>,
        pub name: Option<String>,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct LogMessageData {
        pub text_en: String,
        pub text_ja: String,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct TextCommand {
        pub alias_en: Option<String>,
        pub alias_ja: Option<String>,
        pub command_en: Option<String>,
        pub command_ja: Option<String>,
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(Deserialize))]
pub enum Language {
    En,
    Ja,
    // not yet supported
    // De,
    // Fr
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(Deserialize))]
#[allow(unused)]
pub struct LogMessageData {
    pub name: String,
    pub commands: Vec<String>,
    pub en: LogMessagePair,
    pub ja: LogMessagePair,
}
