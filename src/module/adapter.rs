use std::{ffi::CString, path::Path};

use super::{
    bindings::*,
    meta::Metadata,
    sharedlib::{FuncArc, LibArc},
    Payload,
};

use crate::{error::Error, PLATFORM_CLIENT};

pub trait Adapter: Sized {
    fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error>;
    fn meta(&self) -> Result<Metadata, Error>;
    fn get(&self, component: &str, object: &str) -> Result<Payload, Error>;
    fn set(&self, component: &str, object: &str, payload: &Payload) -> Result<(), Error>;
}

#[derive(Clone)]
struct Context(Handle);

unsafe impl Send for Context {}

impl From<Handle> for Context {
    fn from(handle: Handle) -> Self {
        Self(handle)
    }
}

pub struct ModuleAdapter {
    info: FuncArc<Info>,
    close: FuncArc<Close>,
    set: FuncArc<Set>,
    get: FuncArc<Get>,
    context: Context,
}

impl Adapter for ModuleAdapter {
    fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let lib: LibArc = unsafe { LibArc::new(path) }?;

        let info: FuncArc<Info> = unsafe { lib.find_func(INFO) }?;
        let open: FuncArc<Open> = unsafe { lib.find_func(OPEN) }?;
        let close: FuncArc<Close> = unsafe { lib.find_func(CLOSE) }?;
        let get: FuncArc<Get> = unsafe { lib.find_func(GET) }?;
        let set: FuncArc<Set> = unsafe { lib.find_func(SET) }?;

        let context = call_open(open, CString::new(PLATFORM_CLIENT)?.as_ptr(), 0).into();

        Ok(Self {
            info,
            close,
            set,
            get,
            context,
        })
    }

    fn meta(&self) -> Result<Metadata, Error> {
        let client_name = CString::new(PLATFORM_CLIENT)?;
        let mut payload: JsonString = std::ptr::null_mut();
        let mut payload_size_bytes = 0;

        call_info(
            self.info.clone(),
            client_name.as_ptr(),
            &mut payload,
            &mut payload_size_bytes,
        )?;

        let payload = unsafe {
            std::slice::from_raw_parts(payload as *const u8, payload_size_bytes as usize)
        };
        let payload = String::from_utf8_lossy(payload).to_string();

        Ok(serde_json::from_str(&payload)?)
    }

    fn get(&self, component: &str, object: &str) -> Result<serde_json::Value, Error> {
        let component = CString::new(component)?;
        let object = CString::new(object)?;
        let mut payload: JsonString = std::ptr::null_mut();
        let mut payload_size_bytes = 0;

        call_get(
            self.get.clone(),
            self.context.0,
            component.as_ptr(),
            object.as_ptr(),
            &mut payload,
            &mut payload_size_bytes,
        )?;

        let payload = unsafe {
            std::slice::from_raw_parts(payload as *const u8, payload_size_bytes as usize)
        };
        let payload = String::from_utf8_lossy(payload).to_string();

        Ok(serde_json::from_str(&payload)?)
    }

    fn set(&self, component: &str, object: &str, payload: &serde_json::Value) -> Result<(), Error> {
        let component = CString::new(component)?;
        let object = CString::new(object)?;
        let payload = serde_json::to_string(&payload)?;
        let payload = CString::new(payload)?;
        let size = payload.as_bytes().len() as i32;

        call_set(
            self.set.clone(),
            self.context.0,
            component.as_ptr(),
            object.as_ptr(),
            payload.as_ptr() as JsonString,
            size,
        )?;

        Ok(())
    }
}

impl Drop for ModuleAdapter {
    fn drop(&mut self) {
        call_close(self.close.clone(), self.context.0);
    }
}
