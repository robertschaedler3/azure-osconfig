// Copyright (c) Microsoft Corporation. All rights reserved..
// Licensed under the MIT License.

use ctor::{ctor, dtor};
use libc::{c_char, c_int, c_uint, c_void, EINVAL, ENOMEM};
use std::ffi::CString;

const MMI_OK: i32 = 0;
type MmiHandle = *mut c_void;
type MmiJsonString = *mut c_char;

struct Sample {
    x: i32,
}

#[ctor]
fn load() {
    println!("Loaded foo module")
}

#[dtor]
fn unload() {
    println!("Unloaded foo module")
}

#[no_mangle]
pub extern "C" fn MmiGetInfo(
    client_name: *const c_char,
    payload: *mut MmiJsonString,
    payload_size_bytes: *mut c_int,
) -> c_int {
    if payload.is_null() {
        println!("MmiGetInfo called with null payload");
        return EINVAL;
    }

    if payload_size_bytes.is_null() {
        println!("MmiGetInfo called with null payloadSizeBytes");
        return EINVAL;
    }

    // let value = serde_json::json!({
    //     "name": "Foo",
    //     "description": "Foo module for...",
    //     "components": ["Foo"]
    // });

    let value = serde_json::json!({
        "Name": "Firewall",
        "Description": "Provides functionality to remotely manage firewall rules on device",
        "Manufacturer": "Microsoft",
        "VersionMajor": 4,
        "VersionMinor": 0,
        "VersionInfo": "Zinc",
        "Components": ["Firewall"],
        "Lifetime": 1,
        "UserAccount": 0
    });

    let value = serde_json::to_string(&value).unwrap();
    let value = CString::new(value).unwrap();
    let size = value.as_bytes().len();

    let ptr: MmiJsonString = CString::into_raw(value);

    unsafe {
        *payload = ptr;
        *payload_size_bytes = size as i32;
    }

    MMI_OK
}

#[no_mangle]
pub extern "C" fn MmiOpen(client_name: *const c_char, max_payload_size_bytes: c_uint) -> MmiHandle {
    let handle = Box::<Sample>::new(Sample { x: 42 });
    Box::into_raw(handle) as *mut c_void
}

#[no_mangle]
pub extern "C" fn MmiClose(client_session: MmiHandle) {
    if !client_session.is_null() {
        let _ = unsafe { Box::from_raw(client_session as *mut Sample) };
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
    println!("MmiSet called");

    if payload.is_null() {
        println!("MmiSet called with null payload");
        return EINVAL;
    }

    if payload_size_bytes < 0 {
        println!("MmiSet called with negative payloadSizeBytes");
        return EINVAL;
    }

    let payload = unsafe { std::slice::from_raw_parts(payload as *const u8, payload_size_bytes as usize) };
    let payload = String::from_utf8_lossy(payload).to_string();

    println!("MmiSet payload: {}", payload);

    MMI_OK
}

#[no_mangle]
pub extern "C" fn MmiGet(
    client_session: MmiHandle,
    component_name: *const c_char,
    object_name: *const c_char,
    payload: *mut MmiJsonString,
    payload_size_bytes: *mut c_int,
) -> c_int {
    println!("MmiGet called");

    if payload.is_null() {
        println!("MmiGet called with null payload");
        return EINVAL;
    }

    if payload_size_bytes.is_null() {
        println!("MmiGet called with null payloadSizeBytes");
        return EINVAL;
    }

    let value = serde_json::json!({
        "foo": "bar"
    });

    let value = serde_json::to_string(&value).unwrap();
    let value: CString = CString::new(value).unwrap();
    let size = value.as_bytes().len();

    let ptr: MmiJsonString = CString::into_raw(value);

    unsafe {
        *payload = ptr;
        *payload_size_bytes = size as i32;
    }

    MMI_OK
}

#[no_mangle]
pub extern "C" fn MmiFree(payload: MmiJsonString) {
    if !payload.is_null() {
        let _ = unsafe { CString::from_raw(payload) };
    }
}

// --------------------------------------------------------------------------------

/// A module named "Foo"
struct Foo;

/// The an implmentation of the properties exposed by the "Foo" module
#[module]
impl Foo {
    fn x(&self) -> i32 {
        42
    }

    fn y(&self, y: i32) {
        println!("y: {}", y);
    }
}

// The #[module] macro expands to this impl (plus the typical C interface):

impl Foo {
    fn audit(&self, object: &str) -> Result<serde_json::Value, Error> {
        match object {
            "x" => Ok(serde_json::json!(self.x())),
            _ => Err(Error::ObjectNotFound(object.to_string())),
        }
    }

    fn remediate(&self, object: &str, value: serde_json::Value) -> Result<(), Error> {
        match object {
            "y" => {
                let y = value.as_i64().unwrap() as i32;
                self.y(y);
                Ok(())
            }
            _ => Err(Error::ObjectNotFound(object.to_string())),
        }
    }
}

