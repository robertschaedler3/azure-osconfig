use anyhow::anyhow;
use colored::Colorize;
use format_serde_error::SerdeError;
use serde::Deserialize;
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    hash::Hash,
    io::Write,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::Result;
use osc::module::{self, schema, Library, Session};

use super::log;

#[derive(Debug, Deserialize)]
struct Definition {
    #[serde(default)]
    client: Client,
    modules: Vec<String>,
    setup: Option<Script>,
    teardown: Option<Script>,
    scenarios: Vec<Scenario>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Script(String);

#[derive(Debug, Deserialize)]
struct Scenario {
    name: String,

    steps: Vec<Step>,
}

#[derive(Clone, Debug, Deserialize)]
struct Client {
    #[serde(rename = "client")]
    name: String,
    max_payload_size: u32,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            name: "osc".to_string(),
            max_payload_size: 0,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Step {
    name: String,
    component: String,
    object: String,

    #[serde(flatten)]
    action: Action,

    #[serde(default)]
    expect: Expect,

    #[serde(flatten)]
    #[serde(default)]
    options: Options,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
#[serde(rename_all = "lowercase")]
enum Action {
    Get,
    Set {
        #[serde(flatten)]
        value: Value,

        size: Option<i32>,
    },
}

#[derive(Debug, Deserialize)]
struct Expect {
    #[serde(flatten)]
    value: Option<Value>,

    size: Option<i32>,

    status: Status,
}

impl Default for Expect {
    fn default() -> Self {
        Self {
            value: None,
            size: None,
            status: Status::Success,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[serde(rename_all = "lowercase")]
enum Status {
    // FIXME: success/failure not working
    Success,
    Failure,
    Exit(i32),
}

impl Status {
    fn check(&self, other: i32) -> Result<()> {
        let valid = match self {
            Status::Success => other == 0,
            Status::Failure => other != 0,
            Status::Exit(code) => other == *code,
        };

        if !valid {
            Err(anyhow!("expected status {:?}, got {}", self, other))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Deserialize)]
enum Value {
    #[serde(rename = "payload")]
    Payload(schema::Value),

    #[serde(rename = "json")]
    // TODO: validate this JSON string (https://github.com/serde-rs/serde/issues/939#issuecomment-939514114)
    Json(Json),
}

#[derive(Debug, Deserialize)]
#[serde(try_from = "String")] // Tell serde to deserialize data into a String and then try to convert it into Email
pub struct Json(schema::Value);

impl TryFrom<String> for Json {
    type Error = String;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        let value = serde_json::from_str(&value)
            .map_err(|err| SerdeError::new(value.to_string(), err))
            .unwrap();
        Ok(Self(value))
    }
}

impl Value {
    fn check(&self, other: &str) -> Result<()> {
        match self {
            Value::Payload(value) => {
                let other = serde_json::from_str::<schema::Value>(other)
                    .map_err(|err| SerdeError::new(other.to_string(), err))?;
                if value != &other {
                    return Err(anyhow!("expected {:?}, got {:?}", value, other));
                }
            }
            Value::Json(json) => {
                let other = serde_json::from_str::<schema::Value>(other)
                    .map_err(|err| SerdeError::new(other.to_string(), err))?;
                let value = &json.0;
                if value != &other {
                    return Err(anyhow!("expected {:?}, got {:?}", value, other));
                }
            }
        }
        Ok(())
    }
}

#[derive(Default, Debug, Deserialize)]
struct Options {
    /// Delay in milliseconds before executing the step.
    delay: Option<u64>,

    /// Skip this test step.
    skip: Option<bool>,
}

pub struct Fixture {
    client: Client,
    modules: Vec<String>,
    suites: Vec<Suite>,
}

struct Suite {
    name: String,
    tests: Vec<Test>,
}

struct Test {
    name: String,

    /// A closure that runs the test. This function is passed the client session when it is invoked.
    ///
    /// _This function may not be invoked if a session cannot be opened or found for the corresponding module/component._
    body: Box<
        dyn Fn(
            &Context,
            String,
            &HashMap<Module, Arc<Library>>,
            &HashMap<Component, Module>,
            &HashMap<Module, Session>,
        ) -> Result<TestResult>,
    >,
}

#[derive(Debug)]
enum TestResult {
    Success {
        duration: Duration,
    },
    Failure {
        duration: Duration,
        failure: Failure,
    },
    Skipped,
}

#[derive(Debug, Clone)]
struct Failure {
    name: String,
    error: String,
    log: String,
}

// TODO: use this stuct for binding values between steps, variables, etc
struct Context {
    client: Client,
}

type Module = String;
type Component = String;

impl Fixture {
    pub fn from_file(path: &PathBuf) -> Result<(Option<Script>, Option<Script>, Self)> {
        let s = std::fs::read_to_string(path)?;
        let definition: Definition =
            serde_yaml::from_str(s.as_str()).map_err(|err| SerdeError::new(s.to_string(), err))?;

        // TODO: load the modules (and register components) here and store them in the fixture

        // TODO: ensure unique names for suites and tests

        Ok(definition.into())
    }

    pub fn run(&self, bin: PathBuf) -> Result<()> {
        let mut log = String::new();

        // TODO: example test output
        //
        // test definition: <path>
        //
        // modules:
        //   - <name> (<version>)
        //   - <name> (<version>)
        //
        // running <n> tests from <m> suites
        //
        //   suite <name>:
        //     test <name> ... <passed | failed | skipped>  <duration>ms
        //     test <name> ... <passed | failed | skipped>  <duration>ms
        //     ...
        //
        // failures:
        // ------- <suite>::<test> -------
        //  | <log>
        //  | <log>
        //  | <log>
        //
        // <reason>
        //
        // ...
        //
        //
        // summary: <PASSED|FAILED>
        //     <n> passed
        //     <n> failed
        //     <n> skipped
        //     <n> total <n>ms
        //

        let modules = self
            .modules
            .clone()
            .into_iter()
            .map(|module| {
                let path = bin.join(&module).with_extension("so");
                let lib = log::capture(&mut log, || module::Library::load(path))?;
                Ok((module, Arc::new(lib)))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        // REVIEW: what happens if a component name is duplicated across modules?
        let components = modules
            .iter()
            .map(|(module, lib)| {
                // TODO: a module may fail during get_info(), so errors should be handled here
                let info = log::capture(&mut log, || lib.info(&self.client.name))?;
                Ok(info
                    .components
                    .into_iter()
                    .map(move |component| (component, module.clone())))
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect::<HashMap<_, _>>();

        println!("");
        println!("modules:");
        for (module, _) in &modules {
            // println!("  - {} ({})", module, lib.info(&self.client.name)?.version);
            println!("  - {} (1.0.0)", module);
        }

        println!("");
        // println!("running {} tests from {} suites", self.tests(), self.suites());

        // TODO: print "loading" logs if verbose logging is enabled (indented and colorized)

        let mut ctx = Context {
            client: self.client.clone(),
        };

        // TODO: print "running x tests"

        // REVIEW: it might be cleaner to implment an iterator for suites and tests ?
        // let failures = self
        //     .suites
        //     .iter()
        //     .map(|suite|

        let total = self
            .suites
            .iter()
            .map(|suite| suite.tests.len())
            .sum::<usize>();
        let mut failures = HashMap::new();
        let time = Instant::now();

        println!("running {} tests:", total);
        println!("");

        for suite in &self.suites {
            let result = suite.run(&mut ctx, &modules, &components)?;
            failures.insert(suite.name.clone(), result);
        }

        let failed = failures.values().map(|failures| failures.len()).sum::<usize>();

        // TODO: use miette errors for this
        // println!("\nfailures:");
        for (_, failures) in failures {
            for failure in failures {
                println!("------- {} -------", failure.name);
                if failure.log.len() > 0 {
                    println!("{}", failure.log);
                }
                println!("\t{}\n", failure.error);
            }
        }

        println!("\nsummary:");
        println!("    {} passed", total - failed);
        println!("    {} failed", failed);
        // TODO: print skipped tests
        println!("    {} total {}ms\n", total, time.elapsed().as_millis());

        Ok(())
    }
}

impl Suite {
    fn run(
        &self,
        ctx: &mut Context,
        modules: &HashMap<Module, Arc<Library>>,
        components: &HashMap<Component, Module>,
    ) -> Result<Vec<Failure>> {
        let sessions = modules
            .iter()
            .map(|(module, lib)| {
                let session = lib.open(&ctx.client.name, ctx.client.max_payload_size)?;
                Ok((module.clone(), session))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        let mut failures = Vec::new();

        for test in &self.tests {
            let result = test.invoke(&ctx, modules, components, &sessions)?;
            if let TestResult::Failure { failure, .. } = &result {
                failures.push(failure.clone());
            }
        }

        // TODO: close all sessions (capture logs)

        // TODO: return suite report and failures
        Ok(failures)
    }
}

impl Test {
    pub fn invoke(
        &self,
        ctx: &Context,
        modules: &HashMap<Component, Arc<Library>>,
        components: &HashMap<Component, Module>,
        sessions: &HashMap<Module, Session>,
    ) -> Result<TestResult> {
        print!("  {:<30}", self.id());

        let result = (self.body)(ctx, self.id(), modules, components, sessions);

        match &result {
            Ok(test_result) => println!(" {}", test_result),
            Err(err) => println!("{}", err),
        }

        result
    }

    /// Convert self.name to all lowercase, remove all non-alphanumeric characters, and replace spaces with dashes.
    fn id(&self) -> String {
        self.name
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .map(|c| if c.is_whitespace() { '-' } else { c })
            .collect()
    }
}

impl Script {
    pub fn execute(&self) -> Result<()> {
        let script = &self.0;

        // Write the script to a temporary file
        let mut file = tempfile::NamedTempFile::new()?;
        file.write_all(script.as_bytes())?;

        let output = std::process::Command::new("bash")
            .arg(file.path())
            .output()?;

        if !output.status.success() {
            // TODO: return a better error that will bring nicely
            return Err(anyhow!(
                "script failed ({}): {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ));
        } else {
            // TODO: only print as a log trace
            // println!("{}", String::from_utf8_lossy(&output.stdout));
        }

        Ok(())
    }
}

impl From<Definition> for (Option<Script>, Option<Script>, Fixture) {
    fn from(definition: Definition) -> Self {
        let Definition {
            client,
            modules,
            setup,
            teardown,
            scenarios,
        } = definition;

        let suites = scenarios
            .into_iter()
            .map(|scenario| scenario.into())
            .collect();

        let fixture = Fixture {
            client,
            modules,
            suites,
        };

        (setup, teardown, fixture)
    }
}

impl From<Scenario> for Suite {
    fn from(scenario: Scenario) -> Self {
        let Scenario { name, steps } = scenario;

        Self {
            name,
            tests: steps.into_iter().map(|step| step.into()).collect(),
        }
    }
}

impl From<Step> for Test {
    fn from(step: Step) -> Self {
        let Step {
            name,
            component,
            object,
            action,
            expect,
            options,
        } = step;

        let body = Box::new(
            move |_ctx: &Context,
                  name: String,
                  modules: &HashMap<Component, Arc<Library>>,
                  components: &HashMap<Component, Module>,
                  sessions: &HashMap<Module, Session>| {
                if let Some(true) = options.skip {
                    return Ok(TestResult::skipped());
                }

                let time = Instant::now();
                let component = component.as_str();
                let object = object.as_str();
                let mut log = String::new();

                if let Some(delay) = options.delay {
                    std::thread::sleep(std::time::Duration::from_millis(delay));
                }

                let lookup: Result<_> = {
                    let module = components
                        .get(component)
                        .ok_or_else(|| anyhow!("Component not found: {}", component))?;
                    let session = sessions
                        .get(module)
                        .ok_or_else(|| anyhow!("Session not found: {}", module))?;
                    let lib = modules
                        .get(module)
                        .ok_or_else(|| anyhow!("Module not found: {}", module))?;

                    Ok((lib, session))
                };

                if let Err(err) = lookup {
                    return Ok(TestResult::failure(
                        time.elapsed(),
                        name,
                        err.to_string(),
                        None,
                    ));
                }

                let (lib, session) = lookup.unwrap();

                let result = match action {
                    Action::Get => {
                        let (status, payload) =
                            log::capture(&mut log, || lib.get(session, component, object))?;

                        // TODO: make these real "expect" assertions (ie allow multiple assertions for a single test)
                        expect.status.check(status).and_then(|()| {
                            // TODO: check size (is specified)
                            if let Some(ref value) = expect.value {
                                value.check(&payload)
                            } else {
                                Ok(())
                            }
                        })
                    }
                    Action::Set { ref value, size } => {
                        let (payload, payload_size) = value.into();

                        let status = log::capture(&mut log, || {
                            lib.set(
                                session,
                                component,
                                object,
                                &payload,
                                size.unwrap_or(payload_size),
                            )
                        })?;

                        expect.status.check(status)
                    }
                };

                let duration = time.elapsed();

                match result {
                    Ok(_) => Ok(TestResult::success(duration)),
                    Err(err) => Ok(TestResult::failure(
                        duration,
                        name,
                        err.to_string(),
                        Some(log),
                    )),
                }
            },
        );

        Self { name, body }
    }
}

impl From<&Value> for (String, i32) {
    fn from(value: &Value) -> Self {
        match value {
            Value::Payload(payload) => {
                let payload = serde_json::to_string(&payload).unwrap();
                let size = payload.len();
                (payload, size as i32)
            }
            Value::Json(json) => {
                let payload = serde_json::to_string(&json.0).unwrap();
                let size = payload.len();
                (payload, size as i32)
            }
        }
    }
}

impl TestResult {
    pub fn success(duration: Duration) -> Self {
        Self::Success { duration }
    }

    pub fn skipped() -> Self {
        Self::Skipped
    }

    pub fn failure(duration: Duration, name: String, error: String, log: Option<String>) -> Self {
        Self::Failure {
            duration,
            failure: Failure {
                name,
                error,
                log: log.unwrap_or_default(),
            },
        }
    }
}

impl Display for TestResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            TestResult::Success { duration } => {
                write!(
                    f,
                    "{}   {}ms",
                    "passed".bright_green(),
                    duration.as_millis()
                )
            }
            TestResult::Skipped => write!(f, "{}", "skipped".bright_yellow()),
            TestResult::Failure { duration, .. } => {
                write!(
                    f,
                    "{}   {}ms",
                    "failed".bright_red().bold(),
                    duration.as_millis()
                )
            }
        }
    }
}
