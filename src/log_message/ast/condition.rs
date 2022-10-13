//! Known conditions derived from functions and tags in log messages.
//! Abstracts actual calls so that full output can be pre-calculated with
//! specific portions that require player data.

use thiserror::Error;

pub use crate::log_message::types::Gender;

use super::types::{FuncName, Function, IfParam, Obj, Param, Tag, TagName};

/// Abstraction of conditions provided by functions and tags in log messages.
/// Should only appear as the condition for an if-else.
#[derive(Debug, Clone, Copy)]
pub enum Condition {
    /// if the current player character is the origin of the message
    /// Equal(ObjectParameter(1),ObjectParameter(2))
    IsSelfOrigin,
    /// if the current player character is the target of the message
    /// Equal(ObjectParameter(1),ObjectParameter(3))
    IsSelfTarget,
    /// if the origin of the message's gender is female
    /// <Sheet(BNpcName,PlayerParameter(7),6)/>
    IsOriginFemale,
    /// if the origin of the message's gender is female when not a player(?)
    /// PlayerParameter(5)
    IsOriginFemaleNpc,
    /// if the origin of the message is a player
    /// PlayerParameter(7)
    IsOriginPlayer,
    /// if the target of the message is a player
    /// PlayerParameter(8)
    IsTargetPlayer,
}

/// Abstraction of text with value depending on contextual player data.
/// Should only appear as the then portion of an if-else or otherwise as text.
#[derive(Debug, Clone, Copy)]
pub enum DynamicText {
    /// the name of the origin of the message when not a player
    /// ObjectParameter(2)
    NpcOriginName,
    /// the name of the target of the message when not a player
    /// ObjectParameter(3)
    NpcTargetName,
    /// the EN name of the origin of the message
    /// <SheetEn(ObjStr,2,PlayerParameter(7),1,1)/>
    PlayerOriginNameEn,
    /// the EN name of the target of the message
    /// <SheetEn(ObjStr,2,PlayerParameter(8),1,1)/>
    PlayerTargetNameEn,
    /// the JP name of the origin of the message
    /// <Sheet(ObjStr,PlayerParameter(7),0)/>
    PlayerOriginNameJp,
    /// the JP name of the target of the message
    /// <Sheet(ObjStr,PlayerParameter(8),0)/>
    PlayerTargetNameJp,
}

pub trait ConditionAnswer {
    fn as_bool(&self, cond: &Condition) -> bool;
}

pub trait DynamicTextAnswer {
    fn as_string(&self, text: &DynamicText) -> String;
}

pub trait Answers: ConditionAnswer + DynamicTextAnswer {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Character {
    pub name: String,
    pub gender: Gender,
    pub is_pc: bool,
    pub is_self: bool,
}

impl Character {
    pub fn new<N>(name: N, gender: Gender, is_pc: bool, is_self: bool) -> Character
    where
        N: ToString,
    {
        Character {
            name: name.to_string(),
            gender,
            is_pc,
            is_self,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogMessageAnswers {
    origin_character: Character,
    target_character: Character,
}

#[derive(Debug, Clone, Error)]
pub enum LogMessageAnswersError {
    #[error("Only one character can be self")]
    MultipleSelves,
}

impl LogMessageAnswers {
    pub fn new(
        origin_character: Character,
        target_character: Character,
    ) -> Result<LogMessageAnswers, LogMessageAnswersError> {
        if origin_character.is_self
            && target_character.is_self
            && origin_character != target_character
        {
            Err(LogMessageAnswersError::MultipleSelves)
        } else {
            Ok(LogMessageAnswers {
                origin_character,
                target_character,
            })
        }
    }

    pub fn origin_character(&self) -> &Character {
        &self.origin_character
    }

    pub fn target_character(&self) -> &Character {
        &self.target_character
    }
}

impl ConditionAnswer for LogMessageAnswers {
    fn as_bool(&self, cond: &Condition) -> bool {
        match cond {
            Condition::IsSelfOrigin => self.origin_character.is_self,
            Condition::IsSelfTarget => self.target_character.is_self,
            Condition::IsOriginFemale | Condition::IsOriginFemaleNpc => {
                matches!(self.origin_character.gender, Gender::Female)
            }
            Condition::IsOriginPlayer => self.origin_character.is_pc,
            Condition::IsTargetPlayer => self.target_character.is_pc,
        }
    }
}

impl DynamicTextAnswer for LogMessageAnswers {
    fn as_string(&self, text: &DynamicText) -> String {
        match text {
            // afaik names are the same regardless of language
            // todo add option to append world name
            DynamicText::NpcOriginName
            | DynamicText::PlayerOriginNameEn
            | DynamicText::PlayerOriginNameJp => self.origin_character.name.clone(),
            DynamicText::NpcTargetName
            | DynamicText::PlayerTargetNameEn
            | DynamicText::PlayerTargetNameJp => self.target_character.name.clone(),
        }
    }
}

impl Answers for LogMessageAnswers {}

#[derive(Debug, Clone)]
pub enum Origin {
    Function(Function),
    Tag(Tag),
}

#[derive(Debug, Clone, Error)]
#[error("Unknown condition ({0:?})")]
pub struct ConditionError(Origin);

// in TryFrom impls below, Err only bindings provided for clarity

impl TryFrom<&Function> for Condition {
    type Error = ConditionError;

    fn try_from(fun: &Function) -> Result<Self, Self::Error> {
        #[allow(clippy::match_single_binding)]
        match fun.name {
            FuncName::Equal => match &fun.params[..] {
                [Param::Function(Function {
                    name: FuncName::ObjectParameter,
                    params: p1,
                }), Param::Function(Function {
                    name: FuncName::ObjectParameter,
                    params: p2,
                })] if matches!(&p1[..], [Param::Num(1)]) && matches!(&p2[..], [Param::Num(2)]) => {
                    Ok(Condition::IsSelfOrigin)
                }
                [Param::Function(Function {
                    name: FuncName::ObjectParameter,
                    params: p1,
                }), Param::Function(Function {
                    name: FuncName::ObjectParameter,
                    params: p2,
                })] if matches!(&p1[..], [Param::Num(1)]) && matches!(&p2[..], [Param::Num(3)]) => {
                    Ok(Condition::IsSelfTarget)
                }
                _ => Err(ConditionError(Origin::Function(fun.clone()))),
            },
            FuncName::ObjectParameter => match &fun.params[..] {
                _ => Err(ConditionError(Origin::Function(fun.clone()))),
            },
            FuncName::PlayerParameter => match &fun.params[..] {
                [Param::Num(7)] => Ok(Condition::IsOriginPlayer),
                [Param::Num(8)] => Ok(Condition::IsTargetPlayer),
                [Param::Num(5)] => Ok(Condition::IsOriginFemaleNpc),
                _ => Err(ConditionError(Origin::Function(fun.clone()))),
            },
        }
    }
}

impl TryFrom<&Tag> for Condition {
    type Error = ConditionError;

    fn try_from(tag: &Tag) -> Result<Self, Self::Error> {
        #[allow(clippy::match_single_binding)]
        match tag.name {
            TagName::Clickable => Err(ConditionError(Origin::Tag(tag.clone()))),
            TagName::Sheet => match &tag.params[..] {
                [Param::Obj(Obj::BNpcName), Param::Function(Function {
                    name: FuncName::PlayerParameter,
                    params: p1,
                }), Param::Num(6)]
                    if matches!(&p1[..], [Param::Num(7)]) =>
                {
                    Ok(Condition::IsOriginFemale)
                }
                _ => Err(ConditionError(Origin::Tag(tag.clone()))),
            },
            TagName::SheetEn => match &tag.params[..] {
                _ => Err(ConditionError(Origin::Tag(tag.clone()))),
            },
        }
    }
}

impl TryFrom<&IfParam> for Condition {
    type Error = ConditionError;

    fn try_from(value: &IfParam) -> Result<Self, Self::Error> {
        match value {
            IfParam::Function(f) => Condition::try_from(f),
            IfParam::Tag(t) => Condition::try_from(t),
        }
    }
}

#[derive(Debug, Clone, Error)]
#[error("Unknown dynamic text ({0:?})")]
pub struct DynamicTextError(Origin);

impl TryFrom<Function> for DynamicText {
    type Error = DynamicTextError;

    fn try_from(fun: Function) -> Result<Self, Self::Error> {
        match fun.name {
            #[allow(clippy::match_single_binding)]
            FuncName::Equal => match &fun.params[..] {
                _ => Err(DynamicTextError(Origin::Function(fun))),
            },
            FuncName::ObjectParameter => match &fun.params[..] {
                [Param::Num(2)] => Ok(DynamicText::NpcOriginName),
                [Param::Num(3)] => Ok(DynamicText::NpcTargetName),
                _ => Err(DynamicTextError(Origin::Function(fun))),
            },
            #[allow(clippy::match_single_binding)]
            FuncName::PlayerParameter => match &fun.params[..] {
                _ => Err(DynamicTextError(Origin::Function(fun))),
            },
        }
    }
}

impl TryFrom<Tag> for DynamicText {
    type Error = DynamicTextError;

    fn try_from(tag: Tag) -> Result<Self, Self::Error> {
        match tag.name {
            TagName::Clickable => Err(DynamicTextError(Origin::Tag(tag))),
            TagName::Sheet => match &tag.params[..] {
                [Param::Obj(Obj::ObjStr), Param::Function(Function {
                    name: FuncName::PlayerParameter,
                    params: p1,
                }), Param::Num(0)]
                    if matches!(&p1[..], [Param::Num(7)]) =>
                {
                    Ok(DynamicText::PlayerOriginNameJp)
                }
                [Param::Obj(Obj::ObjStr), Param::Function(Function {
                    name: FuncName::PlayerParameter,
                    params: p1,
                }), Param::Num(0)]
                    if matches!(&p1[..], [Param::Num(8)]) =>
                {
                    Ok(DynamicText::PlayerTargetNameJp)
                }
                _ => Err(DynamicTextError(Origin::Tag(tag))),
            },
            TagName::SheetEn => match &tag.params[..] {
                [Param::Obj(Obj::ObjStr), Param::Num(2), Param::Function(Function {
                    name: FuncName::PlayerParameter,
                    params: p2,
                }), Param::Num(1), Param::Num(1)]
                    if matches!(&p2[..], [Param::Num(7)]) =>
                {
                    Ok(DynamicText::PlayerOriginNameEn)
                }
                [Param::Obj(Obj::ObjStr), Param::Num(2), Param::Function(Function {
                    name: FuncName::PlayerParameter,
                    params: p2,
                }), Param::Num(1), Param::Num(1)]
                    if matches!(&p2[..], [Param::Num(8)]) =>
                {
                    Ok(DynamicText::PlayerTargetNameEn)
                }
                _ => Err(DynamicTextError(Origin::Tag(tag))),
            },
        }
    }
}
