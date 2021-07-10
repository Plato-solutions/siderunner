// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{error::RunnerErrorKind, runner::Runner, webdriver::Webdriver};

mod answer_on_next_prompt;
mod assert;
mod assert_alert;
mod assert_checked;
mod assert_confirmation;
mod assert_editable;
mod assert_element_present;
mod assert_prompt;
mod assert_selected_label;
mod assert_selected_value;
mod assert_text;
mod assert_title;
mod assert_value;
mod check;
mod choose_on_next_;
mod click;
mod close;
mod double_click;
mod echo;
mod edit_content;
mod execute;
mod execute_async;
mod mouse;
mod open;
mod pause;
mod remove_selection;
mod run_script;
mod select;
mod send_keys;
mod set_window_size;
mod store;
mod store_attribute;
mod store_json;
mod store_text;
mod store_title;
mod store_value;
mod store_xpath_count;
mod type_;
mod wait_for_element_editable;
mod wait_for_element_present;
mod wait_for_element_visible;

pub use {
    answer_on_next_prompt::*, assert::*, assert_alert::*, assert_checked::*,
    assert_confirmation::*, assert_editable::*, assert_element_present::*, assert_prompt::*,
    assert_selected_label::*, assert_selected_value::*, assert_text::*, assert_title::*,
    assert_value::*, check::*, choose_on_next_::*, click::*, close::*, double_click::*, echo::*,
    edit_content::*, execute::*, execute_async::*, mouse::*, open::*, pause::*,
    remove_selection::*, run_script::*, select::*, send_keys::*, set_window_size::*, store::*,
    store_attribute::*, store_json::*, store_text::*, store_title::*, store_value::*,
    store_xpath_count::*, type_::*, wait_for_element_editable::*, wait_for_element_present::*,
    wait_for_element_visible::*,
};

#[async_trait::async_trait]
pub trait Command<D>
where
    D: Webdriver,
{
    async fn run(&self, runner: &mut Runner<D>) -> Result<(), RunnerErrorKind>;
}
