//! Known conditions derived from functions and tags in log messages.
//! Abstracts actual calls so that full output can be pre-calculated with
//! specific portions that require player data.

use thiserror::Error;

use super::types::{FuncName, Function, Param, Tag};

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
    /// seemingly a default for the origin of the message's gender
    /// used after checking if self and also with OriginFemale
    /// PlayerParameter(5)
    IsDefaultFemale,
    /// if the origin of the message is a player
    /// PlayerParameter(7)
    IsOriginPlayer,
    /// if the target of the message is a player
    /// PlayerParameter(8)
    IsTargetPlayer,
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

#[derive(Debug, Clone)]
pub enum ConditionOrigin {
    Function(Function),
    Tag(Tag),
}

#[derive(Debug, Clone, Error)]
#[error("Unknown condition ({0:?})")]
pub struct ConditionError(ConditionOrigin);

impl TryFrom<Function> for Condition {
    type Error = ConditionError;

    fn try_from(fun: Function) -> Result<Self, Self::Error> {
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
                _ => Err(ConditionError(ConditionOrigin::Function(fun))),
            },
            FuncName::ObjectParameter => match &fun.params[..] {
                [Param::Num(2)] => Ok(Condition::NpcOriginName),
                [Param::Num(3)] => Ok(Condition::NpcTargetName),
                _ => Err(ConditionError(ConditionOrigin::Function(fun))),
            },
            FuncName::PlayerParameter => match &fun.params[..] {
                [Param::Num(7)] => Ok(Condition::IsOriginPlayer),
                [Param::Num(8)] => Ok(Condition::IsTargetPlayer),
                [Param::Num(5)] => Ok(Condition::IsDefaultFemale),
                _ => Err(ConditionError(ConditionOrigin::Function(fun))),
            },
        }
    }
}
