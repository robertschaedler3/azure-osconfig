use std::ffi::{c_char, c_int, c_uint, c_void};

use errno::Errno;
use trace::trace;

use super::sharedlib::{FuncArc, Symbol};

trace::init_depth_var!();

pub const INFO: &str = "MmiGetInfo";
pub const OPEN: &str = "MmiOpen";
pub const CLOSE: &str = "MmiClose";
pub const SET: &str = "MmiSet";
pub const GET: &str = "MmiGet";

pub type Handle = *mut c_void;
pub type JsonString = *mut c_char;

pub type Info = extern "C" fn(client_name: *const c_char, payload: *mut JsonString, payload_size_bytes: *mut c_int) -> c_int;
pub type Open = extern "C" fn(client_name: *const c_char, max_payload_size: c_uint) -> Handle;
pub type Close = extern "C" fn(handle: Handle);
pub type Set = extern "C" fn(handle: Handle, component_name: *const c_char, object_name: *const c_char, payload: JsonString, payload_size_bytes: c_int) -> c_int;
pub type Get = extern "C" fn(handle: Handle, component_name: *const c_char, object_name: *const c_char, payload: *mut JsonString, payload_size_bytes: *mut c_int) -> c_int;

fn check_err(num: i32) -> Result<(), errno::Errno> {
    if num != 0 {
        return Err(errno::Errno(num));
    }
    Ok(())
}

// TODO: fix enter/exit trace formatting for these functions... for example:
// #[trace(format_enter = "SharedLibClient::get({component}, {object})", format_exit = "SharedLibClient::get({component}, {object}) returned {r}")]

#[trace(logging)]
pub fn call_info(info: FuncArc<Info>, client_name: *const c_char, payload: *mut JsonString, payload_size_bytes: *mut c_int) -> Result<(), Errno> {
    check_err((unsafe { info.get() })(client_name, payload, payload_size_bytes))
}

#[trace(logging)]
pub fn call_open(open: FuncArc<Open>, client_name: *const c_char, max_payload_size_bytes: c_uint) -> Handle {
    (unsafe { open.get() })(client_name, max_payload_size_bytes)
}

#[trace(logging)]
pub fn call_close(close: FuncArc<Close>, handle: Handle) {
    (unsafe { close.get() })(handle)
}

#[trace(logging)]
pub fn call_set(set: FuncArc<Set>, handle: Handle, component_name: *const c_char, object_name: *const c_char, payload: JsonString, payload_size_bytes: c_int) -> Result<(), Errno> {
    check_err((unsafe { set.get() })(handle, component_name, object_name, payload, payload_size_bytes))
}

#[trace(logging)]
pub fn call_get(get: FuncArc<Get>, handle: Handle, component_name: *const c_char, object_name: *const c_char, payload: *mut JsonString, payload_size_bytes: *mut c_int) -> Result<(), Errno> {
    check_err((unsafe { get.get() })(handle, component_name, object_name, payload, payload_size_bytes))
}
