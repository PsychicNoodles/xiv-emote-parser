use pest_consume::Parser;

pub use super::ast::types::{ConditionState, ConditionText, ConditionTexts, Text};
use super::{ast::condition::Answers, EmoteTextError};

#[derive(Parser)]
#[grammar = "log_message/log_message.pest"]
pub struct LogMessageParser;

pub type EmoteTextResult<T> = std::result::Result<T, EmoteTextError>;

/// The entrypoint to this library. Processes the raw log message, plugging in
/// data from the [Answers] implementation where appropriate, and produces a plain text result.
///
/// A default implementation for [Answers] is provided in [LogMessageAnswers].
///
/// [LogMessageAnswers]: super::ast::condition::LogMessageAnswers
pub fn process_log_message<T>(log_msg: &str, answers: &T) -> EmoteTextResult<String>
where
    T: Answers,
{
    let condition_texts = extract_condition_texts(log_msg)?;

    Ok(condition_texts
        .map_texts(answers, |text| match text {
            Text::Dynamic(d) => Some(answers.as_string(d)),
            Text::Static(s) => Some(s.to_string()),
        })
        .collect())
}

pub fn extract_condition_texts(log_msg: &str) -> EmoteTextResult<ConditionTexts> {
    let root = LogMessageParser::parse(Rule::message, log_msg)
        .map_err(EmoteTextError::ParseError)?
        .single()
        .map_err(EmoteTextError::AstError)?;
    let message = LogMessageParser::message(root).map_err(EmoteTextError::AstError)?;
    let condition_texts = message.process_string()?;
    Ok(condition_texts)
}
