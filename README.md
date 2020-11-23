# Pantheon

A library for parsing and running selenium files.

# Get started

An example of running a selenium file against wikipedia.

_You should have a webdriver started before running the snippet_

```rust
    let mut client = Client::new("http://localhost:4444")
        .await
        .expect("can't connect to webdriver");
    let wiki = std::fs::File::open("examples/wiki.side")
        .expect("can't open a wiki selenium file");
    let file = parse(wiki).expect("parsing issue");
    let mut runner = Runner::new(&client);
    runner.run(&file.tests[0]).await.expect("error occured while running a first test");
```
