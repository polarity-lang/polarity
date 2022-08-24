use std::borrow::Cow;

use reedline::{Prompt, PromptEditMode, PromptHistorySearchStatus, PromptViMode};

#[derive(Default)]
pub struct CustomPrompt;

const PROMPT_INDICATOR: &str = "> ";
const VI_INSERT_PROMPT_INDICATOR: &str = ": ";
const VI_NORMAL_PROMPT_INDICATOR: &str = "> ";
const MULTILINE_INDICATOR: &str = "::: ";

impl Prompt for CustomPrompt {
    fn render_prompt_left(&self) -> std::borrow::Cow<str> {
        Cow::Owned(String::from(""))
    }

    fn render_prompt_right(&self) -> std::borrow::Cow<str> {
        Cow::Owned(String::from(""))
    }

    fn render_prompt_indicator(
        &self,
        prompt_mode: reedline::PromptEditMode,
    ) -> std::borrow::Cow<str> {
        match prompt_mode {
            PromptEditMode::Default | PromptEditMode::Emacs => PROMPT_INDICATOR.into(),
            PromptEditMode::Vi(vi_mode) => match vi_mode {
                PromptViMode::Normal => VI_NORMAL_PROMPT_INDICATOR.into(),
                PromptViMode::Insert => VI_INSERT_PROMPT_INDICATOR.into(),
            },
            PromptEditMode::Custom(str) => format!("({})", str).into(),
        }
    }

    fn render_prompt_multiline_indicator(&self) -> std::borrow::Cow<str> {
        Cow::Borrowed(MULTILINE_INDICATOR)
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: reedline::PromptHistorySearch,
    ) -> std::borrow::Cow<str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };
        Cow::Owned(format!("({}reverse-search: {}) ", prefix, history_search.term))
    }
}
