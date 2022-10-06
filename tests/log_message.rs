use std::error::Error;

use discord_xiv_emotes::log_message::{
    condition::{Character, Gender, LogMessageAnswers},
    process_log_message,
};
use serde_json::Value;

// #[test]
// fn can_parse_en() {
//     let log_msg = "<Clickable(<If(Equal(ObjectParameter(1),ObjectParameter(2)))>you<Else/><If(PlayerParameter(7))><SheetEn(ObjStr,2,PlayerParameter(7),1,1)/><Else/>ObjectParameter(2)</If></If>)/> <If(Equal(ObjectParameter(1),ObjectParameter(2)))>look<Else/>looks</If> at <If(Equal(ObjectParameter(1),ObjectParameter(3)))><If(PlayerParameter(8))><SheetEn(ObjStr,2,PlayerParameter(8),1,1)/><Else/>you</If><Else/><If(PlayerParameter(8))><SheetEn(ObjStr,2,PlayerParameter(8),1,1)/><Else/>ObjectParameter(3)</If></If> in surprise.";

//     let parse = LogMessageParser::parse(Rule::message, log_msg).unwrap();
//     println!("{:#?}", parse);
//     let root = parse.single().unwrap();
//     let message = LogMessageParser::message(root);
//     println!("{:#?}", message);

//     assert!(message.is_ok(), "did not parse correctly");
// }

// #[test]
// fn can_parse_jp() {
//     let log_msg = "<If(PlayerParameter(7))><Sheet(ObjStr,PlayerParameter(7),0)/><Else/>ObjectParameter(2)</If>はおどろいた。";

//     let parse = LogMessageParser::parse(Rule::message, log_msg).unwrap();
//     println!("{:#?}", parse);
//     let root = parse.single().unwrap();
//     let message = LogMessageParser::message(root);
//     println!("{:#?}", message);

//     assert!(message.is_ok(), "did not parse correctly");
// }

#[test]
fn can_parse_en_with_ast() -> Result<(), impl Error> {
    let log_msg = "<Clickable(<If(Equal(ObjectParameter(1),ObjectParameter(2)))>you<Else/><If(PlayerParameter(7))><SheetEn(ObjStr,2,PlayerParameter(7),1,1)/><Else/>ObjectParameter(2)</If></If>)/> <If(Equal(ObjectParameter(1),ObjectParameter(2)))>look<Else/>looks</If> at <If(Equal(ObjectParameter(1),ObjectParameter(3)))><If(PlayerParameter(8))><SheetEn(ObjStr,2,PlayerParameter(8),1,1)/><Else/>you</If><Else/><If(PlayerParameter(8))><SheetEn(ObjStr,2,PlayerParameter(8),1,1)/><Else/>ObjectParameter(3)</If></If> in surprise.";

    let origin = Character::new("K'haldru Alaba", Gender::Female, true, true);
    let target = Character::new("Puruo Jelly", Gender::Male, true, false);
    let text = process_log_message(log_msg, LogMessageAnswers::new(origin, target).unwrap());
    println!("{:?}", text);
    text.map(|_| ())
}

#[test]
fn can_parse_en_with_ast_gendered_speaker() -> Result<(), impl Error> {
    let log_msg = "<Clickable(<If(Equal(ObjectParameter(1),ObjectParameter(2)))>you<Else/><If(PlayerParameter(7))><SheetEn(ObjStr,2,PlayerParameter(7),1,1)/><Else/>ObjectParameter(2)</If></If>)/> <If(Equal(ObjectParameter(1),ObjectParameter(2)))>express<Else/>expresses</If> <If(Equal(ObjectParameter(1),ObjectParameter(2)))>your<Else/><If(PlayerParameter(7))><If(<Sheet(BNpcName,PlayerParameter(7),6)/>)>her<Else/>his</If><Else/><If(PlayerParameter(5))>her<Else/>his</If></If></If> annoyance with <If(Equal(ObjectParameter(1),ObjectParameter(3)))><If(PlayerParameter(8))><SheetEn(ObjStr,2,PlayerParameter(8),1,1)/><Else/>you</If><Else/><If(PlayerParameter(8))><SheetEn(ObjStr,2,PlayerParameter(8),1,1)/><Else/>ObjectParameter(3)</If></If>.";

    let origin = Character::new("K'haldru Alaba", Gender::Female, true, true);
    let target = Character::new("Puruo Jelly", Gender::Male, true, false);
    let text = process_log_message(log_msg, LogMessageAnswers::new(origin, target).unwrap());
    println!("{:?}", text);
    text.map(|_| ())
}

// #[test]
// fn can_parse_all_emotes() {
//     let data = include_str!("../emote-22106.json");
//     let v: Value = serde_json::from_str(data).expect("couldn't parse test json");
//     let emotes = v["Results"]
//         .as_array()
//         .expect("test json didn't contain Results array");

//     let char1 = Character::new("K'haldru Alaba", Gender::Female, true, true);
//     let char2 = Character::new("Puruo Jelly", Gender::Male, true, false);
//     let char3 = Character::new("Ardbert Bestboy", Gender::Male, false, false);

//     for emote in emotes {
//         let messages = [
//             &emote["LogMessageTargeted"]["Text_en"],
//             &emote["LogMessageTargeted"]["Text_jp"],
//             &emote["LogMessageUntargeted"]["Text_en"],
//             &emote["LogMessageUntargeted"]["Text_jp"],
//         ]
//         .map(|v| v.as_str().expect("couldn't find log message data"));
//     }
// }
