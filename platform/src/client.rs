use std::{
    ffi::{c_int, CString},
    path::PathBuf,
    slice,
};

use anyhow::anyhow;
use sharedlib::{FuncArc, LibArc, Symbol};

use osc::module::{bindings::*, ModuleInfo};

#[derive(Clone)]
pub struct Library {
    info: FuncArc<Info>,
    open: FuncArc<Open>,
    close: FuncArc<Close>,
    set: FuncArc<Set>,
    get: FuncArc<Get>,
}

#[derive(Clone)]
pub struct Session(Handle);

// https://stackoverflow.com/questions/60292897/why-cant-i-send-mutexmut-c-void-between-threads
unsafe impl Send for Session {}

// TODO: impl Client for Library
impl Library {
    pub fn load(path: PathBuf) -> anyhow::Result<Self> {
        if path.extension().unwrap() != "so" {
            return Err(anyhow::anyhow!("Invalid module file extension"));
        }

        log::info!("Loading module: {:?}", path);

        unsafe {
            let lib = LibArc::new(path).map_err(|err| anyhow!(err.to_string()))?;

            // TODO: collect each of these into a single result and log as appropriate
            let info: FuncArc<Info> = lib
                .find_func(INFO)
                .map_err(|err| anyhow!(err.to_string()))?;
            let open: FuncArc<Open> = lib
                .find_func(OPEN)
                .map_err(|err| anyhow!(err.to_string()))?;
            let close: FuncArc<Close> = lib
                .find_func(CLOSE)
                .map_err(|err| anyhow!(err.to_string()))?;
            let set: FuncArc<Set> = lib.find_func(SET).map_err(|err| anyhow!(err.to_string()))?;
            let get: FuncArc<Get> = lib.find_func(GET).map_err(|err| anyhow!(err.to_string()))?;

            Ok(Self {
                info,
                open,
                close,
                set,
                get,
            })
        }
    }

    pub fn info(&self, client: &str) -> anyhow::Result<ModuleInfo> {
        // TODO: log traces

        let get_info = unsafe { self.info.get() };
        let client_name = CString::new(client).unwrap();
        let mut payload: JsonString = std::ptr::null_mut();
        let mut payload_size_bytes: c_int = 0;

        let status = get_info(client_name.as_ptr(), &mut payload, &mut payload_size_bytes);

        if status != MODULE_OK {
            return Err(anyhow::anyhow!("GetInfo() failed: {}", status));
        }

        let payload = unsafe { slice::from_raw_parts(payload as *const u8, payload_size_bytes as usize) };
        let payload = String::from_utf8_lossy(payload).to_string();

        let info: ModuleInfo = serde_json::from_str(&payload)?;

        Ok(info)
    }

    pub fn open(&self, client: &str, max_payload_size: u32) -> anyhow::Result<Session> {
        // TODO: log traces

        let open = unsafe { self.open.get() };
        let client_name = CString::new(client).unwrap();

        let handle = open(client_name.as_ptr(), max_payload_size);

        if handle.is_null() {
            return Err(anyhow::anyhow!("Open() failed"));
        }

        Ok(Session(handle))
    }

    pub fn close(&self, session: Session) -> anyhow::Result<()> {
        // TODO: log traces

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
        log::trace!("Set: {} {} {} {}", component, object, payload, size);

        let set = unsafe { self.set.get() };
        let handle = session.0;
        let component_name = CString::new(component).unwrap();
        let object_name = CString::new(object).unwrap();
        let value = CString::new(payload).unwrap();

        let status = set(
            handle,
            component_name.as_ptr(),
            object_name.as_ptr(),
            value.as_ptr() as JsonString,
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
        // TODO: log traces

        let get = unsafe { self.get.get() };
        let handle = session.0;
        let component_name = CString::new(component).unwrap();
        let object_name = CString::new(object).unwrap();
        let mut payload: JsonString = std::ptr::null_mut();
        let mut payload_size_bytes: c_int = 0;

        let status = get(
            handle,
            component_name.as_ptr(),
            object_name.as_ptr(),
            &mut payload,
            &mut payload_size_bytes,
        );

        let payload = unsafe { slice::from_raw_parts(payload as *const u8, payload_size_bytes as usize) };
        let payload = String::from_utf8_lossy(payload).to_string();

        Ok((status, payload))
    }
}

trait Client {
    type Handle;
    type Context;

    fn init(&self, context: Self::Context) -> Result<(), anyhow::Error>;

    fn info(&self) -> ModuleInfo;

    fn set<T>(&self, handle: Self::Handle, payload: &T) -> Result<(), anyhow::Error>
    where
        T: serde::Serialize;

    fn get<T>(&self, session: Self::Handle) -> Result<T, anyhow::Error>
    where
        T: serde::de::DeserializeOwned;
}
