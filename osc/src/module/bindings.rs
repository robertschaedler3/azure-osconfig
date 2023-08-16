use std::os::raw::{c_char, c_int, c_uint, c_void};

pub const INFO: &str = "MmiGetInfo";
pub const OPEN: &str = "MmiOpen";
pub const CLOSE: &str = "MmiClose";
pub const GET: &str = "MmiGet";
pub const SET: &str = "MmiSet";
pub const FREE: &str = "MmiFree";

pub const MODULE_OK: i32 = 0;

pub type Handle = *mut c_void;
pub type JsonString = *mut c_char;

pub type Info = extern "C" fn(
    client_name: *const c_char,
    payload: *mut JsonString,
    payload_size_bytes: *mut c_int,
) -> c_int;

pub type Open = extern "C" fn(client_name: *const c_char, max_payload_size: c_uint) -> Handle;

pub type Close = extern "C" fn(handle: Handle);

pub type Set = extern "C" fn(
    handle: Handle,
    component_name: *const c_char,
    object_name: *const c_char,
    payload: JsonString,
    _payload_size_bytes: c_int,
) -> c_int;

pub type Get = extern "C" fn(
    handle: Handle,
    component_name: *const c_char,
    object_name: *const c_char,
    payload: *mut JsonString,
    payload_size_bytes: *mut c_int,
) -> c_int;

// TODO: "Free"
