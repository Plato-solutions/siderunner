use super::Command;
use crate::{error::RunnerErrorKind, webdriver::Webdriver};

pub struct AnswerOnNextPrompt {
    answer: String,
}

impl AnswerOnNextPrompt {
    pub fn new(answer: String) -> Self {
        Self { answer }
    }
}

#[async_trait::async_trait]
impl<D: Webdriver> Command<D> for AnswerOnNextPrompt {
    async fn run(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind> {
        let override_confirm_alert = concat!(
            "var canUseLocalStorage = false; ",
            "try { canUseLocalStorage = !!window.localStorage; } catch(ex) { /* probe failed */ }",
            "var canUseJSON = false; ",
            "try { canUseJSON = !!JSON; } catch(ex) { /* probe failed */ } ",
            "if (canUseLocalStorage && canUseJSON) { ",
            "  window.localStorage.setItem('__webdriverAlerts', JSON.stringify([])); ",
            "  window.alert = function(msg) { ",
            "    var alerts = JSON.parse(window.localStorage.getItem('__webdriverAlerts')); ",
            "    alerts.push(msg); ",
            "    window.localStorage.setItem('__webdriverAlerts', JSON.stringify(alerts)); ",
            "  }; ",
            "  window.localStorage.setItem('__webdriverConfirms', JSON.stringify([])); ",
            "  if (!('__webdriverNextConfirm' in window.localStorage)) { ",
            "    window.localStorage.setItem('__webdriverNextConfirm', JSON.stringify(true)); ",
            "  } ",
            "  window.confirm = function(msg) { ",
            "    var confirms = JSON.parse(window.localStorage.getItem('__webdriverConfirms')); ",
            "    confirms.push(msg); ",
            "    window.localStorage.setItem('__webdriverConfirms', JSON.stringify(confirms)); ",
            "    var res = JSON.parse(window.localStorage.getItem('__webdriverNextConfirm')); ",
            "    window.localStorage.setItem('__webdriverNextConfirm', JSON.stringify(true)); ",
            "    return res; ",
            "  }; ",
            "} else { ",
            "  if (window.__webdriverAlerts) { return; } ",
            "  window.__webdriverAlerts = []; ",
            "  window.alert = function(msg) { window.__webdriverAlerts.push(msg); }; ",
            "  window.__webdriverConfirms = []; ",
            "  window.__webdriverNextConfirm = true; ",
            "  window.confirm = function(msg) { ",
            "    window.__webdriverConfirms.push(msg); ",
            "    var res = window.__webdriverNextConfirm; ",
            "    window.__webdriverNextConfirm = true; ",
            "    return res; ",
            "  }; ",
            "}",
        );

        let js = r"
            function answerOnNextPrompt(answer) {
                var canUseLocalStorage = false;
                    try { canUseLocalStorage = !!window.localStorage; } catch(ex) { /* probe failed */ }
                var canUseJSON = false;
                    try { canUseJSON = !!JSON; } catch(ex) { /* probe failed */ }
                if (canUseLocalStorage && canUseJSON) {
                    window.localStorage.setItem('__webdriverNextPrompt', JSON.stringify(answer));
                } else {
                    window.__webdriverNextPrompt = answer;
                }
            }";

        let js = format!("{} \n answerOnNextPrompt({:?});", js, self.answer);

        runner.exec(&override_confirm_alert).await?;
        runner.exec(&js).await?;

        Ok(())
    }
}
