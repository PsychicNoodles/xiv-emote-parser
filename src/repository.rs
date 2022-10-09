#[cfg(feature = "json")]
use {serde_derive::Deserialize, serde_json};

use std::{collections::HashMap, convert};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum LogMessageRepositoryError {
    #[cfg(feature = "json")]
    #[error("Invalid json input string")]
    InvalidJsonInput(#[from] serde_json::Error),
    #[error("Message not found")]
    NotFound,
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
    #[cfg(feature = "xivapi")]
    #[cfg(feature = "xivapi_blocking")]
    api_key: Option<String>,
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
            #[cfg(feature = "xivapi_blocking")]
            api_key: None,
        })
    }

    #[cfg(feature = "xivapi")]
    pub fn from_xivapi(api_key: &str) -> Result<LogMessageRepository> {
        todo!()
    }

    pub fn targeted(&self, name: String, language: Language) -> Result<&str> {
        self.messages
            .get(&name)
            .map(|m| m.get(&language))
            .flatten()
            .map(|p| p.targeted.as_str())
            .ok_or(LogMessageRepositoryError::NotFound)
    }

    pub fn untargeted(&self, name: String, language: Language) -> Result<&str> {
        self.messages
            .get(&name)
            .map(|m| m.get(&language))
            .flatten()
            .map(|p| p.untargeted.as_str())
            .ok_or(LogMessageRepositoryError::NotFound)
    }

    pub fn messages(&self, name: String) -> Result<[&str; 4]> {
        self.messages
            .get(&name)
            .ok_or(LogMessageRepositoryError::NotFound)
            .map(|m| {
                let en = m
                    .get(&Language::En)
                    .ok_or(LogMessageRepositoryError::NotFound)?;
                let jp = m
                    .get(&Language::Jp)
                    .ok_or(LogMessageRepositoryError::NotFound)?;
                Ok([
                    en.targeted.as_str(),
                    en.untargeted.as_str(),
                    jp.targeted.as_str(),
                    jp.untargeted.as_str(),
                ])
            })
            .and_then(convert::identity)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(Deserialize))]
pub enum Language {
    En,
    Jp,
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
