use crate::{
    error::RunnerErrorKind,
    runner::Runner,
    webdriver::{self, Webdriver},
};

pub mod answer_on_next_prompt;
pub mod assert;
pub mod assert_alert;
pub mod assert_checked;
pub mod click;
pub mod close;
pub mod echo;
pub mod execute;
pub mod execute_async;
pub mod open;
pub mod pause;
pub mod run_script;
pub mod select;
pub mod set_window_size;
pub mod store;
pub mod store_text;
pub mod store_xpath_count;
pub mod wait_for_element_editable;
pub mod wait_for_element_not_present;
pub mod wait_for_element_present;
pub mod wait_for_element_visible;

#[async_trait::async_trait]
pub trait Command {
    async fn run<D, E>(&self, runner: &mut Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver<Element = E, Error = RunnerErrorKind> + Send,
        E: webdriver::Element<Driver = D, Error = RunnerErrorKind> + Send;
}
