use osc::osc_module;

// REVIEW: try to do this without Default
#[derive(Default)]
struct Hostname {
    name: String,
}

#[osc_module]
impl Hostname {
    #[reported(name = "name")]
    fn name(&self) -> String {
        self.name.clone()
    }

    #[desired(name = "name")]
    fn desired_name(&mut self, name: String) {
        self.name = name;
    }
}

// --------------------------------------------------------------------------------

// IDEAS:

#[osc_module(Component1, Component2)]
struct MyModule;

struct Component1 {
    foo: String,
}

#[osc_component]
impl Component1 {
    #[reported]
    fn foo(&self) -> String {
        self.foo.clone()
    }

    #[desired(name = "foo")]
    fn desired_foo(&mut self, foo: String) {
        self.foo = foo;
    }
}

#[osc_component]
impl Component2 {
    #[desired]
    fn bar(&self, blah: String) -> Result<(), Error> {
        println!("bar: {}", blah);
    }

    #[reported]
    fn baz(&self) -> Baz {
        Baz {
            something: "blah".to_string(),
        }
    }
}

#[osc_object]
struct Baz {
    something: String,
}

// ABOVE CODE SHOULD EXPAND TO:

struct MyModule {
    component1: Component1,
    component2: Component2,
}

impl MyModule {
    fn new() -> Self {
        Self {
            component1: Component1::new(),
            component2: Component2::new(),
        }
    }

    fn get(&self, component: &str, object: &str) -> Result<String, Error> {
        match component {
            "component1" => self.component1.get(object),
            "component2" => self.component2.get(object),
            _ => Err(Error::InvalidComponent(component.to_string())),
        }
    }

    fn set(&mut self, component: &str, object: &str, value: Value) -> Result<(), Error> {
        match component {
            "component1" => self.component1.set(object, value),
            "component2" => self.component2.set(object, value),
            _ => Err(Error::InvalidComponent(component.to_string())),
        }
    }
}

impl Component1 {
    fn new() -> Self {
        Self {
            foo: "foo".to_string(),
        }
    }

    fn get(&self, object: &str) -> Result<String, Error> {
        match object {
            "foo" => Ok(self.foo.clone()),
            _ => Err(Error::InvalidObject(object.to_string())),
        }
    }

    fn set(&mut self, object: &str, value: Value) -> Result<(), Error> {
        match object {
            "foo" => {
                self.foo = value.as_str().unwrap().to_string();
                Ok(())
            }
            _ => Err(Error::InvalidObject(object.to_string())),
        }
    }
}

impl Component2 {
    fn new() -> Self {
        Self {}
    }

    fn get(&self, object: &str) -> Result<String, Error> {
        match object {
            "baz" => Ok(serde_json::to_string(&self.baz())?),
            _ => Err(Error::InvalidObject(object.to_string())),
        }
    }

    fn set(&mut self, object: &str, value: Value) -> Result<(), Error> {
        match object {
            "bar" => {
                self.bar(value.as_str().unwrap().to_string());
                Ok(())
            }
            _ => Err(Error::InvalidObject(object.to_string())),
        }
    }
}



// --------------------------------------------------------------------------------

use libc::{c_char, c_int};
use serde_json::Value;
use std::ffi::{CStr, CString};
use std::{env, ptr};

use osc::module::interface::{Handle, JsonString};

fn get_args() -> Option<(String, String)> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        return None;
    }

    let component = &args[1];
    let object = &args[2];

    Some((component.to_string(), object.to_string()))
}

fn call_get(handle: Handle, component: &str, object: &str) {
    println!("Get({:?}, {}, {})", handle, component, object);

    let component = CString::new(component).unwrap();
    let object = CString::new(object).unwrap();

    let mut payload: JsonString = ptr::null_mut();
    let mut payload_size_bytes: c_int = 0;

    let result = MmiGet(
        handle,
        component.as_ptr(),
        object.as_ptr(),
        &mut payload,
        &mut payload_size_bytes,
    );

    if result == 0 {
        let payload = unsafe { CStr::from_ptr(payload) };
        let payload = payload.to_str().unwrap();
        let value: Value = serde_json::from_str(&payload).unwrap();
        // let complex = serde_json::from_value::<Complex>(value).unwrap();
        let value = serde_json::to_string_pretty(&value).unwrap();
        println!("{}", value);
    } else {
        println!("{}", result);
    }
}

fn call_set<T>(handle: Handle, component: &str, object: &str, value: T)
where
    T: serde::Serialize + std::fmt::Debug,
{
    println!("Set({:?}, {}, {}, {:?})", handle, component, object, value);

    let component = CString::new(component).unwrap();
    let object = CString::new(object).unwrap();

    let value = serde_json::to_string(&value).unwrap();
    let value = CString::new(value).unwrap();

    let result = MmiSet(
        handle,
        component.as_ptr(),
        object.as_ptr(),
        value.as_ptr() as JsonString,
        value.as_bytes().len() as c_int,
    );

    if result != 0 {
        println!("{}", result);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (component, object) = get_args().ok_or_else(|| "Usage: <component> <object>")?;

    let blah = CString::new("blah").unwrap();
    let handle = MmiOpen(blah.as_ptr() as *const c_char, 1024);

    let value = "my_computer";

    call_set(handle, &component, &object, value);
    call_get(handle, &component, &object);

    MmiClose(handle);

    Ok(())
}
