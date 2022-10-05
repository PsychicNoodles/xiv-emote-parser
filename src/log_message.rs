use thiserror::Error;

use self::ast::types::EmoteTextProcessError;

mod ast;
mod parser;
mod props;
mod types;

pub use self::parser::process_log_message;
use self::parser::Rule;
pub use self::props::LogMessageProps;

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

#[cfg(test)]
mod test {
    use crate::log_message::{
        parser::{process_log_message, LogMessageParser},
        props::{LogMessageProps, ObjectProp, Player, PlayerProp},
        types::Gender,
    };
    use pest_consume::Parser;

    use super::*;
    #[test]
    fn can_parse_en() {
        let log_msg = "<Clickable(<If(Equal(ObjectParameter(1),ObjectParameter(2)))>you<Else/><If(PlayerParameter(7))><SheetEn(ObjStr,2,PlayerParameter(7),1,1)/><Else/>ObjectParameter(2)</If></If>)/> <If(Equal(ObjectParameter(1),ObjectParameter(2)))>look<Else/>looks</If> at <If(Equal(ObjectParameter(1),ObjectParameter(3)))><If(PlayerParameter(8))><SheetEn(ObjStr,2,PlayerParameter(8),1,1)/><Else/>you</If><Else/><If(PlayerParameter(8))><SheetEn(ObjStr,2,PlayerParameter(8),1,1)/><Else/>ObjectParameter(3)</If></If> in surprise.";

        let parse = LogMessageParser::parse(Rule::message, log_msg).unwrap();
        println!("{:#?}", parse);
        let root = parse.single().unwrap();
        let message = LogMessageParser::message(root);
        println!("{:#?}", message);

        assert!(message.is_ok(), "did not parse correctly");
    }

    #[test]
    fn can_parse_jp() {
        let log_msg = "<If(PlayerParameter(7))><Sheet(ObjStr,PlayerParameter(7),0)/><Else/>ObjectParameter(2)</If>はおどろいた。";

        let parse = LogMessageParser::parse(Rule::message, log_msg).unwrap();
        println!("{:#?}", parse);
        let root = parse.single().unwrap();
        let message = LogMessageParser::message(root);
        println!("{:#?}", message);

        assert!(message.is_ok(), "did not parse correctly");
    }

    #[test]
    fn can_parse_en_with_ast() {
        let log_msg = "<Clickable(<If(Equal(ObjectParameter(1),ObjectParameter(2)))>you<Else/><If(PlayerParameter(7))><SheetEn(ObjStr,2,PlayerParameter(7),1,1)/><Else/>ObjectParameter(2)</If></If>)/> <If(Equal(ObjectParameter(1),ObjectParameter(2)))>look<Else/>looks</If> at <If(Equal(ObjectParameter(1),ObjectParameter(3)))><If(PlayerParameter(8))><SheetEn(ObjStr,2,PlayerParameter(8),1,1)/><Else/>you</If><Else/><If(PlayerParameter(8))><SheetEn(ObjStr,2,PlayerParameter(8),1,1)/><Else/>ObjectParameter(3)</If></If> in surprise.";

        let text = process_log_message(
            log_msg,
            LogMessageProps::new(
                ObjectProp::new("K'haldru Alaba", "K'haldru Alaba", Some("Puruo Jelly")),
                PlayerProp::new(
                    Some(Player::new("K'haldru Alaba", "Asura", Gender::Female)),
                    Some(Player::new("Puruo Jelly", "Asura", Gender::Male)),
                    Gender::Female,
                ),
            ),
        );

        println!("{}", text.expect("did not parse correctly"));
    }

    #[test]
    fn can_parse_en_with_ast_gendered_speaker() {
        let log_msg = "<Clickable(<If(Equal(ObjectParameter(1),ObjectParameter(2)))>you<Else/><If(PlayerParameter(7))><SheetEn(ObjStr,2,PlayerParameter(7),1,1)/><Else/>ObjectParameter(2)</If></If>)/> <If(Equal(ObjectParameter(1),ObjectParameter(2)))>express<Else/>expresses</If> <If(Equal(ObjectParameter(1),ObjectParameter(2)))>your<Else/><If(PlayerParameter(7))><If(<Sheet(BNpcName,PlayerParameter(7),6)/>)>her<Else/>his</If><Else/><If(PlayerParameter(5))>her<Else/>his</If></If></If> annoyance with <If(Equal(ObjectParameter(1),ObjectParameter(3)))><If(PlayerParameter(8))><SheetEn(ObjStr,2,PlayerParameter(8),1,1)/><Else/>you</If><Else/><If(PlayerParameter(8))><SheetEn(ObjStr,2,PlayerParameter(8),1,1)/><Else/>ObjectParameter(3)</If></If>.";

        let text = process_log_message(
            log_msg,
            LogMessageProps::new(
                ObjectProp::new("Other Player", "K'haldru Alaba", Some("Puruo Jelly")),
                PlayerProp::new(
                    Some(Player::new("K'haldru Alaba", "Asura", Gender::Female)),
                    Some(Player::new("Puruo Jelly", "Asura", Gender::Male)),
                    Gender::Female,
                ),
            ),
        );

        println!("{}", text.expect("did not parse correctly"));
    }
}
