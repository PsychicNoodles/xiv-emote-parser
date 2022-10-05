// use pest::iterators::Pairs;
use pest_consume::Parser;

use super::{props::LogMessageProps, EmoteText, EmoteTextError, LogMessageParser, Rule};

pub type EmoteTextResult = std::result::Result<String, EmoteTextError>;

pub fn process_log_message(log_msg: &str, params: LogMessageProps) -> EmoteTextResult {
    let mut emote_text = EmoteText::new();

    let root = LogMessageParser::parse(Rule::message, log_msg)
        .map_err(EmoteTextError::ParseError)?
        .single()
        .map_err(EmoteTextError::AstError)?;
    let message = LogMessageParser::message(root).map_err(EmoteTextError::AstError)?;

    Ok(message.process_string(&params)?)
}

// fn process_pairs(
//     mut pairs: Pairs<Rule>,
//     params: LogMessageProps,
//     targets: Targets,
// ) -> EmoteTextResult {
//     let mut r = EmoteText::new();

//     while let Some(p) = pairs.next() {}

//     todo!()
// }
