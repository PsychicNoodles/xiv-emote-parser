use std::error::Error;

use discord_xiv_emotes::log_message::{
    condition::{Character, Gender, LogMessageAnswers},
    process_log_message, EmoteTextError,
};
use serde_json::Value;
use thiserror::Error;

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
    let text = process_log_message(log_msg, &LogMessageAnswers::new(origin, target).unwrap());
    println!("{:?}", text);
    text.map(|_| ())
}

#[test]
fn can_parse_en_with_ast_gendered_speaker() -> Result<(), impl Error> {
    let log_msg = "<Clickable(<If(Equal(ObjectParameter(1),ObjectParameter(2)))>you<Else/><If(PlayerParameter(7))><SheetEn(ObjStr,2,PlayerParameter(7),1,1)/><Else/>ObjectParameter(2)</If></If>)/> <If(Equal(ObjectParameter(1),ObjectParameter(2)))>express<Else/>expresses</If> <If(Equal(ObjectParameter(1),ObjectParameter(2)))>your<Else/><If(PlayerParameter(7))><If(<Sheet(BNpcName,PlayerParameter(7),6)/>)>her<Else/>his</If><Else/><If(PlayerParameter(5))>her<Else/>his</If></If></If> annoyance with <If(Equal(ObjectParameter(1),ObjectParameter(3)))><If(PlayerParameter(8))><SheetEn(ObjStr,2,PlayerParameter(8),1,1)/><Else/>you</If><Else/><If(PlayerParameter(8))><SheetEn(ObjStr,2,PlayerParameter(8),1,1)/><Else/>ObjectParameter(3)</If></If>.";

    let origin = Character::new("K'haldru Alaba", Gender::Female, true, true);
    let target = Character::new("Puruo Jelly", Gender::Male, true, false);
    let text = process_log_message(log_msg, &LogMessageAnswers::new(origin, target).unwrap());
    println!("{:?}", text);
    text.map(|_| ())
}

#[test]
fn can_parse_en_cry() -> Result<(), impl Error> {
    let log_msg = "<Clickable(<If(Equal(ObjectParameter(1),ObjectParameter(2)))>your<Else/><If(Equal(ObjectParameter(1),ObjectParameter(2)))>you<Else/><If(PlayerParameter(7))><SheetEn(ObjStr,2,PlayerParameter(7),1,1)/><Else/>ObjectParameter(2)</If></If>'s</If>)/> eyes brim over with tears.";

    let origin = Character::new("K'haldru Alaba", Gender::Female, true, true);
    let target = Character::new("Puruo Jelly", Gender::Male, true, false);
    let text = process_log_message(log_msg, &LogMessageAnswers::new(origin, target).unwrap());
    println!("{:?}", text);
    text.map(|_| ())
}

#[derive(Debug, Error)]
#[error("Failed to parse {name} ({error:?}) (original: {original})")]
struct MessageTestError {
    name: String,
    original: String,
    error: EmoteTextError,
}

#[test]
fn can_parse_all_emotes() -> Result<(), impl Error> {
    let data = include_str!("../emote-22106.json");
    let v: Value = serde_json::from_str(data).expect("couldn't parse test json");
    let emotes = v["Results"]
        .as_array()
        .expect("test json didn't contain Results array");

    let char1 = Character::new("K'haldru Alaba", Gender::Female, true, true);
    let char2 = Character::new("Puruo Jelly", Gender::Male, true, false);
    let char3 = Character::new("Nanamo Ul Namo", Gender::Female, false, false);
    let answerses = [
        LogMessageAnswers::new(char1.clone(), char2.clone()),
        LogMessageAnswers::new(char2.clone(), char3.clone()),
        LogMessageAnswers::new(char3.clone(), char1.clone()),
    ]
    .map(|r| r.expect("couldn't set up answers"));

    emotes
        .into_iter()
        .map(|emote| {
            let name = emote["Name"]
                .as_str()
                .expect("emote didn't have a name")
                .to_string();
            let messages = [
                &emote["LogMessageTargeted"]["Text_en"],
                &emote["LogMessageTargeted"]["Text_ja"],
                &emote["LogMessageUntargeted"]["Text_en"],
                &emote["LogMessageUntargeted"]["Text_ja"],
            ]
            .into_iter()
            .filter_map(|v| match v.as_str() {
                Some(s) => Some(s),
                None => {
                    eprintln!("skipping {} due to no messages", name);
                    None
                }
            })
            .collect::<Vec<_>>();

            for message in messages {
                for answers in &answerses {
                    let text = process_log_message(message, answers);
                    match text {
                        Err(e) => {
                            return Err(MessageTestError {
                                name,
                                original: message.to_string(),
                                error: e,
                            });
                        }
                        Ok(t) => {
                            println!(
                                "{} ({:?}, {:?}): {}",
                                name,
                                answers.origin_character(),
                                answers.target_character(),
                                t
                            );
                        }
                    }
                }
            }
            Ok(())
        })
        .collect()
}
