use fantoccini::Client;
/// The example requires to geckodriver have been run
use pantheon::{parse, Result, Runner};

#[tokio::main]
async fn main() {
    let mut client = Client::new("http://localhost:4444")
        .await
        .expect("can't connect to webdriver");
    let wiki = std::fs::File::open("examples/wiki.side").unwrap();
    let tests = parse(wiki).expect("parsing can't be done...");
    let mut runner = Runner::new(&mut client);
    runner.run(&tests[0]).await.unwrap();

    assert_eq!(
        runner.data.get("slogan"),
        Some(&serde_json::Value::String(
            "The Free Encyclopedia".to_owned()
        ))
    );

    client.close().await.unwrap();
}
