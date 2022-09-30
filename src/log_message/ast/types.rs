use strum_macros::EnumString;
use thiserror::Error;

use crate::log_message::{
    props::{LogMessageProps, LogMessageVarError, PParam},
    EmoteText, TargetMessages,
};

#[derive(Debug, Clone, Error)]
pub enum EmoteTextProcessError {
    #[error("Equal function did not have 2 parameters")]
    EqualParamCount,
    #[error("Equal function parameters must have same type")]
    EqualParamTypes,
    #[error("Function used in unexpected place ({name:?})")]
    DanglingFunction { name: FuncName },
    #[error("Could not find value in log message props (not yet implemented?)")]
    PropMissing(#[from] LogMessageVarError),
    #[error("Invalid combination of function ({name:?}) and parameters ({params:?})")]
    InvalidFunc { name: FuncName, params: Vec<Param> },
    #[error("Function returned unexpected value ({name:?} {params:?}, return: {value:?}))")]
    UnexpectedFuncReturn {
        name: FuncName,
        params: Vec<Param>,
        value: String,
    },
    #[error("Clickable contained unexpected param ({params:?})")]
    UnexpectedClickable { params: Vec<Param> },
}

pub(super) trait EmoteTextProcessor {
    fn process(
        &self,
        emote_text: &mut EmoteText,
        targets: TargetMessages,
        props: &LogMessageProps,
    ) -> Result<(), EmoteTextProcessError>;
}

#[derive(Debug, Clone)]
pub struct Message(pub Vec<MessagePart>);

impl EmoteTextProcessor for Message {
    fn process(
        &self,
        emote_text: &mut EmoteText,
        targets: TargetMessages,
        props: &LogMessageProps,
    ) -> Result<(), EmoteTextProcessError> {
        for part in &self.0 {
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
        &self,
        emote_text: &mut EmoteText,
        targets: TargetMessages,
        props: &LogMessageProps,
    ) -> Result<(), EmoteTextProcessError> {
        match self {
            MessagePart::Element(e) => e.process(emote_text, targets, props)?,
            MessagePart::Text(t) => emote_text.push_targets(&targets, &t),
        };
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Param {
    Element(Element),
    Function(Function),
    Num(u32),
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
        &self,
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
        &self,
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
        &self,
        emote_text: &mut EmoteText,
        targets: TargetMessages,
        props: &LogMessageProps,
    ) -> Result<(), EmoteTextProcessError> {
        let if_fun = &self.if_cond.0;
        match if_fun.name {
            FuncName::Equal => {
                if if_fun.params.len() != 2 {}
                match &if_fun.params[..] {
                    [Param::Function(a), Param::Function(b)] => todo!(),
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

#[derive(Debug, Clone, Copy, EnumString, PartialEq, Eq)]
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
        &self,
        emote_text: &mut EmoteText,
        targets: TargetMessages,
        props: &LogMessageProps,
    ) -> Result<(), EmoteTextProcessError> {
        match self.name {
            // clickable is afaik just ignored and always contains an IfElse
            TagName::Clickable => match &self.params[..] {
                [Param::Element(element)] => element.process(emote_text, targets, props),
                _ => {
                    return Err(EmoteTextProcessError::UnexpectedClickable {
                        params: self.params.clone(),
                    })
                }
            },
            TagName::Sheet => todo!(),
            TagName::SheetEn => todo!(),
        }
    }
}

#[derive(Debug, Clone, Copy, EnumString, PartialEq, Eq)]
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
        &self,
        emote_text: &mut EmoteText,
        targets: TargetMessages,
        props: &LogMessageProps,
    ) -> Result<(), EmoteTextProcessError> {
        match self.name {
            FuncName::Equal => {
                return Err(EmoteTextProcessError::DanglingFunction {
                    name: FuncName::Equal,
                })
            }
            FuncName::ObjectParameter => match self.value(props)? {
                FunctionValue::ObjString(s) => {
                    emote_text.push_targets(&targets, s);
                }
                value @ _ => {
                    return Err(EmoteTextProcessError::UnexpectedFuncReturn {
                        name: self.name,
                        params: self.params.clone(),
                        value: format!("{:?}", value),
                    });
                }
            },
            FuncName::PlayerParameter => {
                return Err(EmoteTextProcessError::DanglingFunction {
                    name: FuncName::PlayerParameter,
                })
            }
        };
        Ok(())
    }
}

impl Function {
    fn value<'a>(
        &'a self,
        props: &'a LogMessageProps,
    ) -> Result<FunctionValue, EmoteTextProcessError> {
        match (self.name, &self.params[..]) {
            (FuncName::Equal, _) => Err(EmoteTextProcessError::DanglingFunction {
                name: FuncName::Equal,
            }),
            (FuncName::ObjectParameter, [Param::Num(ind)]) => props
                .object_parameter(*ind)
                .map(FunctionValue::ObjString)
                .map_err(EmoteTextProcessError::PropMissing),
            (FuncName::PlayerParameter, [Param::Num(ind)]) => props
                .player_parameter(*ind)
                .map(FunctionValue::PlayerParam)
                .map_err(EmoteTextProcessError::PropMissing),
            (name, _params) => Err(EmoteTextProcessError::InvalidFunc {
                name,
                params: self.params.clone(),
            }),
        }
    }
}

#[derive(Debug, Clone)]
enum FunctionValue<'a> {
    ObjString(&'a String),
    PlayerParam(PParam<'a>),
}
