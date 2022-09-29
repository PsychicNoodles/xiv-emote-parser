use std::mem;

use strum_macros::EnumString;
use thiserror::Error;

use crate::log_message::{props::LogMessageProps, EmoteText, TargetMessages};

#[derive(Debug, Clone, Error)]
pub enum EmoteTextProcessError {
    #[error("Equal function did not have 2 parameters")]
    EqualParamCount,
    #[error("Equal function parameters must have same type")]
    EqualParamTypes,
}

pub(super) trait EmoteTextProcessor {
    fn process(
        self,
        emote_text: &mut EmoteText,
        targets: TargetMessages,
        props: &LogMessageProps,
    ) -> Result<(), EmoteTextProcessError>;
}

#[derive(Debug, Clone)]
pub struct Message(pub Vec<MessagePart>);

impl EmoteTextProcessor for Message {
    fn process(
        self,
        emote_text: &mut EmoteText,
        targets: TargetMessages,
        props: &LogMessageProps,
    ) -> Result<(), EmoteTextProcessError> {
        for part in self.0 {
            part.process(emote_text, targets, props);
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum MessagePart {
    Element(Element),
    Text(String),
}

impl EmoteTextProcessor for MessagePart {
    fn process(
        self,
        emote_text: &mut EmoteText,
        targets: TargetMessages,
        props: &LogMessageProps,
    ) -> Result<(), EmoteTextProcessError> {
        match self {
            MessagePart::Element(e) => e.process(emote_text, targets, props)?,
            MessagePart::Text(t) => emote_text.push_targets(targets, &t),
        };
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Param {
    Element(Element),
    Function(Function),
    Num(u64),
    Obj(String),
}

#[derive(Debug, Clone)]
pub enum IfElseThen {
    Param(Param),
    Text(String),
}

#[derive(Debug, Clone)]
pub enum Element {
    IfElse(Box<IfElse>),
    Tag(Tag, Option<String>),
}

impl EmoteTextProcessor for Element {
    fn process(
        self,
        emote_text: &mut EmoteText,
        targets: TargetMessages,
        props: &LogMessageProps,
    ) -> Result<(), EmoteTextProcessError> {
        match self {
            Element::IfElse(ie) => ie.process(emote_text, targets, props)?,
            // currently can only be formed by auto_close_tag, which never has a child
            Element::Tag(t, _s) => t.process(emote_text, targets, props)?,
        };
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct IfParam(pub Function);

impl EmoteTextProcessor for IfParam {
    fn process(
        self,
        emote_text: &mut EmoteText,
        targets: TargetMessages,
        props: &LogMessageProps,
    ) -> Result<(), EmoteTextProcessError> {
        self.0.process(emote_text, targets, props)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct IfElse {
    pub if_cond: IfParam,
    pub if_then: IfElseThen,
    pub else_then: IfElseThen,
}

impl EmoteTextProcessor for IfElse {
    fn process(
        self,
        emote_text: &mut EmoteText,
        targets: TargetMessages,
        props: &LogMessageProps,
    ) -> Result<(), EmoteTextProcessError> {
        todo!()
    }
}

#[derive(Debug, Clone, EnumString, PartialEq, Eq)]
pub enum TagName {
    Clickable,
    Sheet,
    SheetEn,
}

#[derive(Debug, Clone)]
pub struct Tag {
    pub name: TagName,
    pub params: Vec<Param>,
}

impl EmoteTextProcessor for Tag {
    fn process(
        self,
        emote_text: &mut EmoteText,
        targets: TargetMessages,
        props: &LogMessageProps,
    ) -> Result<(), EmoteTextProcessError> {
        todo!()
    }
}

#[derive(Debug, Clone, EnumString, PartialEq, Eq)]
pub enum FuncName {
    Equal,
    ObjectParameter,
    PlayerParameter,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: FuncName,
    pub params: Vec<Param>,
}

impl EmoteTextProcessor for Function {
    fn process(
        self,
        emote_text: &mut EmoteText,
        targets: TargetMessages,
        props: &LogMessageProps,
    ) -> Result<(), EmoteTextProcessError> {
        match self.name {
            FuncName::Equal => {
                if self.params.len() != 2 {}
                match &self.params[..] {
                    [a, b] if mem::discriminant(a) == mem::discriminant(b) => todo!(),
                    _ => {
                        return Err(EmoteTextProcessError::EqualParamCount);
                    }
                }
            }
            FuncName::ObjectParameter => todo!(),
            FuncName::PlayerParameter => todo!(),
        };
        Ok(())
    }
}
