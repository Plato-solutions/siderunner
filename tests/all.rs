use siderunner::{parse, Runner};
use std::{fs::File, thread};
use thirtyfour::{Capabilities, DesiredCapabilities, WebDriver, WebDriverCommands};
use tokio::test;

async fn testing(path: &str) {
    let mut file = File::open(path).expect("Failed to read a file");
    let side_file = parse(&mut file).expect("Failed to parse a file");

    let mut cops = DesiredCapabilities::chrome();
    cops.set_headless()
        .expect("Failed to set a headless setting");
    cops.set_unexpected_alert_behaviour(thirtyfour::AlertBehaviour::Ignore)
        .expect("Failed to set an option setting");
    let wb = WebDriver::new("http://localhost:4444/wd/hub", cops)
        .await
        .expect("Failed to create a webdriver");

    let mut runner = Runner::new(&wb);
    match runner.run(&side_file).await {
        Ok(()) => {}
        Err(err) => {
            wb.quit().await.expect("Failed to stop a webdriver");

            // TODO: change command interface to not lose all information
            // let test = side_file
            //     .tests
            //     .iter()
            //     .find(|test| test.name.as_str() == err.test.as_ref().unwrap())
            //     .unwrap();
            // let failed_command = &test.commands[err.index];
            // if failed_command.comment == "FAIL" {
            //     // it's OK
            // }

            panic!("Failed to run a file {:?} test: {:?}", path, err);
        }
    }

    wb.quit().await.expect("Failed to stop a webdriver");
}

macro_rules! test_file {
    ( $test_file:expr, $test_name:ident ) => {
        #[test]
        async fn $test_name() {
            testing($test_file).await;
        }
    };
}

test_file!("tests/resources/basic/test.side.json", basic);
test_file!(
    "tests/resources/open relative url/test.side.json",
    open_relative_url
);
test_file!(
    "tests/resources/commands/assert/test.side.json",
    command_assert
);
test_file!(
    "tests/resources/commands/click/test.side.json",
    command_click
);
test_file!(
    "tests/resources/commands/execute/test.side.json",
    command_execute
);
test_file!(
    "tests/resources/commands/execute async/test.side.json",
    command_execute_async
);
test_file!(
    "tests/resources/commands/run script/test.side.json",
    command_run_script
);
test_file!(
    "tests/resources/commands/for each/test.side.json",
    command_for_each
);
test_file!(
    "tests/resources/commands/add selection/test.side.json",
    command_add_selection
);
test_file!(
    "tests/resources/commands/answer on next prompt/test.side.json",
    command_answer_on_next_prompt
);

#[cfg(not(feature = "fantoccini_backend"))]
test_file!(
    "tests/resources/commands/assert alert/test.side.json",
    command_assert_alert
);
