use osc::osc_module;
use serde::{Deserialize, Serialize, Serializer, Deserializer};

mod command;

use command::{Command, Runner, Status};

struct CommandRunner {
    refresh_id: Option<String>,
    runner: Runner,
}

impl Default for CommandRunner {
    fn default() -> Self {
        Self {
            refresh_id: None,
            runner: Runner::new(),
        }
    }
}

#[osc_module]
impl CommandRunner {
    #[reported(name = "commandStatus")]
    fn status(&self) -> CommandStatus {
        if let Some(refresh_id) = &self.refresh_id {
            if let Some(status) = self.runner.status(&refresh_id) {
                return CommandStatus::from_status(refresh_id.clone(), status);
            }
        }
        CommandStatus::default()
    }

    #[desired(name = "commandArguments")]
    fn command(&mut self, args: CommandArguments) {
        match args.action {
            Action::Run => {
                self.refresh_id = Some(args.id.clone());
                self.runner.run(args.into());
            }
            Action::Cancel => {
                self.runner.cancel(&args.id);
            }
            Action::Refresh => self.refresh_id = Some(args.id),
            _ => println!("Unknown command action: {:?}", args.action),
        }
    }
}

#[derive(Default, Serialize)]
struct CommandStatus {
    #[serde(rename = "commandId")]
    id: String,

    #[serde(rename = "resultCode")]
    exit_code: i32,

    #[serde(rename = "textResult")]
    output: String,

    #[serde(rename = "currentState")]
    state: State,
}

#[derive(Clone, Copy, Debug, Default)]
enum State {
    #[default]
    Unknown= 0,
    Running = 1,
    Succeeded = 2,
    Failed = 3,
    TimedOut = 4,
    Canceled = 5
}

impl Serialize for State {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(*self as i32)
    }
}

// TODO: Remove Serialize
#[derive(Debug, Serialize, Deserialize)]
struct CommandArguments {
    #[serde(rename = "commandId")]
    id: String,

    arguments: String,
    timeout: Option<u32>,

    #[serde(rename = "singleLineTextResult")]
    single_line: bool,

    action: Action,
}

#[derive(Copy, Clone, Debug)]
enum Action {
    None = 0,
    Reboot = 1,
    Shutdown = 2,
    Run = 3,
    Refresh = 4,
    Cancel = 5,
}

// TODO: Remove Serialize
impl Serialize for Action {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(*self as i32)
    }
}

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let i = i32::deserialize(deserializer)?;
        match i {
            0 => Ok(Action::None),
            1 => Ok(Action::Reboot),
            2 => Ok(Action::Shutdown),
            3 => Ok(Action::Run),
            4 => Ok(Action::Refresh),
            5 => Ok(Action::Cancel),
            _ => Err(serde::de::Error::custom("invalid action")),
        }
    }
}

impl From<CommandArguments> for Command {
    fn from(args: CommandArguments) -> Self {
        Command {
            id: args.id,
            arguments: args.arguments,
            timeout: args.timeout,
            single_line: args.single_line,
        }
    }
}

impl CommandStatus {
    fn from_status(id: String, status: Status) -> Self {
        match status {
            Status::Running => Self {
                id,
                state: State::Running,
                ..Default::default()
            },
            Status::Complete(exit_code, output) => Self {
                id,
                exit_code,
                output,
                state: if exit_code == 0 {
                    State::Succeeded
                } else {
                    State::Failed
                }
            },
            Status::TimedOut => Self {
                id,
                state: State::TimedOut,
                ..Default::default()
            },
            Status::Canceled => Self {
                id,
                state: State::Canceled,
                ..Default::default()
            },
        }
    }
}

// --------------------------------------------------------------------------------

use libc::{c_char, c_int};
use serde_json::Value;
use std::ffi::{CStr, CString};
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::{env, ptr};

use osc::module::interface::{Handle, JsonString};

fn get_args() -> Option<(String, String)> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        return None;
    }

    let component = &args[1];
    let object = &args[2];

    Some((component.to_string(), object.to_string()))
}

fn call_get(handle: Handle, component: &str, object: &str) {
    println!("Get({:?}, {}, {})", handle, component, object);

    let component = CString::new(component).unwrap();
    let object = CString::new(object).unwrap();

    let mut payload: JsonString = ptr::null_mut();
    let mut payload_size_bytes: c_int = 0;

    let result = MmiGet(
        handle,
        component.as_ptr(),
        object.as_ptr(),
        &mut payload,
        &mut payload_size_bytes,
    );

    if result == 0 {
        let payload = unsafe { CStr::from_ptr(payload) };
        let payload = payload.to_str().unwrap();
        let value: Value = serde_json::from_str(&payload).unwrap();
        // let complex = serde_json::from_value::<Complex>(value).unwrap();
        let value = serde_json::to_string_pretty(&value).unwrap();
        println!("{}", value);
    } else {
        println!("{}", result);
    }
}

fn call_set<T>(handle: Handle, component: &str, object: &str, value: T)
where
    T: serde::Serialize + std::fmt::Debug,
{
    println!("Set({:?}, {}, {}, {:?})", handle, component, object, value);

    let component = CString::new(component).unwrap();
    let object = CString::new(object).unwrap();

    let value = serde_json::to_string(&value).unwrap();
    let value = CString::new(value).unwrap();

    let result = MmiSet(
        handle,
        component.as_ptr(),
        object.as_ptr(),
        value.as_ptr() as JsonString,
        value.as_bytes().len() as c_int,
    );

    if result != 0 {
        println!("{}", result);
    }
}

// --------------------------------------------------------------------------------

fn spawn(name: String, canceled: Arc<AtomicBool>) -> std::thread::JoinHandle<()> {
    // spawn a thread and print the name of the thread every second
    std::thread::spawn(move || {
        let mut i = 0;
        loop {
            let child = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!("sleep {}; echo [{}] {}", i, name, i))
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();

            let secs =

            // the child process can be canceled, timeout, or complete
            let status = child.wait_with_output().unwrap();


            i += 1;
            std::thread::sleep(std::time::Duration::from_secs(1))
        }
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let (component, object) = get_args().ok_or_else(|| "Usage: <component> <object>")?;

    // let component = "CommandRunner";
    // let desired = "commandArguments";
    // let reported = "commandStatus";

    // let blah = CString::new("blah").unwrap();
    // let handle = MmiOpen(blah.as_ptr() as *const c_char, 1024);

    // let value = CommandArguments {
    //     id: "123".to_string(),
    //     arguments: "sleep 10s".to_string(),
    //     timeout: Some(1),
    //     single_line: true,
    //     action: Action::Run,
    // };

    // call_set(handle, component, desired, value);

    // std::thread::sleep(std::time::Duration::from_secs(1));

    // call_get(handle, component, reported);

    // MmiClose(handle);


    let t1 = spawn("thread".to_string());

    // After 5 seconds send a


    t1.join().unwrap();


    Ok(())
}
