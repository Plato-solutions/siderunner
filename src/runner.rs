// TODO: Mock webdriver
// TODO: interface for fantocini and possibly choose webdriver provider by feature
// TODO: provide more direct error location test + command + location(can be determined just by section (target/value etc.)) + cause
// TODO: Runner may contains basic information to handle relative url

use crate::{error::SideRunnerError, Command, Location, Result, SelectLocator, Test};
use fantoccini::{Client, Locator};
use serde_json::Value;
use std::collections::HashMap;

pub struct Runner<'driver> {
    webdriver: &'driver mut Client,
    pub data: HashMap<String, Value>,
}

impl<'driver> Runner<'driver> {
    pub fn new(client: &'driver mut Client) -> Self {
        Self {
            webdriver: client,
            data: HashMap::new(),
        }
    }

    pub async fn run(&mut self, test: &Test) -> Result<()> {
        for cmd in &test.commands {
            self.run_command(cmd).await?;
        }

        Ok(())
    }

    async fn run_command(&mut self, cmd: &Command) -> Result<()> {
        // TODO: emit variables in value field too
        match cmd {
            Command::Open(url) => {
                self.webdriver.goto(url).await?;
                let url = self.webdriver.current_url().await?;
                assert_eq!(url.as_ref(), url.as_ref());
            }
            Command::StoreText {
                var,
                target,
                targets,
            } => {
                let location = match &target.location {
                    Location::Css(css) => {
                        Location::Css(emit_variables::<_, PlainPrinter>(css, &self.data))
                    }
                    Location::Id(id) => {
                        Location::Id(emit_variables::<_, PlainPrinter>(id, &self.data))
                    }
                    Location::XPath(path) => {
                        Location::XPath(emit_variables::<_, PlainPrinter>(path, &self.data))
                    }
                };

                let locator = match &location {
                    Location::Css(css) => Locator::Css(&css),
                    Location::Id(id) => Locator::Id(&id),
                    Location::XPath(path) => Locator::XPath(&path),
                };

                let value = self.webdriver.find(locator).await?.text().await?;

                let value = Value::String(value);
                self.data.insert(var.clone(), value);

                // TODO: if `target` not found we should look up targets?
            }
            Command::Execute { script, var } => {
                // TODO: the logic is different from Selenium IDE
                // If the element is not loaded on the page IDE will fail not emidiately but our implementation will.
                // they might wait a little bit or something but there's something there

                let (script, used_vars) = emit_variables_custom(script, &self.data);
                let args = used_vars.iter().map(|var| self.data[var].clone()).collect();
                let res = self
                    .webdriver
                    .execute(
                        &format!("return (function(arguments) {{ {} }})(arguments)", script),
                        args,
                    )
                    .await;
                match res {
                    Ok(res) => match var {
                        Some(var) => {
                            self.data.insert(var.clone(), res);
                        }
                        None => (),
                    },
                    Err(err) => println!("Exec errr {}", err),
                }
            }
            Command::Echo(text) => {
                let text = emit_variables::<_, PlainPrinter>(text, &self.data);
                println!("{}", text);
            }
            Command::WaitForElementVisible { timeout, target } => {
                // todo: implemented wrongly
                // it's implmenetation more suited for WaitForElementPresent
                //
                // TODO: timout implementation is a bit wrong since we need to 'gracefully' stop running feature
                let locator = match &target.location {
                    Location::Css(css) => Locator::Css(&css),
                    Location::Id(id) => Locator::Id(&id),
                    Location::XPath(path) => Locator::XPath(&path),
                };

                let timeout = std::time::Duration::from_millis(*timeout);

                match tokio::time::timeout(timeout, self.webdriver.wait_for_find(locator)).await {
                    Ok(Err(..)) => println!("Error"),
                    Ok(..) => (),
                    Err(..) => println!("timeout"),
                }
            }
            Command::WaitForElementEditable { timeout, target } => {
                std::thread::sleep_ms(10000);
                // TODO: #issue https://github.com/jonhoo/fantoccini/issues/93
            }
            Command::Select { locator, target } => {
                let select_locator = match &target.location {
                    Location::Css(css) => Locator::Css(&css),
                    Location::Id(id) => Locator::Id(&id),
                    Location::XPath(path) => Locator::XPath(&path),
                };

                let select = self.webdriver.find(select_locator).await?;
                match locator {
                    SelectLocator::Index(index) => {
                        let index = emit_variables::<_, PlainPrinter>(index, &self.data);
                        match index.parse() {
                            Ok(index) => select.select_by_index(index).await?,
                            Err(..) => Err(SideRunnerError::MismatchedType(format!(
                                "expected to get int type but got {:?}",
                                index
                            )))?,
                        }
                    }
                };
            }
        };

        Ok(())
    }
    // argument[0] -> argument[1] -> argument[2] goes to implementing JS formatting
}

fn emit_variables<S: AsRef<str>, P: VarPrinter>(
    text: S,
    variables: &HashMap<String, Value>,
) -> String {
    // TODO: check how to emit string in quotes or not
    //
    // regex look up for variable name in brackets #{var}
    // it exclude " sign to manage cases like ${var} }
    // it's important in emiting vars in JSON
    let re = regex::Regex::new(r#"\$\{(\w+?)\}"#).unwrap();
    let replacer = VarReplacer::<P> {
        data: variables,
        printer: std::marker::PhantomData::default(),
    };

    let new_text = re.replace_all(text.as_ref(), replacer);

    new_text.into()
}

fn emit_variables_custom<S: AsRef<str>>(
    text: S,
    variables: &HashMap<String, Value>,
) -> (String, Vec<String>) {
    // TODO: check how to emit string in quotes or not
    //
    // regex look up for variable name in brackets #{var}
    // it exclude " sign to manage cases like ${var} }
    // it's important in emiting vars in JSON
    //
    // https://github.com/SeleniumHQ/selenium-ide/blob/dd0c8ce313171672d2f0670cfb05786611f85b73/packages/side-runtime/src/preprocessors.js#L119
    let re = regex::Regex::new(r#"\$\{(.*?)\}"#).unwrap();
    let mut replacer = PuttingArg {
        emited_vars: HashMap::new(),
        index: 0,
    };

    let new_text = re.replace_all(text.as_ref(), &mut replacer);

    let count_positions = replacer.index;
    let mut vars = Vec::new();
    for i in 0..count_positions {
        vars.push(replacer.emited_vars[&i].clone());
    }

    (new_text.into(), vars)
}

struct PuttingArg {
    emited_vars: HashMap<usize, String>,
    index: usize,
}

// TODO: clean this up.
// it's library dependent ?
impl regex::Replacer for &mut PuttingArg {
    fn replace_append(&mut self, caps: &regex::Captures, dst: &mut String) {
        let var = caps.get(1).unwrap().as_str();

        let index = if let Some((pos, _)) = self.emited_vars.iter().find(|(_, v)| v.as_str() == var)
        {
            *pos
        } else {
            let index = self.index;
            self.emited_vars.insert(index, var.to_owned());
            self.index += 1;
            index
        };
        let replacement = format!("arguments[{}]", index);

        dst.push_str(replacement.as_str());
    }
}

struct VarReplacer<'a, P: VarPrinter> {
    data: &'a HashMap<String, Value>,
    printer: std::marker::PhantomData<P>,
}

trait VarPrinter {
    fn print(val: &Value) -> String;
}

struct JSPrinter {}

impl VarPrinter for JSPrinter {
    fn print(val: &Value) -> String {
        val.to_string()
    }
}

struct PlainPrinter {}

impl VarPrinter for PlainPrinter {
    fn print(val: &Value) -> String {
        match val {
            Value::String(val) => val.clone(),
            Value::Null => "".to_string(),
            Value::Number(val) => val.to_string(),
            Value::Object(..) => "[object Object]".to_string(), // is it ok behaviour?
            Value::Array(values) => values
                .into_iter()
                .map(|v| Self::print(v))
                .collect::<Vec<_>>()
                .join(","),
            Value::Bool(val) => val.to_string(),
        }
    }
}

impl<P: VarPrinter> regex::Replacer for VarReplacer<'_, P> {
    fn replace_append(&mut self, caps: &regex::Captures, dst: &mut String) {
        let var = caps.get(1).unwrap().as_str();
        eprintln!("{}", var);
        let replacement = match self.data.get(var) {
            Some(value) => P::print(value),
            None => "".to_string(),
        };

        dst.push_str(replacement.as_str());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_emit_variables() {
        let mut vars = HashMap::new();
        vars.insert("hello".to_string(), json!("Hello"));
        vars.insert("world".to_string(), json!("World"));
        vars.insert("something".to_string(), json!("XXX"));

        assert_eq!(
            "\"Hello\"",
            emit_variables::<_, JSPrinter>("${hello}", &vars)
        );
        assert_eq!(
            "\"Hello\" \"World\"!",
            emit_variables::<_, JSPrinter>("${hello} ${world}!", &vars)
        );
        assert_eq!(
            "There are no vars here",
            emit_variables::<_, JSPrinter>("There are no vars here", &vars)
        );
        assert_eq!(
            "${\"XXX\"}",
            emit_variables::<_, JSPrinter>("${${something}}", &vars)
        );

        assert_eq!(
            "\"\"World\"\"",
            emit_variables::<_, JSPrinter>("\"${world}\"", &vars)
        );
        assert_eq!(
            "\"World\"\" }",
            emit_variables::<_, JSPrinter>("${world}\" }", &vars)
        );
        assert_eq!(
            "\"World\"\"}",
            emit_variables::<_, JSPrinter>("${world}\"}", &vars)
        );
        assert_eq!(
            "\"World\" }",
            emit_variables::<_, JSPrinter>("${world} }", &vars)
        );
        assert_eq!(
            "\"World\"}",
            emit_variables::<_, JSPrinter>("${world}}", &vars)
        );
    }

    #[test]
    fn test_emit_variables_types() {
        let mut vars = HashMap::new();

        vars.insert("test".to_string(), json!("string"));
        assert_eq!(
            r#""string""#,
            emit_variables::<_, JSPrinter>("${test}", &vars)
        );

        vars.insert("test".to_string(), json!(2));
        assert_eq!("2", emit_variables::<_, JSPrinter>("${test}", &vars));

        vars.insert("test".to_string(), json!({"h3": 3}));
        assert_eq!(
            r#"{"h3":3}"#,
            emit_variables::<_, JSPrinter>("${test}", &vars)
        );

        vars.insert("test".to_string(), json!(["h4", 4]));
        assert_eq!(
            r#"["h4",4]"#,
            emit_variables::<_, JSPrinter>("${test}", &vars)
        );
    }
}
