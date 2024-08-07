// Copyright (c) Microsoft Corporation. All rights reserved..
// Licensed under the MIT License.

use std::{
    ffi::{CStr, CString},
    slice,
};

use ctor::{ctor, dtor};
use libc::{c_char, c_int, c_uint, c_void, EINVAL};
use osc::{
    module::{bindings::MODULE_OK, ModuleInfo},
    serde::{Deserialize, Serialize},
    serde_json,
};

type MmiHandle = *mut c_void;
type MmiJsonString = *mut c_char;

#[ctor]
fn init() {
    osc::init_logger();
    log::info!("Blah module loaded");
}

#[dtor]
fn deinit() {
    log::info!("Blah module unloaded");
}

enum ModuleError {
    InvalidArgument,
    IoError,
    SerdeError,
}

impl From<i32> for ModuleError {
    fn from(code: i32) -> Self {
        match code {
            EINVAL => ModuleError::InvalidArgument,
            _ => ModuleError::IoError,
        }
    }
}

#[no_mangle]
pub extern "C" fn MmiGetInfo(
    client_name: *const c_char,
    payload: *mut MmiJsonString,
    payload_size_bytes: *mut c_int,
) -> c_int {
    let status = if client_name.is_null() {
        log::error!("MmiGetInfo: client_name is null");
        EINVAL
    } else if payload.is_null() {
        log::error!("MmiGetInfo: payload is null");
        EINVAL
    } else if payload_size_bytes.is_null() {
        log::error!("MmiGetInfo: payload_size_bytes is null");
        EINVAL
    } else {
        let client_name: &CStr = unsafe { CStr::from_ptr(client_name) };
        let client_name: &str = client_name.to_str().unwrap_or_default(); // TODO: handle error
        match get_info(client_name) {
            Ok(module_info) => {
                let json = serde_json::to_string(&module_info).unwrap();
                let json = CString::new(json).unwrap();
                let size = json.as_bytes().len() as c_int;
                let json = json.into_raw();
                unsafe {
                    *payload_size_bytes = size;
                    *payload = json;
                }
                MODULE_OK
            }
            Err(error) => {
                log::error!("MmiGetInfo: TODO: ERROR");
                EINVAL
            }
        }
    };

    // log::trace!("MmiGetInfo({}, {}, {}) returning {}", client_name, payload_size_bytes, payload, status);

    status
}

// TODO: macro for converting this into the above
// #[get_info]
fn get_info(_client_name: &str) -> Result<ModuleInfo, ModuleError> {
    let module_info: ModuleInfo = ModuleInfo {
        name: "Blah".to_string(),
        description: "Blah module".to_string(),
        manufacturer: "Microsoft".to_string(),
        components: vec!["Blah".to_string()],
    };

    Ok(module_info)
}

#[derive(Default)]
struct Context {
    message: String,
}

#[no_mangle]
pub extern "C" fn MmiOpen(client_name: *const c_char, max_payload_size_bytes: c_uint) -> MmiHandle {
    if client_name.is_null() {
        log::error!("MmiOpen: client_name is null");
        std::ptr::null_mut()
    } else {
        let context = Box::new(Context::default());
        let context = Box::into_raw(context);
        context as MmiHandle
    }
}

#[no_mangle]
pub extern "C" fn MmiClose(client_session: MmiHandle) {
    if client_session.is_null() {
        log::error!("MmiClose: client_session is null");
    } else {
        let context: Box<Context> = unsafe { Box::from_raw(client_session as *mut Context) };
        drop(context);
    }
}

#[no_mangle]
pub extern "C" fn MmiSet(
    client_session: MmiHandle,
    component_name: *const c_char,
    object_name: *const c_char,
    payload: MmiJsonString,
    payload_size_bytes: c_int,
) -> c_int {
    if client_session.is_null() {
        log::error!("MmiSet: client_session is null");
        EINVAL
    } else if component_name.is_null() {
        log::error!("MmiSet: component_name is null");
        EINVAL
    } else if object_name.is_null() {
        log::error!("MmiSet: object_name is null");
        EINVAL
    } else if payload.is_null() {
        log::error!("MmiSet: payload is null");
        EINVAL
    } else if payload_size_bytes < 0 {
        log::error!("MmiSet: payload_size_bytes is negative");
        EINVAL
    } else {
        unimplemented!()
        // let client_session: &mut Context = unsafe { &mut *(client_session as *mut Context) };
        // let component_name: &CStr = unsafe { CStr::from_ptr(component_name) };
        // let component_name: &str = component_name.to_str().unwrap_or_default(); // TODO: handle error
        // let object_name: &CStr = unsafe { CStr::from_ptr(object_name) };
        // let object_name: &str = object_name.to_str().unwrap_or_default(); // TODO: handle error
        // match set(client_session, component_name, object_name, payload, payload_size_bytes) {
        //     Ok(()) => MODULE_OK,
        //     Err(error) => {
        //         log::error!("MmiSet: TODO: ERROR");
        //         EINVAL
        //     }
        // }
    }
}

// TODO: macro for converting this into the above
// #[set]
// #[component("Blah")]
// #[object("foo", set_foo)]

fn set(
    context: &mut Context,
    component_name: &str,
    object_name: &str,
    payload: MmiJsonString,
    payload_size_bytes: c_int,
) -> Result<(), ModuleError> {
    let payload =
        unsafe { slice::from_raw_parts(payload as *const u8, payload_size_bytes as usize) };
    let payload = String::from_utf8(payload.to_vec()).unwrap();
    let payload: serde_json::Value = serde_json::from_str(&payload).unwrap();

    match component_name {
        "Blah" => match object_name {
            "foo" => {
                let foo: Foo = serde_json::from_value(payload).unwrap();
                set_foo(foo)
            }
            "bar" => {
                let bar: Bar = serde_json::from_value(payload).unwrap();
                set_bar(context, bar)
            }
            _ => {
                log::error!("MmiSet: invalid object_name {}", object_name);
                Err(ModuleError::InvalidArgument)
            }
        },
        _ => {
            log::error!("MmiSet: invalid component_name {}", component_name);
            Err(ModuleError::InvalidArgument)
        }
    }
}

// TODO: there should be some way with generics to make this work for any type
// and not need complicated/nested match statements

#[derive(Serialize, Deserialize)]
struct Foo {
    message: String,
}

#[derive(Serialize, Deserialize)]
struct Bar {
    message: String,
}

// #[set("foo")]
fn set_foo(foo: Foo) -> Result<(), ModuleError> {
    log::info!("setting foo: {}", foo.message);

    Ok(())
}

// #[set("bar", context = Context)]
fn set_bar(context: &mut Context, bar: Bar) -> Result<(), ModuleError> {
    log::info!("context: {}", context.message);
    log::info!("setting bar: {}", bar.message);

    context.message = bar.message;

    Ok(())
}

#[no_mangle]
pub extern "C" fn MmiGet(
    client_session: MmiHandle,
    component_name: *const c_char,
    object_name: *const c_char,
    payload: *mut MmiJsonString,
    payload_size_bytes: *mut c_int,
) -> c_int {
    log::info!("YAY");

    0
}

// #[no_mangle]
// pub extern "C" fn MmiFree(payload: MmiJsonString) {

// }
