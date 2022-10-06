use thiserror::Error;

use self::ast::types::EmoteTextProcessError;

mod ast;
mod parser;
mod types;

pub use self::ast::condition;
pub use self::parser::process_log_message;
pub use self::parser::EmoteTextResult;
use self::parser::Rule;

#[derive(Debug, Error)]
pub enum EmoteTextError {
    #[error("No log message found by parser")]
    ParseError(#[source] pest::error::Error<Rule>),
    #[error("Could not parse to intermediate ast")]
    AstError(#[source] pest_consume::Error<Rule>),
    #[error("Could not access parsed log message")]
    MessageParseError,
    #[error("Error while processing log message ast")]
    ProcessError(#[from] EmoteTextProcessError),
}
