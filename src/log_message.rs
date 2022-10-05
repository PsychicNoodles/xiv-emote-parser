use pest_consume::Parser;
use thiserror::Error;

use self::ast::types::EmoteTextProcessError;

mod ast;
mod parser;
mod props;
mod types;

#[derive(Parser)]
#[grammar = "log_message/log_message.pest"]
struct LogMessageParser;

#[derive(Debug)]
pub struct EmoteText {
    // no args
    pub you_untarget: String,
    // "target"
    pub you_target_other: String,
    // "user"
    pub other_target_you: String,
    // "user", "target"
    pub other_target_other: String,
    // "user"
    pub other_untarget: String,
}

impl EmoteText {
    fn new() -> EmoteText {
        EmoteText {
            you_untarget: String::new(),
            you_target_other: String::new(),
            other_target_you: String::new(),
            other_target_other: String::new(),
            other_untarget: String::new(),
        }
    }

    fn push_targets(&mut self, targets: &Targets, s: &str) {
        if targets.you_untarget {
            self.you_untarget.push_str(s);
        }
        if targets.you_target_other {
            self.you_target_other.push_str(s);
        }
        if targets.other_target_you {
            self.other_target_you.push_str(s);
        }
        if targets.other_target_other {
            self.other_target_other.push_str(s);
        }
        if targets.other_untarget {
            self.other_untarget.push_str(s);
        }
    }

    fn push_all(&mut self, s: &str) {
        self.you_untarget.push_str(s);
        self.you_target_other.push_str(s);
        self.other_target_you.push_str(s);
        self.other_target_other.push_str(s);
        self.other_untarget.push_str(s);
    }
}

#[derive(Debug, Clone, Copy)]
struct Targets {
    you_untarget: bool,
    you_target_other: bool,
    other_target_you: bool,
    other_target_other: bool,
    other_untarget: bool,
}

impl Targets {
    fn new() -> Targets {
        Targets {
            you_untarget: true,
            you_target_other: true,
            other_target_you: true,
            other_target_other: true,
            other_untarget: true,
        }
    }
}

// #[derive(Debug, Clone)]
// struct TargetedMessage {
//     targets: Targets,
//     text: String,
// }

// impl TargetedMessage {
//     fn new(targets: Targets, text: String) -> TargetedMessage {
//         TargetedMessage { targets, text }
//     }
// }

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

mod test {
    use crate::log_message::{
        props::{LogMessageProps, ObjectProp, Player, PlayerProp},
        types::Gender,
    };

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

        let parse = LogMessageParser::parse(Rule::message, log_msg).unwrap();
        println!("{:#?}", parse);
        let root = parse.single().unwrap();
        let message = LogMessageParser::message(root).unwrap();

        let text = message.process_string(&LogMessageProps::new(
            ObjectProp::new("K'haldru Alaba", "K'haldru Alaba", Some("Puruo Jelly")),
            PlayerProp::new(
                Some(Player::new("K'haldru Alaba", "Asura", Gender::Female)),
                Some(Player::new("Puruo Jelly", "Asura", Gender::Male)),
                Gender::Female,
            ),
        ));

        println!("{}", text.expect("did not parse correctly"));
    }

    #[test]
    fn can_parse_en_with_ast_gendered_speaker() {
        let log_msg = "<Clickable(<If(Equal(ObjectParameter(1),ObjectParameter(2)))>you<Else/><If(PlayerParameter(7))><SheetEn(ObjStr,2,PlayerParameter(7),1,1)/><Else/>ObjectParameter(2)</If></If>)/> <If(Equal(ObjectParameter(1),ObjectParameter(2)))>express<Else/>expresses</If> <If(Equal(ObjectParameter(1),ObjectParameter(2)))>your<Else/><If(PlayerParameter(7))><If(<Sheet(BNpcName,PlayerParameter(7),6)/>)>her<Else/>his</If><Else/><If(PlayerParameter(5))>her<Else/>his</If></If></If> annoyance with <If(Equal(ObjectParameter(1),ObjectParameter(3)))><If(PlayerParameter(8))><SheetEn(ObjStr,2,PlayerParameter(8),1,1)/><Else/>you</If><Else/><If(PlayerParameter(8))><SheetEn(ObjStr,2,PlayerParameter(8),1,1)/><Else/>ObjectParameter(3)</If></If>.";

        let parse = LogMessageParser::parse(Rule::message, log_msg).unwrap();
        println!("{:#?}", parse);
        let root = parse.single().unwrap();
        let message = LogMessageParser::message(root).unwrap();

        let text = message.process_string(&LogMessageProps::new(
            ObjectProp::new("Other Player", "K'haldru Alaba", Some("Puruo Jelly")),
            PlayerProp::new(
                Some(Player::new("K'haldru Alaba", "Asura", Gender::Female)),
                Some(Player::new("Puruo Jelly", "Asura", Gender::Male)),
                Gender::Female,
            ),
        ));

        println!("{}", text.expect("did not parse correctly"));
    }
}
