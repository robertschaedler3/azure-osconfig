use std::os::raw::{c_char, c_int, c_uint};

pub type MmiHandle = super::interface::Handle;
pub type MmiJsonString = super::interface::JsonString;

pub type MmiGetInfo = extern "C" fn(
    client_name: *const c_char,
    payload: *mut MmiJsonString,
    payload_size_bytes: *mut c_int,
) -> c_int;
pub type MmiOpen = extern "C" fn(client_name: *const c_char, max_payload_size: c_uint) -> MmiHandle;
pub type MmiClose = extern "C" fn(handle: MmiHandle);
pub type MmiSet = extern "C" fn(
    handle: MmiHandle,
    component_name: *const c_char,
    object_name: *const c_char,
    payload: MmiJsonString,
    _payload_size_bytes: c_int,
) -> c_int;
pub type MmiGet = extern "C" fn(
    handle: MmiHandle,
    component_name: *const c_char,
    object_name: *const c_char,
    payload: *mut MmiJsonString,
    payload_size_bytes: *mut c_int,
) -> c_int;
