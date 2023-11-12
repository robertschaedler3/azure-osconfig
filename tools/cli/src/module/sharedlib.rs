use std::{
    ffi::CString,
    os::raw::{c_char, c_int, c_uint},
    path::Path,
    sync::Arc,
};

use libloading::Library;
use osc::module::{
    bind::{Close, Get, Handle, Info, JsonString, Open, Set, CLOSE, GET, INFO, OPEN, SET},
    Meta,
};
use trace::trace;

use crate::PLATFORM_CLIENT;

use super::Client;

trace::init_depth_var!();

#[derive(Clone)]
struct Context(Handle);

unsafe impl Send for Context {}

impl From<Handle> for Context {
    fn from(handle: Handle) -> Self {
        Self(handle)
    }
}

/// TODO: note why we can't call close in Drop if this is Clone
pub struct SharedLibClient {
    info: FuncArc<Info>,
    close: FuncArc<Close>,
    set: FuncArc<Set>,
    get: FuncArc<Get>,
    context: Context,
}

impl Drop for SharedLibClient {
    fn drop(&mut self) {
        call_close(self.close.clone(), self.context.0);
    }
}

impl Client for SharedLibClient {
    fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let lib: LibArc = unsafe { LibArc::new(path) }?;

        let info: FuncArc<Info> = unsafe { lib.find_func(INFO) }?;
        let open: FuncArc<Open> = unsafe { lib.find_func(OPEN) }?;
        let close: FuncArc<Close> = unsafe { lib.find_func(CLOSE) }?;
        let get: FuncArc<Get> = unsafe { lib.find_func(GET) }?;
        let set: FuncArc<Set> = unsafe { lib.find_func(SET) }?;

        // REVIEW: what max_payload_size_bytes should be used here?
        let context = call_open(open, CString::new(PLATFORM_CLIENT)?.as_ptr(), 0).into();

        Ok(Self {
            info,
            close,
            set,
            get,
            context,
        })
    }

    fn meta(&self) -> anyhow::Result<Meta> {
        let client_name = CString::new(PLATFORM_CLIENT)?;
        let mut payload: JsonString = std::ptr::null_mut();
        let mut payload_size_bytes: c_int = 0;

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

    fn get(&self, component: &str, object: &str) -> anyhow::Result<serde_json::Value> {
        let component = CString::new(component)?;
        let object = CString::new(object)?;
        let mut payload: JsonString = std::ptr::null_mut();
        let mut payload_size_bytes: c_int = 0;

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

    fn set(
        &self,
        component: &str,
        object: &str,
        payload: &serde_json::Value,
    ) -> anyhow::Result<()> {
        let component = CString::new(component)?;
        let object = CString::new(object)?;
        let payload = serde_json::to_string(&payload)?;
        let payload = CString::new(payload)?;
        let size = payload.as_bytes().len() as c_int;

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

// REVIEW: fix enter/exit trace formatting for these functions... for example:
// #[trace(format_enter = "SharedLibClient::get({component}, {object})", format_exit = "SharedLibClient::get({component}, {object}) returned {r}")]

#[trace(logging)]
fn call_info(
    info: FuncArc<Info>,
    client_name: *const c_char,
    payload: *mut JsonString,
    payload_size_bytes: *mut c_int,
) -> anyhow::Result<()> {
    check_err((unsafe { info.get() })(
        client_name,
        payload,
        payload_size_bytes,
    ))
}

#[trace(logging)]
fn call_open(
    open: FuncArc<Open>,
    client_name: *const c_char,
    max_payload_size_bytes: c_uint,
) -> Handle {
    (unsafe { open.get() })(client_name, max_payload_size_bytes)
}

#[trace(logging)]
fn call_close(close: FuncArc<Close>, handle: Handle) {
    (unsafe { close.get() })(handle)
}

#[trace(logging)]
fn call_set(
    set: FuncArc<Set>,
    handle: Handle,
    component_name: *const c_char,
    object_name: *const c_char,
    payload: JsonString,
    payload_size_bytes: c_int,
) -> anyhow::Result<()> {
    check_err((unsafe { set.get() })(
        handle,
        component_name,
        object_name,
        payload,
        payload_size_bytes,
    ))
}

#[trace(logging)]
fn call_get(
    get: FuncArc<Get>,
    handle: Handle,
    component_name: *const c_char,
    object_name: *const c_char,
    payload: *mut JsonString,
    payload_size_bytes: *mut c_int,
) -> anyhow::Result<()> {
    check_err((unsafe { get.get() })(
        handle,
        component_name,
        object_name,
        payload,
        payload_size_bytes,
    ))
}

fn check_err(num: i32) -> anyhow::Result<()> {
    if num != 0 {
        return Err(errno::Errno(num).into());
    }
    Ok(())
}

// A symbol from a shared library.
trait Symbol<T> {
    /// Provides access to the data that this symbol references.
    ///
    /// # Unsafety
    /// If the data that this symbol references contains pointers to other things in the shared
    /// library, and `T: Clone`, we can obtain a clone of the data and use it to outlast the
    /// library. To prevent this, the return of this function should never be cloned.
    unsafe fn get(&self) -> T;
}

/// A pointer to a shared function which provides no protection against outliving its library.
type FuncUnsafe<T> = T;

/// A pointer to a shared function which allows a user-provided ref-counting implementation to avoid outliving its library.
#[derive(Debug)]
struct FuncTracked<T, TLib> {
    func: FuncUnsafe<T>,
    lib: TLib,
}

/// A pointer to a shared function which uses atomic ref-counting to avoid outliving its library.
type FuncArc<T> = FuncTracked<T, Arc<LibUnsafe>>;

/// A shared library which does not track its [`Symbols`].
/// The inner library may be dropped at any time, even if it has loose symbols.
type LibUnsafe = Library;

/// A shared library which implements [LibTracked](struct.LibTracked.html) with atomic ref-counting to track its [Symbols](trait.Symbol.html).
type LibArc = LibTracked<Arc<LibUnsafe>>;

/// A shared library which which allows a user-provided ref-counting implementation to track its [`Symbols`].
/// The inner library will not be droped until all of the ref-counts are dropped.
#[derive(Clone, Debug)]
struct LibTracked<TLib> {
    inner: TLib,
}

impl<TLib> LibTracked<TLib>
where
    TLib: AsRef<LibUnsafe> + Clone + From<LibUnsafe>,
{
    unsafe fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let lib = LibUnsafe::new(path.as_ref())?;
        let inner = TLib::from(lib);
        Ok(LibTracked { inner: inner })
    }

    /// Finds and returns a function symbol of the shared library.
    unsafe fn find_func<T: Copy, TStr: AsRef<str>>(
        &self,
        symbol: TStr,
    ) -> anyhow::Result<FuncTracked<T, TLib>> {
        let lib = self.inner.as_ref();
        let func = lib.get::<T>(symbol.as_ref().as_bytes())?;
        let func = std::mem::transmute_copy(&func);
        Ok(FuncTracked::new(func, self.inner.clone()))
    }
}

impl<T, TLib> FuncTracked<T, TLib> {
    /// Creates a new [FuncTracked](struct.FuncTracked.html).
    /// This should only be called within the library.
    fn new(func: FuncUnsafe<T>, lib: TLib) -> Self {
        FuncTracked {
            func: func,
            lib: lib,
        }
    }
}

impl<T: Copy> Symbol<T> for FuncUnsafe<T> {
    unsafe fn get(&self) -> T {
        self.clone()
    }
}

impl<T: Copy, TLib> Symbol<T> for FuncTracked<T, TLib> {
    unsafe fn get(&self) -> T {
        self.func
    }
}

impl<T: Copy, TLib: Clone> Clone for FuncTracked<T, TLib> {
    fn clone(&self) -> Self {
        FuncTracked {
            func: self.func,
            lib: self.lib.clone(),
        }
    }
}
