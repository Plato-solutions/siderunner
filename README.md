<h6 align="center">
    
[![Build Status](https://img.shields.io/github/workflow/status/Plato-solutions/siderunner/Continuous%20integration?style=flat-square)](https://github.com/Plato-solutions/siderunner/actions)
[![Crates.io](https://img.shields.io/crates/v/siderunner.svg?style=flat-square)](https://crates.io/crates/siderunner)
[![Docs.rs](https://img.shields.io/badge/docs.rs-siderunner-blue?style=flat-square)](https://docs.rs/siderunner)
[![CodeCov](https://img.shields.io/codecov/c/github/Plato-solutions/siderunner/master?style=flat-square)](https://app.codecov.io/gh/Plato-solutions/siderunner)
[![License:MPL-2.0](https://img.shields.io/badge/License-MPL_2.0-yellow.svg?style=flat-square)](https://opensource.org/licenses/MPL-2.0)
    
</h6>


<h1 align="center">
    Siderunner
</h1>
    
A library for parsing and running selenium `.side` files used in a [`Selenium IDE`].

# Get started

To run a file you should initially parse it and then you can run a test by its index.

```rust
use siderunner::{parse, Runner};
use thirtyfour::{DesiredCapabilities, WebDriver};
use serde_json::json;

let wiki = std::fs::File::open("examples/wiki.side").expect("Can't open a side file");
let file = parse(wiki).expect("parsing can't be done...");

let client = WebDriver::new("http://localhost:4444", DesiredCapabilities::firefox())
    .await
    .expect("can't connect to webdriver");

let mut runner = Runner::new(&client);
runner.run(&file).await.expect("Error occured while running a side file");

assert_eq!(
    runner.get_value("slogan"),
    Some(&json!("The Free Encyclopedia")),
);

runner.close().await.unwrap();
```

A `.side` file for the example can be found in example directory.

## Backends

`siderunner` supports 2 backends:

* [`thirtyfour`] - default
* [`fantoccini`]

You can tweak `fantoccini` backend by providing a feature `fantoccini_backend` and turn off default features, `default-features = false`

## Supported commands

[`Selenium IDE`] supports the following [commands](https://www.selenium.dev/selenium-ide/docs/en/api/commands).

- [x] add selection
- [x] answer on next prompt
- [x] assert
- [x] assert alert
- [x] assert checked
- [x] assert confirmation
- [ ] assert editable
- [ ] assert element present
- [ ] assert element not present
- [x] assert not checked
- [ ] assert not editable
- [x] assert not selected value
- [x] assert not text
- [x] assert prompt
- [x] assert selected value
- [x] assert selected label
- [x] assert text
- [x] assert title
- [x] assert value
- [x] check
- [x] choose cancel on next confirmation
- [x] choose cancel on next prompt
- [x] choose ok on next confirmation
- [x] click
- [ ] click at
- [x] close
- [ ] debugger
- [x] do
- [x] double click
- [ ] double click at
- [ ] drag and drop to object
- [x] echo
- [x] edit content
- [x] else
- [x] else if
- [x] end
- [x] execute script
- [x] execute async script
- [x] for each
- [x] if
- [x] mouse down
- [ ] mouse down at
- [ ] mouse move at
- [ ] mouse out
- [ ] mouse over
- [x] mouse up
- [ ] mouse up at
- [x] open
- [x] pause
- [ ] remove selection
- [x] repeat if
- [ ] run
- [x] run script
- [x] select
- [ ] select frame
- [ ] select window
- [x] send keys
- [ ] set speed
- [x] set window size
- [x] store
- [ ] store attribute
- [ ] store json
- [x] store text
- [x] store title
- [ ] store value
- [ ] store window handle
- [x] store xpath count
- [ ] submit
- [x] times
- [x] type
- [x] uncheck
- [ ] verify
- [ ] verify checked
- [ ] verify editable
- [ ] verify element present
- [ ] verify element not present
- [ ] verify not checked
- [ ] verify not editable
- [ ] verify not selected value
- [ ] verify not text
- [ ] verify selected label
- [ ] verify selected value
- [ ] verify text
- [ ] verify title
- [ ] verify value
- [x] wait for element editable
- [ ] wait for element not editable
- [x] wait for element not present
- [ ] wait for element not visible
- [x] wait for element present
- [ ] wait for element visible
- [ ] webdriver answer on visible prompt
- [ ] webdriver choose cancel on visible confirmation
- [ ] webdriver choose cancel on visible prompt
- [ ] webdriver choose ok on visible confirmation
- [x] while

## Development

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

## Contributing

All contributions are welcomed.

I would recomend to start by tackling some of not implemented commands. And there's one more good place to start is [`fantoccini`] backend, as it got a bit outdated as there's a bunch of not implemented commands there compared to default backend.

There might be something to do in the backend repos so you can help them out as well.

[`Selenium IDE`]: https://www.selenium.dev/selenium-ide/
[`thirtyfour`]: https://github.com/stevepryde/thirtyfour
[`fantoccini`]: https://github.com/jonhoo/fantoccini
