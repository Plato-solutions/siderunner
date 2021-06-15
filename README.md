[![Crates.io](https://img.shields.io/crates/v/siderunner.svg?style=flat-square)](https://crates.io/crates/thirtyfour)
[![docs.rs](https://img.shields.io/badge/docs.rs-siderunner-blue?style=flat-square)](https://docs.rs/thirtyfour)
[![Build Status](https://img.shields.io/github/workflow/status/Plato-solutions/siderunner/build-check/master?style=flat-square)](https://github.com/Plato-solutions/siderunner/actions)

# Siderunner

A library for parsing and running selenium files.

# Get started

```rust
use siderunner::{parse, Runner};
use thirtyfour::{DesiredCapabilities, WebDriver};

let wiki = std::fs::File::open("examples/wiki.side").unwrap();
let file = parse(wiki).expect("parsing can't be done...");

let client = WebDriver::new("http://localhost:4444", DesiredCapabilities::firefox())
    .await
    .expect("can't connect to webdriver");

let mut runner = Runner::new(&client);
runner.run(&file.tests[0]).await.unwrap();

assert_eq!(
    runner.data.get("slogan"),
    Some(&serde_json::json!("The Free Encyclopedia")),
);

runner.close().await.unwrap();
```

## Backends

`siderunner` supports 2 backends:

* [`thirtyfour`](https://github.com/stevepryde/thirtyfour) - default
* [`fantoccini`](https://github.com/jonhoo/fantoccini)

#### Notion that currently not all [commands](https://www.selenium.dev/selenium-ide/docs/en/api/commands) are covered.

## Testing

### Unit tests

```
cargo test --lib
```

### Integrational tests

To run a integration test suit you must set an environment.
You can use `test.bash` file to run tests and manage the environment.
Just run it.

```
./test.bash
```

#### Requirements

* `docker-compose`
