//! Intermediate type, post pest rule processing

use super::{LogMessageParser, Rule};
use pest_consume::{match_nodes, Error};
use std::str::FromStr;
use strum_macros::EnumString;

type Result<T> = std::result::Result<T, Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

#[derive(Debug, Clone)]
pub struct Message(Vec<MessagePart>);

#[derive(Debug, Clone)]
pub enum MessagePart {
    Element(Element),
    Text(String),
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

#[derive(Debug, Clone)]
pub struct IfElse {
    if_cond: Param,
    if_then: IfElseThen,
    else_then: IfElseThen,
}

#[derive(Debug, Clone, EnumString, PartialEq, Eq)]
pub enum TagName {
    Clickable,
    Sheet,
    SheetEn,
}

#[derive(Debug, Clone)]
pub struct Tag {
    name: TagName,
    params: Vec<Param>,
}

#[derive(Debug, Clone, EnumString, PartialEq, Eq)]
pub enum FuncName {
    Equal,
    ObjectParameter,
    PlayerParameter,
}

#[derive(Debug, Clone)]
pub struct Function {
    name: FuncName,
    params: Vec<Param>,
}

#[pest_consume::parser]
impl LogMessageParser {
    fn EOI(_input: Node) -> Result<()> {
        Ok(())
    }

    fn text(input: Node) -> Result<String> {
        Ok(input.as_str().to_string())
    }

    fn tag_name(input: Node) -> Result<TagName> {
        TagName::from_str(input.as_str()).map_err(|e| input.error(e))
    }

    fn func_name(input: Node) -> Result<FuncName> {
        FuncName::from_str(input.as_str()).map_err(|e| input.error(e))
    }

    fn param_num(input: Node) -> Result<u64> {
        input.as_str().parse().map_err(|e| input.error(e))
    }

    fn param_obj(input: Node) -> Result<String> {
        Ok(input.as_str().to_string())
    }

    fn open_tag(input: Node) -> Result<Tag> {
        Ok(match_nodes!(input.into_children();
            [tag_name(name), param(params)..] => Tag { name, params: params.collect() }
        ))
    }

    fn auto_closing_tag(input: Node) -> Result<Tag> {
        Ok(match_nodes!(input.into_children();
            [tag_name(name), param(params)..] => Tag { name, params: params.collect() }
        ))
    }

    fn close_tag(input: Node) -> Result<TagName> {
        Ok(match_nodes!(input.into_children();
            [tag_name(name)] => name
        ))
    }

    fn if_else_then(input: Node) -> Result<IfElseThen> {
        Ok(match_nodes!(input.into_children();
            [param(param)] => IfElseThen::Param(param),
            [text(text)] => IfElseThen::Text(text),
        ))
    }

    fn if_else_element(input: Node) -> Result<Box<IfElse>> {
        Ok(match_nodes!(input.into_children();
            [param(if_cond), if_else_then(if_then), if_else_then(else_then)] =>
                Box::new(IfElse { if_cond, if_then, else_then })
        ))
    }

    fn element(input: Node) -> Result<Element> {
        // lose input when calling into_children, so create this in advance just in case
        let nonmatch_err = Err(input.error("open and close tags do not match"));

        Ok(match_nodes!(input.into_children();
            [if_else_element(if_else)] => Element::IfElse(if_else),
            [open_tag(tag), text(mut text).., close_tag(close_tag)] => if tag.name == close_tag {
                Element::Tag(tag, text.next())
            } else {
                return nonmatch_err;
            },
            [auto_closing_tag(tag)] => Element::Tag(tag, None)
        ))
    }

    fn function(input: Node) -> Result<Function> {
        Ok(match_nodes!(input.into_children();
            [func_name(name), param(params)..] => Function { name, params: params.collect() }
        ))
    }

    fn param(input: Node) -> Result<Param> {
        Ok(match_nodes!(input.into_children();
            [element(element)] => Param::Element(element),
            [param_num(num)] => Param::Num(num),
            [param_obj(obj)] => Param::Obj(obj),
            [function(func)] => Param::Function(func)
        ))
    }

    fn message_part(input: Node) -> Result<MessagePart> {
        Ok(match_nodes!(input.into_children();
            [text(text)] => MessagePart::Text(text),
            [element(element)] => MessagePart::Element(element)
        ))
    }

    pub fn message(input: Node) -> Result<Message> {
        Ok(match_nodes!(input.into_children();
            [message_part(parts).., EOI] => Message(parts.collect())
        ))
    }
}
