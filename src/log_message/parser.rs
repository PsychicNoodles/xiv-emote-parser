use pest_consume::Parser;

use super::{props::LogMessageProps, EmoteTextError};

#[derive(Parser)]
#[grammar = "log_message/log_message.pest"]
pub struct LogMessageParser;

pub type EmoteTextResult = std::result::Result<String, EmoteTextError>;

/// The entrypoint to this library. Processes the raw log message, plugging in
/// data from the [LogMessageProps] where appropriate, and produces a plain text result.
pub fn process_log_message(log_msg: &str, params: LogMessageProps) -> EmoteTextResult {
    let root = LogMessageParser::parse(Rule::message, log_msg)
        .map_err(EmoteTextError::ParseError)?
        .single()
        .map_err(EmoteTextError::AstError)?;
    let message = LogMessageParser::message(root).map_err(EmoteTextError::AstError)?;

    Ok(message.process_string(&params)?)
}
