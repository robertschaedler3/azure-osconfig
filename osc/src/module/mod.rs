pub mod bindings;
pub mod interface;
pub mod schema;
pub mod value;

use anyhow::anyhow;
use libc::c_int;
use serde::Deserialize;
use sharedlib::{FuncArc, LibArc, Symbol};
use std::{ffi::CString, path::PathBuf, slice};

use self::bindings::{MmiClose, MmiGet, MmiGetInfo, MmiHandle, MmiJsonString, MmiOpen, MmiSet};
use crate::error::Error;

pub const SUCCESS: i32 = 0;

// TODO: use custom MIM types
pub type Object = serde_json::Value;

pub trait Interface {
    ///
    fn open(client_name: &str, max_payload_size: u32) -> Self;

    // TODO: meta() for get_info

    ///
    fn get(&self, component_name: &str, object_name: &str) -> Result<Object, Error>;

    ///
    fn set(&mut self, component_name: &str, object_name: &str, value: &str) -> Result<(), Error>;
}

pub trait Component {
    /// The name of the component
    fn name(&self) -> &str;

    // fn meta(&self) -> Meta;

    /// Gets a reported object
    fn reported(&self, object_name: &str) -> Result<Object, Error>;

    /// Sets a desired object with the given value
    fn desired(&mut self, object_name: &str, value: Object) -> Result<(), Error>;
    // TODO: fn desired<T>(&mut self, object_name: &str, value: Object) -> Result<T, Error>;
}

// TODO: Meta struct for "GetInfo()" type
// TODO: Ideally this struct should be able to contain a full schema of the component
//       and its properties/objects (especaiily easy to add with macros)

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")] // TODO: this seems bad to be PascalCase
pub struct Info {
    pub name: String,
    pub description: String,
    pub manufacturer: String,
    // pub version: String, // TODO: Version struct ???
    pub components: Vec<String>,
    // TODO:
    // - version info
    // - lifetime
    // - license URI
    // - project URI
    // - user account
}

/// A struct representation of a module's shared library.
pub struct Library {
    // lib: LibArc,
    get_info: FuncArc<MmiGetInfo>,
    open: FuncArc<MmiOpen>,
    close: FuncArc<MmiClose>,
    set: FuncArc<MmiSet>,
    get: FuncArc<MmiGet>,
}

pub struct Session(MmiHandle);

impl Library {
    pub fn load(path: PathBuf) -> anyhow::Result<Self> {
        if path.extension().unwrap() != "so" {
            return Err(anyhow::anyhow!("Invalid module file extension"));
        }

        unsafe {
            let lib = LibArc::new(path).map_err(|err| anyhow!(err.to_string()))?;

            let get_info: FuncArc<MmiGetInfo> = lib
                .find_func("MmiGetInfo")
                .map_err(|err| anyhow!(err.to_string()))?;
            let open: FuncArc<MmiOpen> = lib
                .find_func("MmiOpen")
                .map_err(|err| anyhow!(err.to_string()))?;
            let close: FuncArc<MmiClose> = lib
                .find_func("MmiClose")
                .map_err(|err| anyhow!(err.to_string()))?;
            let set: FuncArc<MmiSet> = lib
                .find_func("MmiSet")
                .map_err(|err| anyhow!(err.to_string()))?;
            let get: FuncArc<MmiGet> = lib
                .find_func("MmiGet")
                .map_err(|err| anyhow!(err.to_string()))?;

            Ok(Self {
                // lib,
                get_info,
                open,
                close,
                set,
                get,
            })
        }
    }

    pub fn info(&self, client: &str) -> anyhow::Result<Info> {
        let get_info = unsafe { self.get_info.get() };
        let client_name = CString::new(client).unwrap();
        let mut payload: MmiJsonString = std::ptr::null_mut();
        let mut payload_size_bytes: c_int = 0;

        let status = get_info(client_name.as_ptr(), &mut payload, &mut payload_size_bytes);

        if status != SUCCESS {
            return Err(anyhow::anyhow!("MmiGetInfo() failed: {}", status));
        }

        let payload =
            unsafe { slice::from_raw_parts(payload as *const u8, payload_size_bytes as usize) };
        let payload = String::from_utf8_lossy(payload).to_string();

        let info: Info = serde_json::from_str(&payload)?;

        Ok(info)
    }

    pub fn open(&self, client: &str, max_payload_size: u32) -> anyhow::Result<Session> {
        let open = unsafe { self.open.get() };
        let client_name = CString::new(client).unwrap();

        let handle = open(client_name.as_ptr(), max_payload_size);

        if handle.is_null() {
            return Err(anyhow::anyhow!("MmiOpen() failed"));
        }

        Ok(Session(handle))
    }

    pub fn close(&self, session: Session) -> anyhow::Result<()> {
        let close = unsafe { self.close.get() };
        let handle = session.0;

        close(handle);

        Ok(())
    }

    pub fn set(
        &self,
        session: &Session,
        component: &str,
        object: &str,
        payload: &str,
        size: usize,
    ) -> anyhow::Result<i32> {
        let set = unsafe { self.set.get() };
        let handle = session.0;
        let component_name = CString::new(component).unwrap();
        let object_name = CString::new(object).unwrap();
        let value = CString::new(payload).unwrap();

        let status = set(
            handle,
            component_name.as_ptr(),
            object_name.as_ptr(),
            value.as_ptr() as MmiJsonString,
            size as i32,
        );

        Ok(status)
    }

    pub fn get(
        &self,
        session: &Session,
        component: &str,
        object: &str,
    ) -> anyhow::Result<(i32, String)> {
        let get = unsafe { self.get.get() };
        let handle = session.0;
        let component_name = CString::new(component).unwrap();
        let object_name = CString::new(object).unwrap();
        let mut payload: MmiJsonString = std::ptr::null_mut();
        let mut payload_size_bytes: c_int = 0;

        let status = get(
            handle,
            component_name.as_ptr(),
            object_name.as_ptr(),
            &mut payload,
            &mut payload_size_bytes,
        );

        let payload =
            unsafe { slice::from_raw_parts(payload as *const u8, payload_size_bytes as usize) };
        let payload = String::from_utf8_lossy(payload).to_string();

        Ok((status, payload))
    }
}
