use sharedlib::{FuncArc, LibArc, Symbol};

use std::ffi::{CStr, CString};

// TODO: move to "bindings"
use std::os::raw::{c_char, c_int, c_uint, c_void};

// TODO: move all these types to a better location (maybe shared lib for macros?)
type MmiHandle = *mut c_void;
type MmiJsonString = *const c_char;

type MmiGetInfo = extern "C" fn(
    client_name: *const c_char,
    payload: *mut MmiJsonString,
    payload_size_bytes: *mut c_int,
) -> c_int;
type MmiOpen = extern "C" fn(client_name: *const c_char, max_payload_size: c_uint) -> MmiHandle;
type MmiClose = extern "C" fn(handle: MmiHandle);
type MmiSet = extern "C" fn(
    handle: MmiHandle,
    component_name: *const c_char,
    object_name: *const c_char,
    payload: MmiJsonString,
    _payload_size_bytes: c_int,
) -> c_int;
type MmiGet = extern "C" fn(
    handle: MmiHandle,
    component_name: *const c_char,
    object_name: *const c_char,
    payload: *mut MmiJsonString,
    payload_size_bytes: *mut c_int,
) -> c_int;

#[derive(Debug)]
struct Mmi {
    lib: LibArc,
    // get_info: FuncArc<MmiGetInfo>,
    open: FuncArc<MmiOpen>,
    close: FuncArc<MmiClose>,
    set: FuncArc<MmiSet>,
    get: FuncArc<MmiGet>,
}

#[derive(Debug)]
struct Module {
    path: String,
    components: Vec<String>,
    interface: Option<Mmi>,
}

#[derive(Debug, Clone)]
pub struct Error(String);

impl Module {
    // TODO: use load() as "constructor"
    fn new(path: &str) -> Self {
        Module {
            path: path.to_string(),
            components: Vec::new(),
            interface: None,
        }
    }

    // TODO: return Result<Self, Error>
    pub fn load(&mut self, client_name: &str) {
        let lib = unsafe { LibArc::new(&self.path) };
        match lib {
            Ok(lib) => {
                unsafe {
                    // TODO: handle errors from unwrap(s) and propogate error in Result<>
                    let get_info: FuncArc<MmiGetInfo> = lib.find_func("MmiGetInfo").unwrap();
                    let open: FuncArc<MmiOpen> = lib.find_func("MmiOpen").unwrap();
                    let close: FuncArc<MmiClose> = lib.find_func("MmiClose").unwrap();
                    let set: FuncArc<MmiSet> = lib.find_func("MmiSet").unwrap();
                    let get: FuncArc<MmiGet> = lib.find_func("MmiGet").unwrap();

                    let client_name = CString::new(client_name).unwrap();
                    let get_info = get_info.get();
                    let mut payload_size_bytes = 0;
                    let mut payload = std::mem::zeroed();
                    let result =
                        get_info(client_name.as_ptr(), &mut payload, &mut payload_size_bytes);
                    if result == 0 {
                        // Read the payload into a string using the returned size.
                        let payload = std::slice::from_raw_parts(
                            payload as *const u8,
                            payload_size_bytes as usize,
                        );
                        let payload = String::from_utf8_lossy(payload);
                        let payload: serde_json::Value = serde_json::from_str(&payload).unwrap();
                        let components = payload["Components"].as_array().unwrap();
                        self.components = components
                            .iter()
                            .map(|c| c.as_str().unwrap().to_string())
                            .collect();
                    } else {
                        println!("MmiGetInfo failed: {}", result);
                    }

                    self.interface = Some(Mmi {
                        lib,
                        // get_info,
                        open,
                        close,
                        set,
                        get,
                    });
                }
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    }

    pub fn open(&self, client_name: &str, max_payload_size: u32) -> Result<MmiHandle, Error> {
        if self.interface.is_none() {
            return Err(Error("Module not loaded".to_string()));
        }

        let client_name = CString::new(client_name).unwrap();
        let open = unsafe { self.interface.as_ref().unwrap().open.get() };
        let handle = open(client_name.as_ptr(), max_payload_size);
        if handle.is_null() {
            Err(Error("Open failed".to_string()))
        } else {
            Ok(handle)
        }
    }

    pub fn close(&self, handle: MmiHandle) {
        let close = unsafe { self.interface.as_ref().unwrap().close.get() };
        close(handle);
    }

    // pub fn get_info(&self, client_name: &str) -> Result<String, Error> {
    // }

    pub fn set(
        &self,
        handle: MmiHandle,
        component_name: &str,
        object_name: &str,
        payload: &str,
    ) -> Result<(), Error> {
        let component_name = CString::new(component_name).unwrap();
        let object_name = CString::new(object_name).unwrap();
        let payload = CString::new(payload).unwrap();
        let set = unsafe { self.interface.as_ref().unwrap().set.get() };
        let result = set(
            handle,
            component_name.as_ptr(),
            object_name.as_ptr(),
            payload.as_ptr(),
            payload.as_bytes().len() as i32,
        );
        if result == 0 {
            Ok(())
        } else {
            Err(Error("Set failed".to_string()))
        }
    }

    pub fn get(
        &self,
        handle: MmiHandle,
        component_name: &str,
        object_name: &str,
    ) -> Result<String, Error> {
        if self.interface.is_none() {
            return Err(Error("Module not loaded".to_string()));
        }

        let component_name = CString::new(component_name).unwrap();
        let object_name = CString::new(object_name).unwrap();
        let get = unsafe { self.interface.as_ref().unwrap().get.get() };
        let mut payload_size_bytes = 0;
        let mut payload = unsafe { std::mem::zeroed() };
        let result = get(
            handle,
            component_name.as_ptr(),
            object_name.as_ptr(),
            &mut payload,
            &mut payload_size_bytes,
        );
        if result == 0 {
            let payload = unsafe { CStr::from_ptr(payload) };
            Ok(payload.to_string_lossy().into_owned())
        } else {
            Err(Error("Get failed".to_string()))
        }
    }
}

#[derive(Debug, Default)]
pub struct ModuleManager {
    modules: Vec<Box<Module>>,
}

impl ModuleManager {
    // pub fn new(client_name: &str, path: &str) -> Self {
    //     let mut manager = ModuleManager::default();
    //     manager.load(client_name, path);
    //     manager
    // }

    // TODO: return Result<Self, Error>
    pub fn load(&mut self, client_name: &str, path: &str) {
        // List all files in the directory
        let mut files = Vec::new();
        let dir = std::fs::read_dir(path).unwrap();
        for entry in dir {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                let filename = path.file_name().unwrap().to_str().unwrap();
                if filename.ends_with(".so") {
                    println!("Loading {}", path.to_str().unwrap());
                    files.push(path.to_str().unwrap().to_string());
                }
            }
        }

        // Load each module
        for filename in files {
            let mut module = Box::new(Module::new(&filename));
            module.load(client_name);
            self.modules.push(module);
        }
    }

    pub fn get(&self, client: &str, component: &str, object: &str) -> Result<String, Error> {
        let mut module = None;
        for m in &self.modules {
            if m.components.contains(&component.to_string()) {
                module = Some(m);
                break;
            }
        }
        match module {
            Some(module) => {
                let handle = module.open(client, 0)?;
                let payload = module.as_ref().get(handle, component, object);
                module.close(handle);
                payload
            }
            None => Err(Error("Module not found".to_string())),
        }
    }
}
