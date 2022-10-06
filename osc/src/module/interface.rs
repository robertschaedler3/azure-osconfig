use libc::{c_char, c_int, c_uint, c_void, EINVAL};
use std::ffi::{CStr, CString};
use std::{slice};

pub type Handle = *mut c_void;
pub type JsonString = *mut c_char;

use super::{Interface};
use crate::error::Error;

// TODO: "MmiGetInfo()" function

pub fn open<ModuleT>(
    client_name: *const c_char,
    max_payload_size: c_uint,
) -> Result<ModuleT, Error>
where
    ModuleT: Interface,
{
    // REVIEW: does client_name need to be checked for null?
    let client_name = unsafe { CStr::from_ptr(client_name) };
    let client_name = client_name.to_str()?;
    let module = ModuleT::open(client_name, max_payload_size);
    Ok(module)
}

pub fn close<ModuleT>(client_session: Handle)
where
    ModuleT: Interface,
{
    if !client_session.is_null() {
        unsafe { Box::from_raw(client_session as *mut ModuleT) };
    }
}

pub fn set<ModuleT>(
    client_session: Handle,
    component_name: *const c_char,
    object_name: *const c_char,
    payload: JsonString,
    payload_size_bytes: c_int,
) -> Result<(), Error>
where
    ModuleT: Interface,
{
    if client_session.is_null() {
        return Err(Box::new(std::io::Error::from_raw_os_error(EINVAL)));
    }

    let component_name = unsafe { CStr::from_ptr(component_name) };
    let component_name = component_name.to_str()?;
    let object_name = unsafe { CStr::from_ptr(object_name) };
    let object_name = object_name.to_str()?;
    let payload =
        unsafe { slice::from_raw_parts(payload as *const u8, payload_size_bytes as usize) };
    let payload = String::from_utf8_lossy(payload).to_string();

    let module = unsafe { &mut *(client_session as *mut ModuleT) };
    module.set(component_name, object_name, &payload)
}

pub fn get<ModuleT>(
    client_session: Handle,
    component_name: *const c_char,
    object_name: *const c_char,
    payload: *mut JsonString,
    payload_size_bytes: *mut c_int,
) -> Result<(), Error>
where
    ModuleT: Interface,
{
    if client_session.is_null() {
        return Err(Box::new(std::io::Error::from_raw_os_error(EINVAL)));
    }

    let component_name = unsafe { CStr::from_ptr(component_name) };
    let component_name = component_name.to_str()?;
    let object_name = unsafe { CStr::from_ptr(object_name) };
    let object_name = object_name.to_str()?;

    let module = unsafe { &mut *(client_session as *mut ModuleT) };
    let value = module.get(component_name, object_name)?;
    let json = serde_json::to_string(&value)?;
    let json = CString::new(json.as_str())?;
    let size = json.as_bytes().len() as c_int;
    unsafe {
        *payload = json.into_raw();
        *payload_size_bytes = size as c_int;
    };

    Ok(())
}

// TODO: "MmiFree()" function
