use pest_consume::Parser;
use thiserror::Error;

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

    fn push_targets(&mut self, targets: TargetMessages, s: &str) {
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
}

#[derive(Debug, Clone, Copy)]
struct TargetMessages {
    you_untarget: bool,
    you_target_other: bool,
    other_target_you: bool,
    other_target_other: bool,
    other_untarget: bool,
}

#[derive(Debug, Error)]
pub enum EmoteTextError {
    #[error("No log message found by parser")]
    ParseError(#[from] pest::error::Error<Rule>),
    #[error("Could not access parsed log message")]
    MessageParseError,
}

mod test {
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

        let parse = LogMessageParser::parse(Rule::message, log_msg);
        println!("{:#?}", parse);

        assert!(parse.is_ok(), "did not parse correctly");
    }

    #[test]
    fn can_parse_empty_open_tag() {
        let msg = "<Clickable>";

        let parse = LogMessageParser::parse(Rule::open_tag, msg);
        println!("{:#?}", parse);

        assert!(parse.is_ok(), "did not parse correctly");
    }

    #[test]
    fn can_parse_num_param_open_tag() {
        let msg = "<Clickable(1)>";

        let parse = LogMessageParser::parse(Rule::open_tag, msg);
        println!("{:#?}", parse);

        assert!(parse.is_ok(), "did not parse correctly");
    }

    #[test]
    fn can_parse_empty_element() {
        let msg = "<Clickable></Clickable>";

        let parse = LogMessageParser::parse(Rule::element, msg);
        println!("{:#?}", parse);

        assert!(parse.is_ok(), "did not parse correctly");
    }

    #[test]
    fn can_parse_text_element() {
        let msg = "<Clickable>asdf</Clickable>";

        let parse = LogMessageParser::parse(Rule::element, msg);
        println!("{:#?}", parse);

        assert!(parse.is_ok(), "did not parse correctly");
    }

    #[test]
    fn can_parse_clickable() {
        let msg = "<Clickable(<If(Equal(ObjectParameter(1),ObjectParameter(2)))>you<Else/><If(PlayerParameter(7))><SheetEn(ObjStr,2,PlayerParameter(7),1,1)/><Else/>ObjectParameter(2)</If></If>)/>";

        let parse = LogMessageParser::parse(Rule::element, msg);
        println!("{:#?}", parse);

        assert!(parse.is_ok(), "did not parse correctly");
    }

    #[test]
    fn can_parse_if() {
        let msg = "<If(PlayerParameter(7))><SheetEn(ObjStr,2,PlayerParameter(7),1,1)/><Else/>ObjectParameter(2)</If>";

        let parse = LogMessageParser::parse(Rule::if_else_element, msg);
        println!("{:#?}", parse);

        assert!(parse.is_ok(), "did not parse correctly");
    }
}
