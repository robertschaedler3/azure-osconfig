use std::ffi::{c_char, c_void, c_int, c_uint};

pub use libc;

pub const OK: i32 = 0;
pub const ERROR: i32 = -1;

pub const INFO: &str = "MmiGetInfo";
pub const OPEN: &str = "MmiOpen";
pub const CLOSE: &str = "MmiClose";
pub const SET: &str = "MmiSet";
pub const GET: &str = "MmiGet";

pub type Handle = *mut c_void;
pub type JsonString = *mut c_char;

pub type Info = extern "C" fn(client: *const c_char, payload: *mut JsonString, size: *mut c_int) -> c_int;
pub type Open = extern "C" fn(client: *const c_char, max_payload_size: c_uint) -> Handle;
pub type Close = extern "C" fn(handle: Handle);
pub type Set = extern "C" fn(handle: Handle, component: *const c_char, property: *const c_char, payload: JsonString, size: c_int) -> c_int;
pub type Get = extern "C" fn(handle: Handle, component: *const c_char, property: *const c_char, payload: *mut JsonString, size: *mut c_int) -> c_int;
