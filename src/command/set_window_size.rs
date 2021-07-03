use super::Command;
use crate::{error::RunnerErrorKind, webdriver::Webdriver};

pub struct SetWindowSize {
    width: u32,
    heigth: u32,
}

impl SetWindowSize {
    pub fn new(width: u32, heigth: u32) -> Self {
        Self { width, heigth }
    }
}

#[async_trait::async_trait]
impl<D: Webdriver> Command<D> for SetWindowSize {
    async fn run(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind> {
        runner
            .get_webdriver()
            .set_window_size(self.width, self.heigth)
            .await?;

        Ok(())
    }
}
