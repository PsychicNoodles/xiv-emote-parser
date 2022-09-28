use pest::iterators::Pairs;

use super::{props::LogMessageProps, EmoteText, EmoteTextError, Rule, TargetMessages};

// certain rules must be followed by other specific rules
enum PrevRule {
    TagName,
    FuncName,
}

fn process_pairs(
    mut pairs: Pairs<Rule>,
    params: LogMessageProps,
    targets: TargetMessages,
) -> Result<EmoteText, EmoteTextError> {
    let mut r = EmoteText::new();
    let mut prev = None;

    while let Some(p) = pairs.next() {
        match p.as_rule() {
            // next iteration should naturally exit
            Rule::EOI => {}
            Rule::text => r.push_targets(targets, p.as_str()),
            Rule::tag_name => prev = Some(PrevRule::TagName),
            Rule::func_name => prev = Some(PrevRule::FuncName),
            Rule::param_num => todo!(),
            Rule::param_obj => todo!(),
            Rule::open_tag => todo!(),
            Rule::auto_closing_tag => todo!(),
            Rule::close_tag => todo!(),
            Rule::if_else_element => todo!(),
            Rule::element => todo!(),
            Rule::function => todo!(),
            Rule::param => todo!(),
            Rule::message => todo!(),

            // tags
            Rule::tag_clickable => todo!(),
            Rule::tag_sheet => todo!(),
            Rule::tag_sheet_en => todo!(),

            // functions
            Rule::func_equal => todo!(),
            Rule::func_obj_param => todo!(),
            Rule::func_player_param => todo!(),
        }
    }

    todo!()
}
