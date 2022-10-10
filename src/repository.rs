#[cfg(feature = "json")]
use {serde_derive::Deserialize, serde_json};

#[cfg(any(feature = "xivapi", feature = "xivapi_blocking"))]
use reqwest;

use std::{collections::HashMap, convert};

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
struct LogMessagePair {
    targeted: String,
    untargeted: String,
}

#[derive(Debug, Clone, Default)]
pub struct LogMessageRepository {
    messages: HashMap<String, HashMap<Language, LogMessagePair>>,
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
            .fold(
                HashMap::<String, HashMap<Language, LogMessagePair>>::new(),
                |mut map, data| {
                    map.entry(data.name)
                        .and_modify(|m| {
                            m.insert(
                                data.language,
                                LogMessagePair {
                                    targeted: data.targeted,
                                    untargeted: data.untargeted,
                                },
                            );
                        })
                        .or_default();
                    map
                },
            );
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

    #[cfg(feature = "xivapi")]
    pub async fn from_xivapi(api_key: Option<String>) -> Result<LogMessageRepository> {
        let client = reqwest::Client::new();
        let mut query = Vec::with_capacity(3);
        query.push(("snake_case".to_string(), "1".to_string()));
        query.push((
            "columns".to_string(),
            "LogMessageTargeted,LogMessageUntargeted,Name".to_string(),
        ));
        if let Some(key) = api_key {
            query.push(("private_key".to_string(), key));
        }
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
        let mut query = Vec::with_capacity(3);
        query.push(("snake_case".to_string(), "1".to_string()));
        query.push((
            "columns".to_string(),
            "LogMessageTargeted,LogMessageUntargeted,Name".to_string(),
        ));
        if let Some(key) = api_key {
            query.push(("private_key".to_string(), key));
        }
        Ok(LogMessageRepository {
            messages: Self::load_xivapi_blocking(&client, &query)?,
            #[cfg(feature = "xivapi")]
            client: reqwest::Client::new(),
            client_blocking: client,
            query,
        })
    }

    #[cfg(any(feature = "xivapi", feature = "xivapi_blocking"))]
    fn parse_xivapi(
        data: self::xivapi::Response,
    ) -> HashMap<String, HashMap<Language, LogMessagePair>> {
        data.results
            .into_iter()
            .fold(HashMap::new(), |mut map, result| {
                if let self::xivapi::EmoteData {
                    log_message_targeted: Some(targeted),
                    log_message_untargeted: Some(untargeted),
                    name: Some(name),
                } = result
                {
                    let mut m = HashMap::new();
                    m.insert(
                        Language::En,
                        LogMessagePair {
                            targeted: targeted.text_en,
                            untargeted: untargeted.text_en,
                        },
                    );
                    m.insert(
                        Language::Ja,
                        LogMessagePair {
                            targeted: targeted.text_ja,
                            untargeted: untargeted.text_ja,
                        },
                    );
                    map.insert(name, m);
                }
                map
            })
    }

    #[cfg(feature = "xivapi")]
    async fn load_xivapi(
        client: &reqwest::Client,
        query: &[(String, String)],
    ) -> Result<HashMap<String, HashMap<Language, LogMessagePair>>> {
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
    ) -> Result<HashMap<String, HashMap<Language, LogMessagePair>>> {
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
            .and_then(|m| m.get(&language))
            .map(|p| p.targeted.as_str())
            .ok_or(LogMessageRepositoryError::NotFound)
    }

    pub fn untargeted(&self, name: &str, language: Language) -> Result<&str> {
        self.messages
            .get(name)
            .and_then(|m| m.get(&language))
            .map(|p| p.untargeted.as_str())
            .ok_or(LogMessageRepositoryError::NotFound)
    }

    fn extract_messages(m: &HashMap<Language, LogMessagePair>) -> Result<[&str; 4]> {
        let en = m
            .get(&Language::En)
            .ok_or(LogMessageRepositoryError::NotFound)?;
        let jp = m
            .get(&Language::Ja)
            .ok_or(LogMessageRepositoryError::NotFound)?;
        Ok([
            en.targeted.as_str(),
            en.untargeted.as_str(),
            jp.targeted.as_str(),
            jp.untargeted.as_str(),
        ])
    }

    pub fn messages(&self, name: &str) -> Result<[&str; 4]> {
        self.messages
            .get(name)
            .ok_or(LogMessageRepositoryError::NotFound)
            .map(Self::extract_messages)
            .and_then(convert::identity)
    }

    pub fn all_messages(&self) -> Result<Vec<[&str; 4]>> {
        self.messages.values().map(Self::extract_messages).collect()
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
        pub name: Option<String>,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct LogMessageData {
        pub text_en: String,
        pub text_ja: String,
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
pub struct LogMessageData {
    name: String,
    targeted: String,
    untargeted: String,
    language: Language,
}
