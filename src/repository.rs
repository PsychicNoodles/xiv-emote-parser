#[cfg(feature = "json")]
use {serde_derive::Deserialize, serde_json};

#[cfg(feature = "xivapi")]
use {std::time::Duration, ureq};

use std::collections::HashMap;
use std::sync::Arc;
use tracing::*;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum LogMessageRepositoryError {
    #[error("Message not found")]
    NotFound,
    #[cfg(feature = "json")]
    #[error("Invalid json input string")]
    InvalidJsonInput(#[from] serde_json::Error),
    #[cfg(feature = "xivapi")]
    #[error("A network error occurred")]
    Network(#[from] ureq::Error),
    #[cfg(feature = "xivapi")]
    #[error("Request limit reached, wait before trying again")]
    RequestLimit,
    #[cfg(feature = "xivapi")]
    #[error("Io error while requesting data")]
    Io(#[from] std::io::Error),
}

// a conservative limit, but emotes should not require more than a small handful of pages
#[cfg(feature = "xivapi")]
pub const XIVAPI_REQUEST_LIMIT: u32 = 15;

pub type Result<T> = std::result::Result<T, LogMessageRepositoryError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmoteData {
    pub id: u32,
    pub name: String,
    pub en: LogMessagePair,
    pub ja: LogMessagePair,
}

impl Ord for EmoteData {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for EmoteData {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(Deserialize))]
pub struct LogMessagePair {
    pub targeted: String,
    pub untargeted: String,
}

pub type MessagesMap = HashMap<String, Arc<EmoteData>>;

#[derive(Debug, Clone)]
pub struct LogMessageRepository {
    messages: MessagesMap,
    #[cfg(feature = "xivapi")]
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
                    id: data.id,
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
            query: Vec::with_capacity(3),
        })
    }

    #[cfg(feature = "xivapi")]
    pub fn prep_xivapi_query(api_key: Option<String>) -> Vec<(String, String)> {
        let mut query = Vec::with_capacity(3);
        query.push(("snake_case".to_string(), "1".to_string()));
        query.push((
            "columns".to_string(),
            "LogMessageTargeted,LogMessageUntargeted,Name,TextCommand,ID".to_string(),
        ));
        if let Some(key) = api_key {
            trace!("adding xivapi private key");
            query.push(("private_key".to_string(), key));
        }
        query
    }

    #[cfg(feature = "xivapi")]
    pub fn from_xivapi(api_key: Option<String>) -> Result<LogMessageRepository> {
        let query = Self::prep_xivapi_query(api_key);
        Ok(LogMessageRepository {
            messages: Self::parse_xivapi(Self::load_xivapi(&query)?),
            query,
        })
    }

    #[cfg(feature = "xivapi")]
    fn parse_xivapi(results: Vec<self::xivapi::EmoteData>) -> MessagesMap {
        results
            .into_iter()
            .fold::<MessagesMap, _>(HashMap::new(), |mut map, result| {
                debug!("processing from xivapi: {:?}", result);
                if let self::xivapi::EmoteData {
                    log_message_targeted: Some(targeted),
                    log_message_untargeted: Some(untargeted),
                    text_command: Some(text_command),
                    name: Some(name),
                    id: Some(id),
                } = result
                {
                    let data = Arc::new(EmoteData {
                        id,
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
                    .filter(|cmd| !cmd.is_empty())
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
    pub fn load_xivapi(query: &[(String, String)]) -> Result<Vec<self::xivapi::EmoteData>> {
        let agent = ureq::AgentBuilder::new().build();
        let mut results = Vec::new();
        let mut req_count = 0;
        loop {
            req_count += 1;
            if req_count >= XIVAPI_REQUEST_LIMIT {
                std::thread::sleep(Duration::from_secs(2));
            }
            let page_query = ("page".to_string(), req_count.to_string());
            debug!("loading page {}", page_query.1);
            let mut req = agent.get("https://xivapi.com/emote");
            for q in query {
                req = req.query(&q.0, &q.1);
            }
            let res = req.query("page", &req_count.to_string()).call()?;
            let text = res.into_string();
            debug!("loaded from xivapi: {:?}", text);
            let mut data: self::xivapi::Response = serde_json::from_str(text?.as_str())?;
            results.append(&mut data.results);
            if data.pagination.page_next.is_none() {
                break;
            }
        }

        Ok(results)
    }

    #[cfg(feature = "xivapi")]
    pub fn reload_messages(&mut self) -> Result<()> {
        self.messages = Self::parse_xivapi(Self::load_xivapi(&self.query)?);
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

    pub fn emote_list_by_id(&self) -> impl Iterator<Item = &String> {
        let mut values: Vec<_> = self.messages.iter().collect();
        values.sort_unstable_by(|(_, v1), (_, v2)| v1.id.cmp(&v2.id));
        values.into_iter().map(|(k, _)| k)
    }

    pub fn find_emote_id(&self, name: &str) -> Option<u32> {
        self.messages
            .iter()
            .find(|msg| msg.0 == name)
            .map(|msg| msg.1.id)
    }

    pub fn messages_map(&self) -> &MessagesMap {
        &self.messages
    }
}

#[cfg(feature = "xivapi")]
pub mod xivapi {
    use serde_derive::Deserialize;

    #[derive(Debug, Clone, Deserialize)]
    pub struct Response {
        pub pagination: Pagination,
        pub results: Vec<EmoteData>,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct Pagination {
        pub page_next: Option<u32>,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct EmoteData {
        pub log_message_targeted: Option<LogMessageData>,
        pub log_message_untargeted: Option<LogMessageData>,
        pub text_command: Option<TextCommand>,
        pub name: Option<String>,
        pub id: Option<u32>,
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
    pub id: u32,
    pub name: String,
    pub commands: Vec<String>,
    pub en: LogMessagePair,
    pub ja: LogMessagePair,
}
