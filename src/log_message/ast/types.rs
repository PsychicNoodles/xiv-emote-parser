use log::*;
use strum_macros::EnumString;
use thiserror::Error;

use super::condition::{Condition, ConditionAnswer, ConditionError, DynamicText, DynamicTextError};

#[derive(Debug, Clone, Error)]
pub enum EmoteTextProcessError {
    #[error("Function used in unexpected place ({name:?})")]
    DanglingFunction { name: FuncName },
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
    #[error("Unexpected condition (not implemented?)")]
    ConditionError(#[from] ConditionError),
    #[error("Unexpected dynamic text (not implemented?)")]
    DynamicTextError(#[from] DynamicTextError),
}

#[derive(Debug, Clone)]
pub enum Text {
    Dynamic(DynamicText),
    Static(String),
}

// todo maybe come up with a better name...
#[derive(Debug, Clone)]
pub struct ConditionState {
    pub cond: Condition,
    pub is_true: bool,
}

#[derive(Debug, Clone)]
pub struct ConditionText {
    pub conds: Vec<ConditionState>,
    pub text: Text,
}

#[derive(Debug, Clone)]
pub struct ConditionTexts(Vec<ConditionText>);

impl ConditionTexts {
    /// Executes text_handler for each [Text] value of contained [ConditionText]s, filtering to only return
    /// values that are [Some] and returning the iterator result.
    pub fn map_texts<'a, F, R, C>(
        &'a self,
        cond_answer: &'a C,
        text_handler: F,
    ) -> impl Iterator<Item = R> + '_
    where
        F: Fn(&Text) -> Option<R> + 'a,
        C: ConditionAnswer,
    {
        self.0.iter().filter_map(move |ctxt| {
            let ConditionText { conds, text } = ctxt;
            if conds
                .iter()
                .all(|ConditionState { cond, is_true }| cond_answer.as_bool(cond) == *is_true)
            {
                trace!("cond = true, calling handler");
                text_handler(text)
            } else {
                trace!("cond = false, skipping handler");
                None
            }
        })
    }

    /// Executes text_handler for each [Text] value of contained [ConditionText]s
    pub fn for_each_texts<'a, F, C>(&'a self, cond_answer: &'a C, mut text_handler: F)
    where
        F: FnMut(&Text),
        C: ConditionAnswer,
    {
        self.0.iter().for_each(move |ctxt| {
            let ConditionText { conds, text } = ctxt;
            if conds
                .iter()
                .all(|ConditionState { cond, is_true }| cond_answer.as_bool(cond) == *is_true)
            {
                trace!("cond = true, calling handler");
                text_handler(text)
            } else {
                trace!("cond = false, skipping handler");
            }
        });
    }
}

trait EmoteTextProcessor {
    // todo maybe make this cow
    fn process(
        &self,
        conds: Vec<ConditionState>,
    ) -> Result<Vec<ConditionText>, EmoteTextProcessError>;
}

#[derive(Debug, Clone)]
pub struct Message(pub Vec<MessagePart>);

impl Message {
    pub fn process_string(&self) -> Result<ConditionTexts, EmoteTextProcessError> {
        self.0
            .iter()
            .map(|part| part.process(vec![]))
            .collect::<Result<Vec<_>, EmoteTextProcessError>>()
            .map(|vecs| vecs.into_iter().flatten().collect())
            .map(ConditionTexts)
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
        conds: Vec<ConditionState>,
    ) -> Result<Vec<ConditionText>, EmoteTextProcessError> {
        match self {
            MessagePart::Element(e) => e.process(conds),
            MessagePart::Text(t) => Ok(vec![ConditionText {
                conds,
                text: Text::Static(t.clone()),
            }]),
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
    fn process(
        &self,
        conds: Vec<ConditionState>,
    ) -> Result<Vec<ConditionText>, EmoteTextProcessError> {
        match self {
            Param::Element(e) => e.process(conds),
            Param::Function(f) => f.process(conds),
            Param::Obj(o) => Err(EmoteTextProcessError::UnexpectedObj { name: *o }),
            Param::Num(n) => Err(EmoteTextProcessError::UnexpectedNum { value: *n }),
        }
    }
}

#[derive(Debug, Clone)]
pub enum IfElseThen {
    Function(Function),
    Element(Element),
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
        conds: Vec<ConditionState>,
    ) -> Result<Vec<ConditionText>, EmoteTextProcessError> {
        match self {
            Element::IfElse(ie) => ie.process(conds),
            // currently can only be formed by auto_close_tag, which never has a child
            Element::Tag(t, _s) => t.process(conds),
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
    pub if_then: Vec<IfElseThen>,
    pub else_then: Vec<IfElseThen>,
}

impl EmoteTextProcessor for IfElse {
    fn process(
        &self,
        conds: Vec<ConditionState>,
    ) -> Result<Vec<ConditionText>, EmoteTextProcessError> {
        let if_cond = Condition::try_from(&self.if_cond)?;
        let mut if_conds = conds.clone();
        if_conds.push(ConditionState {
            cond: if_cond,
            is_true: true,
        });
        let mut else_conds = conds;
        else_conds.push(ConditionState {
            cond: if_cond,
            is_true: false,
        });

        let mut res = vec![];
        for then in &self.if_then {
            match then {
                IfElseThen::Function(f) => {
                    res.append(&mut f.process(if_conds.clone())?);
                }
                IfElseThen::Element(e) => {
                    res.append(&mut e.process(if_conds.clone())?);
                }
                IfElseThen::Text(t) => {
                    res.push(ConditionText {
                        conds: if_conds.clone(),
                        text: Text::Static(t.clone()),
                    });
                }
            }
        }
        for then in &self.else_then {
            match then {
                IfElseThen::Function(f) => {
                    res.append(&mut f.process(else_conds.clone())?);
                }
                IfElseThen::Element(e) => {
                    res.append(&mut e.process(else_conds.clone())?);
                }
                IfElseThen::Text(t) => {
                    res.push(ConditionText {
                        conds: else_conds.clone(),
                        text: Text::Static(t.clone()),
                    });
                }
            }
        }
        Ok(res)
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
        conds: Vec<ConditionState>,
    ) -> Result<Vec<ConditionText>, EmoteTextProcessError> {
        match (self.name, &self.params[..]) {
            // Clickable seems to always be a superfluous wrapper on the first message part
            (TagName::Clickable, [p]) => p.process(conds),
            _ => Ok(vec![ConditionText {
                conds,
                text: Text::Dynamic(DynamicText::try_from(self.clone())?),
            }]),
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
        conds: Vec<ConditionState>,
    ) -> Result<Vec<ConditionText>, EmoteTextProcessError> {
        Ok(vec![ConditionText {
            conds,
            text: Text::Dynamic(DynamicText::try_from(self.clone())?),
        }])
    }
}
