use super::Command;
use crate::{error::RunnerErrorKind, webdriver::Webdriver};

pub struct Assert {
    var: String,
    value: String,
}

impl Assert {
    pub fn new(variable: String, value: String) -> Self {
        Self {
            var: variable,
            value,
        }
    }
}

#[async_trait::async_trait]
impl<D: Webdriver> Command<D> for Assert {
    async fn run(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind> {
        // NOTION: intentially don't use a print_plain_value even though SELENIUM IDE uses this approach
        let var = runner.get_value(&self.var).map_or_else(
            || "undefined".to_string(),
            |v| v.to_string().trim_matches('\"').to_string(),
        );

        if var != self.value {
            return Err(RunnerErrorKind::AssertFailed {
                lhs: var,
                rhs: self.value.clone(),
            });
        }

        Ok(())
    }
}
