use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Result};
use colored::Colorize;

use osc::module::{self, Library, Session};

use super::{
    definition::{Action, Assertion, Config, Definition, Step},
    log,
};

pub struct Fixture {
    config: Config,
    modules: Vec<String>,
    tests: Vec<Test>,
}

pub struct Test {
    delay: Duration,
    skip: bool,

    /// A closure that runs the test. This function is passed the client session when it is invoked.
    ///
    /// _This function may not be invoked if a session cannot be opened or found for the corresponding module/component._
    body: Body,
    // result: Option<Outcome>,
}

pub struct Body(Box<dyn Fn(&Context) -> Result<()>>);

// "Helper" type to improve readability
type Module = String;
type Component = String;

pub struct Context {
    modules: HashMap<Module, Arc<Library>>,
    components: HashMap<Component, Module>,
    sessions: HashMap<Module, Session>,
}

#[derive(Debug)]
enum Outcome {
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
    error: String,
    log: String,
}

impl Fixture {
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

        let Config {
            client_name,
            max_payload_size,
        } = self.config.clone();

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

        println!("");
        println!("modules:");

        // REVIEW: what happens if a component name is duplicated across modules?
        let components = modules
            .iter()
            .map(|(module, lib)| {
                let info = log::capture(&mut log, || lib.info(&client_name))?;
                println!("  - {} ({})", info.name, info.version);
                Ok(info
                    .components
                    .into_iter()
                    .map(move |component| (component, module.clone())))
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect::<HashMap<_, _>>();

        let sessions = modules
            .iter()
            .map(|(module, lib)| {
                let session = log::capture(&mut log, || lib.open(&client_name, max_payload_size))?;
                Ok((module.clone(), session))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        // TODO: print "loading" logs if verbose logging is enabled (indented and colorized)

        let ctx = Context {
            modules,
            components,
            sessions,
        };

        let total = self.tests.len();
        let mut skipped = 0;
        let mut failures = HashMap::new();

        println!("");
        println!("running {} tests:", total);
        println!("");

        for (i, test) in self.tests.iter().enumerate() {
            let outcome = test.run(&ctx);
            println!("  test {} ... {}", i, outcome);

            if let Outcome::Skipped = &outcome {
                skipped += 1;
            }

            if let Outcome::Failure { failure, .. } = outcome {
                failures.insert(i, failure);
            }
        }

        println!("");

        for (i, failure) in &failures {
            println!("------- test {} -------", i);
            println!("{}", failure.log);
            println!("{}", failure.error);
            println!("");
        }

        let failed = failures.len();
        let passed = total - failed - skipped;

        println!("");
        println!(
            "summary: {}",
            if failed > 0 {
                "FAILED".red()
            } else {
                "PASSED".green()
            }
        );
        println!("    {} passed", passed);
        println!("    {} failed", failed);
        println!("    {} skipped", skipped);
        println!("    {} total", total);

        Ok(())
    }
}

impl Test {
    fn run(&self, ctx: &Context) -> Outcome {
        let time = Instant::now();

        if self.skip {
            return Outcome::skipped();
        }

        if self.delay > Duration::from_secs(0) {
            std::thread::sleep(self.delay);
        }

        let (log, result) = (self.body).invoke(ctx);
        let duration = time.elapsed();

        match result {
            Ok(()) => Outcome::passed(duration),
            Err(err) => Outcome::failed(duration, err.to_string(), Some(log)),
        }
    }
}

impl Context {
    fn module(&self, component: &str) -> Result<(Arc<Library>, Session)> {
        let module = self
            .components
            .get(component)
            .ok_or_else(|| anyhow!("Component not found: {}", component))?;
        let session = self
            .sessions
            .get(module)
            .ok_or_else(|| anyhow!("Session not found: {}", module))?;
        let lib = self
            .modules
            .get(module)
            .ok_or_else(|| anyhow!("Module not found: {}", module))?;

        Ok((lib.clone(), session.clone()))
    }

    // TODO: function for capturing logs and storing them in the context for later printing
}

impl Body {
    pub fn new(action: Action, assertion: Assertion) -> Self {
        let body = Box::new(move |ctx: &Context| {
            match action {
                Action::Get {
                    ref component,
                    ref object,
                } => {
                    let (lib, session) = ctx.module(&component)?;

                    let (status, payload) = lib.get(&session, &component, &object)?;

                    // TODO: run the assertions here
                }
                Action::Set {
                    ref component,
                    ref object,
                    ref value,
                    size,
                } => {
                    let (lib, session) = ctx.module(&component)?;

                    let (payload, payload_size) = value.into();
                    let payload_size = size.unwrap_or(payload_size);

                    let status = lib.set(
                        &session,
                        &component,
                        &object,
                        &payload,
                        payload_size as usize,
                    )?;

                    // TODO: run the assertions here
                }
            };

            // if let Err(err) = result {
            //     return Outcome::failed(duration, err, Some(log));
            // }

            Ok(())
        });

        Self(body)
    }

    /// Invoke the test body.
    /// TODO: fix the return type here (this is messy)
    fn invoke(&self, ctx: &Context) -> (String, Result<()>) {
        let mut buffer = String::new();
        let result = log::capture(&mut buffer, || (self.0)(ctx));
        (buffer, result)
    }
}

impl Outcome {
    pub fn passed(duration: Duration) -> Self {
        Self::Success { duration }
    }

    pub fn skipped() -> Self {
        Self::Skipped
    }

    pub fn failed(duration: Duration, error: String, log: Option<String>) -> Self {
        Self::Failure {
            duration,
            failure: Failure {
                error,
                log: log.unwrap_or_default(),
            },
        }
    }
}

impl From<&Definition> for Fixture {
    fn from(definition: &Definition) -> Self {
        let Definition {
            config,
            modules,
            steps,
            ..
        } = definition;

        let config = config.clone();
        let modules = modules.clone();
        let tests = steps.iter().map(|step| step.into()).collect();

        Self {
            config,
            modules,
            tests,
        }
    }
}

impl From<&Step> for Test {
    fn from(step: &Step) -> Self {
        let Step {
            action,
            assert,
            options,
        } = step;

        let body = Body::new(action.clone(), assert.clone());
        let delay = options.delay.unwrap_or(0);
        let delay = std::time::Duration::from_millis(delay);
        let skip = options.skip.unwrap_or(false);

        Self {
            body,
            delay,
            skip,
            // result: None,
        }
    }
}

impl Display for Outcome {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Outcome::Success { duration } => {
                write!(
                    f,
                    "{}    {}ms",
                    "passed".bright_green(),
                    duration.as_millis()
                )
            }
            Outcome::Skipped => write!(f, "{}", "skipped".bright_yellow()),
            Outcome::Failure { duration, .. } => {
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
