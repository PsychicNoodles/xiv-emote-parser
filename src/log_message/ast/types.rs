use strum_macros::EnumString;
use thiserror::Error;

use crate::log_message::props::{LogMessagePropError, LogMessageProps, PParam, Player};

#[derive(Debug, Clone, Error)]
pub enum EmoteTextProcessError {
    #[error("Function used in unexpected place ({name:?})")]
    DanglingFunction { name: FuncName },
    #[error("Could not find value in log message props (not yet implemented?)")]
    PropMissing(#[from] LogMessagePropError),
    #[error("Invalid combination of function ({name:?}) and parameters ({params:?})")]
    InvalidFunc { name: FuncName, params: Vec<Param> },
    #[error("Invalid combination of tag ({name:?}) and parameters ({params:?})")]
    InvalidTag { name: TagName, params: Vec<Param> },
    #[error("Function returned unexpected value ({name:?} {params:?}, return: {value:?}))")]
    UnexpectedFuncReturn {
        name: FuncName,
        params: Vec<Param>,
        value: String,
    },
    #[error("Tag returned unexpected value ({name:?} {params:?}, return: {value:?}))")]
    UnexpectedTagReturn {
        name: TagName,
        params: Vec<Param>,
        value: String,
    },
    #[error("Clickable contained unexpected param ({params:?})")]
    UnexpectedClickable { params: Vec<Param> },
    #[error("Unexpected obj parameter ({name:?})")]
    UnexpectedObj { name: Obj },
    #[error("Unexpected num parameter ({value:?})")]
    UnexpectedNum { value: u32 },
}

pub(super) trait EmoteTextProcessor {
    fn process(&self, props: &LogMessageProps) -> Result<String, EmoteTextProcessError>;
}

#[derive(Debug, Clone)]
pub struct Message(pub Vec<MessagePart>);

impl EmoteTextProcessor for Message {
    fn process(&self, props: &LogMessageProps) -> Result<String, EmoteTextProcessError> {
        self.0
            .iter()
            .map(|p| p.process(props))
            .collect::<Result<_, EmoteTextProcessError>>()
    }
}

impl Message {
    pub fn process_string(&self, props: &LogMessageProps) -> Result<String, EmoteTextProcessError> {
        self.process(props)
    }
}

#[derive(Debug, Clone)]
pub enum MessagePart {
    Element(Element),
    Text(String),
}

impl EmoteTextProcessor for MessagePart {
    fn process(&self, props: &LogMessageProps) -> Result<String, EmoteTextProcessError> {
        match self {
            MessagePart::Element(e) => e.process(props),
            MessagePart::Text(t) => Ok(t.clone()),
        }
    }
}

#[derive(Debug, Clone, Copy, EnumString, PartialEq, Eq)]
pub enum Obj {
    ObjStr,
    BNpcName,
}

#[derive(Debug, Clone)]
pub enum Param {
    Element(Element),
    Function(Function),
    Num(u32),
    Obj(Obj),
}

impl EmoteTextProcessor for Param {
    fn process(&self, props: &LogMessageProps) -> Result<String, EmoteTextProcessError> {
        match self {
            Param::Element(e) => e.process(props),
            Param::Function(f) => f.process(props),
            Param::Obj(o) => Err(EmoteTextProcessError::UnexpectedObj { name: *o }),
            Param::Num(n) => Err(EmoteTextProcessError::UnexpectedNum { value: *n }),
        }
    }
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
    fn process(&self, props: &LogMessageProps) -> Result<String, EmoteTextProcessError> {
        match self {
            Element::IfElse(ie) => ie.process(props),
            // currently can only be formed by auto_close_tag, which never has a child
            Element::Tag(t, _s) => t.process(props),
        }
    }
}

#[derive(Debug, Clone)]
pub enum IfParam {
    Function(Function),
    Tag(Tag),
}

#[derive(Debug, Clone)]
pub struct IfElse {
    pub if_cond: IfParam,
    pub if_then: IfElseThen,
    pub else_then: IfElseThen,
}

impl EmoteTextProcessor for IfElse {
    fn process(&self, props: &LogMessageProps) -> Result<String, EmoteTextProcessError> {
        let if_cond = match &self.if_cond {
            IfParam::Function(fun) => {
                match fun.value(props)? {
                    FunctionValue::ObjString(o) => {
                        Err(EmoteTextProcessError::UnexpectedFuncReturn {
                            name: fun.name,
                            params: fun.params.clone(),
                            value: format!("{:?}", o),
                        })
                    }
                    // ie. <If(PlayerParameter(8))>
                    FunctionValue::PlayerParam(p) => match p {
                        PParam::Player(opt) => Ok(opt.is_some()),
                        PParam::Bool(b) => Ok(b),
                    },
                    FunctionValue::Bool(b) => Ok(b),
                }
            }
            IfParam::Tag(tag) => match tag.value(props)? {
                TagValue::Text(t) => Err(EmoteTextProcessError::UnexpectedTagReturn {
                    name: tag.name,
                    params: tag.params.clone(),
                    value: t,
                }),
                TagValue::Bool(b) => Ok(b),
                TagValue::None => Err(EmoteTextProcessError::UnexpectedTagReturn {
                    name: tag.name,
                    params: tag.params.clone(),
                    value: "(Clickable)".to_string(),
                }),
            },
        }?;
        if if_cond {
            match &self.if_then {
                IfElseThen::Param(p) => p.process(props),
                IfElseThen::Text(t) => Ok(t.clone()),
            }
        } else {
            match &self.else_then {
                IfElseThen::Param(p) => p.process(props),
                IfElseThen::Text(t) => Ok(t.clone()),
            }
        }
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
    fn process(&self, props: &LogMessageProps) -> Result<String, EmoteTextProcessError> {
        match (self.name, self.params.len()) {
            // clickable is afaik just ignored and always contains an IfElse
            (TagName::Clickable, _) => match &self.params[..] {
                [Param::Element(element)] => element.process(props),
                _ => Err(EmoteTextProcessError::UnexpectedClickable {
                    params: self.params.clone(),
                }),
            },
            (TagName::Sheet, 3) | (TagName::SheetEn, 5) => match self.value(props) {
                Ok(TagValue::Text(t)) => Ok(t),
                Ok(v) => Err(EmoteTextProcessError::UnexpectedTagReturn {
                    name: self.name,
                    params: self.params.clone(),
                    value: format!("{:?}", v),
                }),
                Err(e) => Err(e),
            },
            _ => Err(EmoteTextProcessError::InvalidTag {
                name: self.name,
                params: self.params.clone(),
            }),
        }
    }
}

impl Tag {
    fn value(&self, props: &LogMessageProps) -> Result<TagValue, EmoteTextProcessError> {
        match (self.name, &self.params[..]) {
            (TagName::Clickable, [Param::Element(_)]) => Ok(TagValue::None),
            (
                TagName::Sheet,
                [Param::Obj(obj), Param::Function(
                    pparam_fun @ Function {
                        name: FuncName::PlayerParameter,
                        params: _,
                    },
                ), Param::Num(p3)],
            ) => {
                // todo make Param::Obj hold an enum
                match obj {
                    Obj::ObjStr => Ok(TagValue::Text(LogMessageProps::sheet_objstr(
                        pparam_fun.to_player(props)?,
                        *p3,
                    )?)),
                    Obj::BNpcName => Ok(TagValue::Bool(LogMessageProps::sheet_bnpcname(
                        pparam_fun.to_player(props)?,
                        *p3,
                    )?)),
                }
            }
            (
                TagName::SheetEn,
                [Param::Obj(Obj::ObjStr), Param::Num(p1), Param::Function(
                    pparam_fun @ Function {
                        name: FuncName::PlayerParameter,
                        params: _,
                    },
                ), Param::Num(p3), Param::Num(p4)],
            ) => Ok(TagValue::Text(LogMessageProps::sheet_en(
                *p1,
                pparam_fun.to_player(props)?,
                *p3,
                *p4,
            )?)),
            (name, _params) => Err(EmoteTextProcessError::InvalidTag {
                name,
                params: self.params.clone(),
            }),
        }
    }
}

#[derive(Debug, Clone)]
enum TagValue {
    Text(String),
    Bool(bool),
    None,
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
    fn process(&self, props: &LogMessageProps) -> Result<String, EmoteTextProcessError> {
        match self.name {
            FuncName::Equal => Err(EmoteTextProcessError::DanglingFunction {
                name: FuncName::Equal,
            }),
            FuncName::ObjectParameter => match self.value(props)? {
                FunctionValue::ObjString(s) => Ok(s.clone()),
                value => Err(EmoteTextProcessError::UnexpectedFuncReturn {
                    name: self.name,
                    params: self.params.clone(),
                    value: format!("{:?}", value),
                }),
            },
            FuncName::PlayerParameter => Err(EmoteTextProcessError::DanglingFunction {
                name: FuncName::PlayerParameter,
            }),
        }
    }
}

impl Function {
    fn value<'a>(
        &'a self,
        props: &'a LogMessageProps,
    ) -> Result<FunctionValue, EmoteTextProcessError> {
        match (self.name, &self.params[..]) {
            // todo determine if Equal params are always the same type or not
            (FuncName::Equal, params @ [_, _]) => match params {
                [Param::Function(f1), Param::Function(f2)] => {
                    Ok(FunctionValue::Bool(f1.value(props)? == f2.value(props)?))
                }
                _ => Err(EmoteTextProcessError::InvalidFunc {
                    name: FuncName::Equal,
                    params: self.params.clone(),
                }),
            },
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

    fn to_player<'a>(
        &'a self,
        props: &'a LogMessageProps,
    ) -> Result<&Player, EmoteTextProcessError> {
        match self.value(props)? {
            FunctionValue::PlayerParam(PParam::Player(Some(p))) => Ok(p),
            value => Err(EmoteTextProcessError::UnexpectedFuncReturn {
                name: FuncName::PlayerParameter,
                params: self.params.clone(),
                value: format!("{:?}", value),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FunctionValue<'a> {
    ObjString(&'a String),
    PlayerParam(PParam<'a>),
    Bool(bool),
}
