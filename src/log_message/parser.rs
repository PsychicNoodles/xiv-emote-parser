use pest::iterators::Pairs;
use pest_consume::Parser;

use super::{
    ast::types::MessagePart, props::LogMessageProps, EmoteText, EmoteTextError, LogMessageParser,
    Rule, TargetMessages,
};

type EmoteTextResult = std::result::Result<EmoteText, EmoteTextError>;

pub fn process_log_message(log_msg: &str, params: LogMessageProps) -> EmoteTextResult {
    let mut emote_text = EmoteText::new();
    let targets = TargetMessages::new();

    let root = LogMessageParser::parse(Rule::message, log_msg)
        .map_err(EmoteTextError::ParseError)?
        .single()
        .map_err(EmoteTextError::AstError)?;
    let message = LogMessageParser::message(root).map_err(EmoteTextError::AstError)?;

    for part in message.0 {
        match part {
            MessagePart::Element(_) => todo!(),
            MessagePart::Text(t) => emote_text.push_all(&t),
        }
    }

    todo!()
}

fn process_pairs(
    mut pairs: Pairs<Rule>,
    params: LogMessageProps,
    targets: TargetMessages,
) -> EmoteTextResult {
    let mut r = EmoteText::new();

    while let Some(p) = pairs.next() {}

    todo!()
}

fn process_tag(
    pairs: &mut Pairs<Rule>,
    params: &LogMessageProps,
    targets: TargetMessages,
    emote_text: EmoteText,
) -> EmoteTextResult {
    todo!()
}

fn process_func(
    pairs: &mut Pairs<Rule>,
    params: &LogMessageProps,
    targets: TargetMessages,
    emote_text: EmoteText,
) -> EmoteTextResult {
    todo!()
}
