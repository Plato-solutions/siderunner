use crate::{error::RunnerErrorKind, runner::Runner, webdriver::Webdriver};

mod answer_on_next_prompt;
mod assert;
mod assert_alert;
mod assert_checked;
mod assert_prompt;
mod click;
mod close;
mod echo;
mod execute;
mod execute_async;
mod open;
mod pause;
mod run_script;
mod select;
mod set_window_size;
mod store;
mod store_text;
mod store_xpath_count;
mod wait_for_element_editable;
mod wait_for_element_not_present;
mod wait_for_element_present;
mod wait_for_element_visible;

pub use {
    answer_on_next_prompt::*, assert::*, assert_alert::*, assert_checked::*, assert_prompt::*,
    click::*, close::*, echo::*, execute::*, execute_async::*, open::*, pause::*, run_script::*,
    select::*, set_window_size::*, store::*, store_text::*, store_xpath_count::*,
    wait_for_element_editable::*, wait_for_element_not_present::*, wait_for_element_present::*,
    wait_for_element_visible::*,
};

#[async_trait::async_trait]
pub trait Command<D>
where
    D: Webdriver,
{
    async fn run(&self, runner: &mut Runner<D>) -> Result<(), RunnerErrorKind>;
}
