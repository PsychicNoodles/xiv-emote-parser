use log::*;

use super::{
    condition::ConditionAnswer,
    types::{ConditionState, ConditionText, Text},
};

#[derive(Debug, Clone)]
pub struct ConditionTexts(Vec<ConditionText>);

impl ConditionTexts {
    pub fn new(texts: Vec<ConditionText>) -> ConditionTexts {
        ConditionTexts(texts)
    }

    /// Executes text_handler for each [Text] value of contained [ConditionText]s whose condition resolves to true,
    /// filtering to only return values that are [Some] and returning the iterator result.
    pub fn filter_map_texts<'a, F, R, C>(
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

    /// Executes text_handler for each [Text] value of contained [ConditionText]s whose condition resolves to true,
    /// consuming the [ConditionTexts] and filtering to only return values that are [Some] and returning
    /// the iterator result.
    pub fn into_filter_map_texts<'a, F, R, C>(
        self,
        cond_answer: &'a C,
        text_handler: F,
    ) -> impl Iterator<Item = R> + '_
    where
        F: Fn(Text) -> Option<R> + 'a,
        C: ConditionAnswer,
    {
        self.0.into_iter().filter_map(move |ctxt| {
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

    /// Executes text_handler for each [Text] value of contained [ConditionText]s whose condition resolves to true,
    /// filtering to only return values that are [Some] and returning the iterator result.
    pub fn filter_map_texts_mut<'a, F, R, C>(
        &'a self,
        cond_answer: &'a C,
        mut text_handler: F,
    ) -> impl Iterator<Item = R> + '_
    where
        F: FnMut(&Text) -> Option<R> + 'a,
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

    /// Executes text_handler for each [Text] value of contained [ConditionText]s whose condition resolves to true,
    /// consuming the [ConditionTexts]and filtering to only return values that are [Some] and returning
    /// the iterator result.
    pub fn into_filter_map_texts_mut<'a, F, R, C>(
        self,
        cond_answer: &'a C,
        mut text_handler: F,
    ) -> impl Iterator<Item = R> + '_
    where
        F: FnMut(Text) -> Option<R> + 'a,
        C: ConditionAnswer,
    {
        self.0.into_iter().filter_map(move |ctxt| {
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

    /// Executes text_handler for each [Text] value of contained [ConditionText]s whose condition resolves to true,
    /// returning the iterator result.
    pub fn map_texts<'a, F, R, C>(
        &'a self,
        cond_answer: &'a C,
        text_handler: F,
    ) -> impl Iterator<Item = R> + '_
    where
        F: Fn(&Text) -> R + 'a,
        C: ConditionAnswer,
    {
        self.0.iter().filter_map(move |ctxt| {
            let ConditionText { conds, text } = ctxt;
            if conds
                .iter()
                .all(|ConditionState { cond, is_true }| cond_answer.as_bool(cond) == *is_true)
            {
                trace!("cond = true, calling handler");
                Some(text_handler(text))
            } else {
                trace!("cond = false, skipping handler");
                None
            }
        })
    }

    /// Executes text_handler for each [Text] value of contained [ConditionText]s whose condition resolves to true,
    /// consuming the [ConditionTexts] and returning the iterator result.
    pub fn into_map_texts<'a, F, R, C>(
        self,
        cond_answer: &'a C,
        text_handler: F,
    ) -> impl Iterator<Item = R> + '_
    where
        F: Fn(Text) -> R + 'a,
        C: ConditionAnswer,
    {
        self.0.into_iter().filter_map(move |ctxt| {
            let ConditionText { conds, text } = ctxt;
            if conds
                .iter()
                .all(|ConditionState { cond, is_true }| cond_answer.as_bool(cond) == *is_true)
            {
                trace!("cond = true, calling handler");
                Some(text_handler(text))
            } else {
                trace!("cond = false, skipping handler");
                None
            }
        })
    }

    /// Executes text_handler for each [Text] value of contained [ConditionText]s whose condition resolves to true,
    /// returning the iterator result.
    pub fn map_texts_mut<'a, F, R, C>(
        &'a self,
        cond_answer: &'a C,
        mut text_handler: F,
    ) -> impl Iterator<Item = R> + '_
    where
        F: FnMut(&Text) -> R + 'a,
        C: ConditionAnswer,
    {
        self.0.iter().filter_map(move |ctxt| {
            let ConditionText { conds, text } = ctxt;
            if conds
                .iter()
                .all(|ConditionState { cond, is_true }| cond_answer.as_bool(cond) == *is_true)
            {
                trace!("cond = true, calling handler");
                Some(text_handler(text))
            } else {
                trace!("cond = false, skipping handler");
                None
            }
        })
    }

    /// Executes text_handler for each [Text] value of contained [ConditionText]s whose condition resolves to true,
    /// consuming the [ConditionTexts] and returning the iterator result.
    pub fn into_map_texts_mut<'a, F, R, C>(
        self,
        cond_answer: &'a C,
        mut text_handler: F,
    ) -> impl Iterator<Item = R> + '_
    where
        F: FnMut(Text) -> R + 'a,
        C: ConditionAnswer,
    {
        self.0.into_iter().filter_map(move |ctxt| {
            let ConditionText { conds, text } = ctxt;
            if conds
                .iter()
                .all(|ConditionState { cond, is_true }| cond_answer.as_bool(cond) == *is_true)
            {
                trace!("cond = true, calling handler");
                Some(text_handler(text))
            } else {
                trace!("cond = false, skipping handler");
                None
            }
        })
    }

    /// Executes text_handler for each [Text] value of contained [ConditionText]s whose condition resolves to true
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

    /// Executes text_handler for each [Text] value of contained [ConditionText]s whose condition resolves to true,
    /// consuming the [ConditionTexts]
    pub fn into_for_each_texts<'a, F, C>(self, cond_answer: &'a C, mut text_handler: F)
    where
        F: FnMut(Text),
        C: ConditionAnswer,
    {
        self.0.into_iter().for_each(move |ctxt| {
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

    pub fn into_inner(self) -> Vec<ConditionText> {
        self.0
    }
}
