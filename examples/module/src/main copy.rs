use osc::osc_module;
use serde::{Deserialize, Serialize};

// REVIEW: try to do this without Default
// #[derive(Debug, Default)]
// struct Hostname;

// TODO: #[osc_component(module = "MyModule")]
// #[osc_component]
// impl Hostname {
//     #[osc_object(reported)]
//     fn name(&self) -> String {
//         hostname::get().unwrap().to_string_lossy().to_string()
//     }

//     #[osc_object(reported, name = "hosts")]
//     fn hosts(&self) -> Vec<String> {
//         let command = std::process::Command::new("hostname")
//             .arg("-I")
//             .output()
//             .unwrap();
//         let output = String::from_utf8_lossy(&command.stdout);
//         output.split_whitespace().map(|s| s.to_string()).collect()
//     }

//     #[osc_object(desired, name = "name")]
//     fn desired_name(&mut self, name: &str) {
//         // hostname::set(name).unwrap();
//         println!("setting hostname: {}", name);
//     }
// }

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Complex {
    string: String,
    num: u32,
    boolean: bool,
    array: Vec<String>,
    str_enum: StrEnum,
    num_enum: NumEnum,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum StrEnum {
    #[default]
    EnumValue,
}

#[derive(Debug, Default, Copy, Clone)]
enum NumEnum {
    #[default]
    EnumValue = 1,
}

impl Serialize for NumEnum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(*self as u32)
    }
}

impl<'de> Deserialize<'de> for NumEnum {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let num = u32::deserialize(deserializer)?;
        match num {
            1 => Ok(NumEnum::EnumValue),
            _ => Err(serde::de::Error::custom("invalid enum value")),
        }
    }
}

#[derive(Default)]
struct Something {
    complex: Complex,
}

#[osc_module]
impl Something {
    #[osc_object(reported)]
    fn something(&self) -> &str {
        "something"
    }

    #[osc_object(reported)]
    fn complex(&self) -> Complex {
        self.complex.clone()
    }

    #[osc_object(desired, name = "complex")]
    fn desired_complex(&mut self, complex: Complex) {
        self.complex = complex;
    }
}

// --------------------------------------------------------------------------------

use libc::{c_char, c_int, c_uint, EINVAL};

use osc::module::interface::{close, get, open, set, Handle, JsonString};

type Blah = SomethingModule;

// TODO: the "Mmi" interface should be generated by the osc_codegen crate

#[no_mangle]
pub extern "C" fn MmiOpen(client_name: *const c_char, max_payload_size: c_uint) -> Handle {
    if let Ok(module) = open::<Blah>(client_name, max_payload_size) {
        Box::into_raw(Box::new(module)) as Handle
    } else {
        // TODO: log error
        println!("MmiOpen failed");
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn MmiClose(client_session: Handle) {
    close::<Blah>(client_session);
}

#[no_mangle]
pub extern "C" fn MmiSet(
    client_session: Handle,
    component_name: *const c_char,
    object_name: *const c_char,
    payload: JsonString,
    payload_size_bytes: c_int,
) -> c_int {
    if let Err(err) = set::<Blah>(
        client_session,
        component_name,
        object_name,
        payload,
        payload_size_bytes,
    ) {
        // TODO: log error
        println!("error: {}", err);

        // TODO: convert error to appropriate error code
        EINVAL
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn MmiGet(
    client_session: Handle,
    component_name: *const c_char,
    object_name: *const c_char,
    payload: *mut JsonString,
    payload_size_bytes: *mut c_int,
) -> c_int {
    if let Err(err) = get::<Blah>(
        client_session,
        component_name,
        object_name,
        payload,
        payload_size_bytes,
    ) {
        // TODO: log error
        println!("error: {}", err);

        // TODO: convert error to appropriate error code
        EINVAL
    } else {
        0
    }
}

// --------------------------------------------------------------------------------

use serde_json::Value;
use std::ffi::{CStr, CString};
use std::{env, ptr};

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (component, object) = get_args().ok_or_else(|| "Usage: <component> <object>")?;

    let blah = CString::new("blah").unwrap();
    let handle = MmiOpen(blah.as_ptr() as *const c_char, 1024);

    let complex = Complex {
        string: "hello".to_string(),
        num: 42,
        boolean: true,
        array: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        num_enum: NumEnum::EnumValue,
        str_enum: StrEnum::EnumValue,
    };

    call_set(handle, &component, &object, complex);
    call_get(handle, &component, &object);

    MmiClose(handle);

    Ok(())
}