use osc::osc_component;
use serde::Serialize;

#[derive(Debug, Default)]
struct Hostname;

// TODO: #[osc_component(module = "MyModule")]
#[osc_component]
impl Hostname {
    #[osc_object(reported)]
    fn name(&self) -> String {
        hostname::get().unwrap().to_string_lossy().to_string()
    }

    #[osc_object(reported)]
    fn complex(&self) -> Complex {
        Complex { a: 1, b: 2, c: 3 }
    }

    #[osc_object(desired, name = "name")]
    fn desired_name(&mut self, name: &str) {
        // hostname::set(name).unwrap();
        println!("setting hostname: {}", name);
    }
}

#[derive(Serialize)]
struct Complex {
    a: u32,
    b: u32,
    c: u32,
}

// --------------------------------------------------------------------------------

// struct Blah {
//     client: String,
//     max_payload_size: u32,
// }

// impl osc::module::Module for Blah {
//     fn new(client_name: &str, max_payload_size: u32) -> Self {
//         Self {
//             client: client_name.to_string(),
//             max_payload_size,
//         }
//     }

//     fn get(&self, component: &str, object: &str) -> String {
//         println!("get: {} {}", component, object);
//         String::new()
//     }

//     fn set(&mut self, component: &str, object: &str, value: &str) {
//         println!("set: {} {} {}", component, object, value);
//     }
// }

type Blah = MyModule;

// --------------------------------------------------------------------------------

use libc::{c_char, c_int, c_uint, EINVAL};

use osc::module::interface::{close, get, open, set, Handle, JsonString};

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

use std::ffi::{CStr, CString};
use std::ptr;

fn main() {
    let component = CString::new("Hostname").unwrap();
    let object = CString::new("complex").unwrap();
    let blah = CString::new("blah").unwrap();

    let handle = MmiOpen(blah.as_ptr() as *const c_char, 1024);

    let mut payload: JsonString = ptr::null_mut();
    let mut payload_size_bytes: c_int = 0;
    MmiGet(
        handle,
        component.as_ptr(),
        object.as_ptr(),
        &mut payload,
        &mut payload_size_bytes,
    );

    let payload = unsafe { CStr::from_ptr(payload) };
    let payload = payload.to_str().unwrap();
    println!("payload: {}", payload);

    // MmiSet(handle, component.as_ptr(), object.as_ptr(), blah.as_ptr() as MmiJsonString, payload_size_bytes);

    MmiClose(handle);

    // let mut m = Module::new();
    // println!("{:?}", m.get("Hostname", "name"));
    // m.set("Hostname", "name", "foo");
    // println!("{:?}", m.get("Hostname", "name"));
}
