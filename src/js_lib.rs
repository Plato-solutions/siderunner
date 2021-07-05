use crate::{error::RunnerErrorKind, runner::Runner, webdriver::Webdriver};

macro_rules! include_func {
    ($file:expr $(,)?) => {{
        #[cfg(unix)]
        {
            include_str!(concat!("../js_lib/", $file))
        }
        #[cfg(windows)]
        {
            include_str!(concat!("..\\js_lib\\", $file))
        }
    }};
}

const REPLACE_ALERT_METHOD: &str = include_func!("replaceAlertMethod.js");
const ANSWER_ON_NEXT_PROMPT: &str = include_func!("answerOnNextPrompt.js");
const SET_NEXT_CONFIRMATION_STATE: &str = include_func!("setNextConfirmationState.js");

pub async fn answer_on_next_prompt<D>(
    runner: &mut Runner<D>,
    answer: &str,
) -> Result<(), RunnerErrorKind>
where
    D: Webdriver,
{
    let code = format!(
        "{}{} replaceAlertMethod(null); answerOnNextPrompt({:?});",
        REPLACE_ALERT_METHOD, ANSWER_ON_NEXT_PROMPT, answer
    );

    runner.exec(&code).await?;
    Ok(())
}

pub async fn choose_cancel_on_next_confirmation<D>(
    runner: &mut Runner<D>,
) -> Result<(), RunnerErrorKind>
where
    D: Webdriver,
{
    let code = format!(
        "{}{} replaceAlertMethod(null); setNextConfirmationState({});",
        REPLACE_ALERT_METHOD, SET_NEXT_CONFIRMATION_STATE, false
    );

    runner.exec(&code).await?;
    Ok(())
}

pub async fn choose_ok_on_next_confirmation<D>(
    runner: &mut Runner<D>,
) -> Result<(), RunnerErrorKind>
where
    D: Webdriver,
{
    let code = format!(
        "{}{} replaceAlertMethod(null); setNextConfirmationState({});",
        REPLACE_ALERT_METHOD, SET_NEXT_CONFIRMATION_STATE, true
    );

    runner.exec(&code).await?;
    Ok(())
}
