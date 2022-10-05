use super::types::*;
use crate::log_message::parser::{LogMessageParser, Rule};

use pest_consume::{match_nodes, Error};
use std::str::FromStr;

type Result<T> = std::result::Result<T, Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

#[pest_consume::parser]
impl LogMessageParser {
    #[allow(dead_code)]
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

    fn param_num(input: Node) -> Result<u32> {
        input.as_str().parse().map_err(|e| input.error(e))
    }

    fn param_obj(input: Node) -> Result<Obj> {
        Obj::from_str(input.as_str()).map_err(|e| input.error(e))
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

    fn if_param(input: Node) -> Result<IfParam> {
        Ok(match_nodes!(input.into_children();
            [function(fun)] => IfParam::Function(fun),
            [auto_closing_tag(tag)] => IfParam::Tag(tag)
        ))
    }

    fn if_else_element(input: Node) -> Result<Box<IfElse>> {
        Ok(match_nodes!(input.into_children();
            [if_param(if_cond), if_else_then(if_then), if_else_then(else_then)] =>
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
            [message_part(parts).., _] => Message(parts.collect())
        ))
    }
}
