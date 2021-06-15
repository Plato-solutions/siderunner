/// The example requires a webdriver have been run

#[cfg(feature = "thirtyfour_backend")]
fn main() {
    panic!("This example requires 'fantoccini_backend' feature")
}

#[cfg(feature = "fantoccini_backend")]
#[tokio::main]
async fn main() {
    use fantoccini::Client;
    use siderunner::{parse, Runner};

    let client = Client::new("http://localhost:4444")
        .await
        .expect("can't connect to webdriver");
    let wiki = std::fs::File::open("examples/wiki.side").unwrap();
    let file = parse(wiki).expect("parsing can't be done...");
    let mut runner = Runner::new(client);
    runner.run(&file.tests[0]).await.unwrap();

    assert_eq!(
        runner.get_data().get("slogan"),
        Some(&serde_json::Value::String(
            "The Free Encyclopedia".to_owned()
        ))
    );

    runner.close().await.unwrap();
}
