use pest_consume::Parser;

use super::{
    ast::{condition::Answers, types::Text},
    EmoteTextError,
};

#[derive(Parser)]
#[grammar = "log_message/log_message.pest"]
pub struct LogMessageParser;

pub type EmoteTextResult = std::result::Result<String, EmoteTextError>;

/// The entrypoint to this library. Processes the raw log message, plugging in
/// data from the [Answers] implementation where appropriate, and produces a plain text result.
///
/// A default implementation for [Answers] is provided in [LogMessageAnswers].
///
/// [LogMessageAnswers]: super::ast::condition::LogMessageAnswers
pub fn process_log_message<T>(log_msg: &str, answers: T) -> EmoteTextResult
where
    T: Answers,
{
    let root = LogMessageParser::parse(Rule::message, log_msg)
        .map_err(EmoteTextError::ParseError)?
        .single()
        .map_err(EmoteTextError::AstError)?;
    let message = LogMessageParser::message(root).map_err(EmoteTextError::AstError)?;

    let condition_texts = message.process_string()?;

    Ok(condition_texts
        .into_iter()
        .filter_map(|ctxt| {
            if ctxt.conds.into_iter().all(|cond| answers.as_bool(cond)) {
                match ctxt.text {
                    Text::Dynamic(d) => Some(answers.as_string(d)),
                    Text::Static(s) => Some(s),
                }
            } else {
                None
            }
        })
        .collect())
}
