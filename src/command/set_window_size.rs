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
impl Command for SetWindowSize {
    async fn run<D>(&self, runner: &mut crate::runner::Runner<D>) -> Result<(), RunnerErrorKind>
    where
        D: Webdriver,
    {
        runner
            .get_webdriver()
            .set_window_size(self.width, self.heigth)
            .await?;

        Ok(())
    }
}
