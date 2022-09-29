use pest::iterators::Pairs;

use super::{props::LogMessageProps, EmoteText, EmoteTextError, Rule, TargetMessages};

type Result = std::result::Result<EmoteText, EmoteTextError>;

fn process_pairs(
    mut pairs: Pairs<Rule>,
    params: LogMessageProps,
    targets: TargetMessages,
) -> Result {
    let mut r = EmoteText::new();

    while let Some(p) = pairs.next() {
        // match p.as_rule() {
        //     // next iteration should naturally exit
        //     Rule::EOI => {}
        //     Rule::text => r.push_targets(targets, p.as_str()),
        //     Rule::tag_name => r = process_tag(&mut pairs, &params, targets, r)?,
        //     Rule::func_name => r = process_func(&mut pairs, &params, targets, r)?,
        //     Rule::param_num => todo!(),
        //     Rule::param_obj => todo!(),
        //     Rule::open_tag => todo!(),
        //     Rule::auto_closing_tag => todo!(),
        //     Rule::close_tag => todo!(),
        //     Rule::if_else_element => todo!(),
        //     Rule::element => todo!(),
        //     Rule::function => todo!(),
        //     Rule::param => todo!(),
        //     Rule::message => todo!(),

        //     // processed during

        //     // tags
        //     Rule::tag_clickable => todo!(),
        //     Rule::tag_sheet => todo!(),
        //     Rule::tag_sheet_en => todo!(),

        //     // functions
        //     Rule::func_equal => todo!(),
        //     Rule::func_obj_param => todo!(),
        //     Rule::func_player_param => todo!(),
        // }
    }

    todo!()
}

fn process_tag(
    pairs: &mut Pairs<Rule>,
    params: &LogMessageProps,
    targets: TargetMessages,
    emote_text: EmoteText,
) -> Result {
    todo!()
}

fn process_func(
    pairs: &mut Pairs<Rule>,
    params: &LogMessageProps,
    targets: TargetMessages,
    emote_text: EmoteText,
) -> Result {
    todo!()
}
