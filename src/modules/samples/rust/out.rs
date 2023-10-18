#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use osc::osc_module;
use serde::Serialize;
struct Sample {
    x: i32,
}
#[automatically_derived]
impl ::core::default::Default for Sample {
    #[inline]
    fn default() -> Sample {
        Sample {
            x: ::core::default::Default::default(),
        }
    }
}
impl Sample {
    fn desired_simple(&mut self, x: i32) {
        self.x = x;
    }
    fn simple(&self) -> Option<i32> {
        Some(self.x)
    }
    fn complex_1() -> Foo {
        Foo {
            x: 42,
            y: "hello".to_string(),
        }
    }
}
extern fn __ctor__() {
    ::osc::log::init();
    {
        let lvl = ::log::Level::Info;
        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
            ::log::__private_api::log(
                format_args!("loaded: {0}", "Sample"),
                lvl,
                &("rust_sample", "rust_sample", "src/modules/samples/rust/src/lib.rs"),
                12u32,
                ::log::__private_api::Option::None,
            );
        }
    };
}
#[used]
#[allow(non_upper_case_globals)]
#[doc(hidden)]
#[link_section = ".init_array"]
static __ctor_____rust_ctor___ctor: unsafe extern "C" fn() -> usize = {
    #[link_section = ".text.startup"]
    unsafe extern "C" fn __ctor_____rust_ctor___ctor() -> usize {
        __ctor__();
        0
    }
    __ctor_____rust_ctor___ctor
};
extern fn __dtor__() {
    {
        let lvl = ::log::Level::Info;
        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
            ::log::__private_api::log(
                format_args!("unloaded: {0}", "Sample"),
                lvl,
                &("rust_sample", "rust_sample", "src/modules/samples/rust/src/lib.rs"),
                12u32,
                ::log::__private_api::Option::None,
            );
        }
    };
}
mod __dtor_____rust_dtor___mod {
    use super::__dtor__;
    #[cfg(not(any(target_os = "macos", target_os = "ios")))]
    #[inline(always)]
    unsafe fn do_atexit(cb: unsafe extern fn()) {
        extern "C" {
            fn atexit(cb: unsafe extern fn());
        }
        atexit(cb);
    }
    #[used]
    #[allow(non_upper_case_globals)]
    #[link_section = ".init_array"]
    static __dtor_export: unsafe extern "C" fn() = {
        #[cfg(not(any(target_os = "macos", target_os = "ios")))]
        #[link_section = ".text.exit"]
        unsafe extern "C" fn __dtor_____rust_dtor___dtor() {
            __dtor__()
        }
        #[link_section = ".text.startup"]
        unsafe extern fn __dtor_atexit() {
            do_atexit(__dtor_____rust_dtor___dtor);
        }
        __dtor_atexit
    };
}
#[no_mangle]
pub extern "C" fn MmiGetInfo(
    client: *const ::std::ffi::c_char,
    payload: *mut *mut ::std::ffi::c_char,
    payload_size: *mut ::std::ffi::c_int,
) -> ::std::ffi::c_int {
    if client.is_null() {
        {
            let lvl = ::log::Level::Error;
            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                ::log::__private_api::log(
                    format_args!("null client name"),
                    lvl,
                    &(
                        "rust_sample",
                        "rust_sample",
                        "src/modules/samples/rust/src/lib.rs",
                    ),
                    12u32,
                    ::log::__private_api::Option::None,
                );
            }
        };
        return ::libc::EINVAL;
    }
    if payload.is_null() {
        {
            let lvl = ::log::Level::Error;
            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                ::log::__private_api::log(
                    format_args!("null payload"),
                    lvl,
                    &(
                        "rust_sample",
                        "rust_sample",
                        "src/modules/samples/rust/src/lib.rs",
                    ),
                    12u32,
                    ::log::__private_api::Option::None,
                );
            }
        };
        return ::libc::EINVAL;
    }
    if payload_size.is_null() {
        {
            let lvl = ::log::Level::Error;
            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                ::log::__private_api::log(
                    format_args!("null payload size"),
                    lvl,
                    &(
                        "rust_sample",
                        "rust_sample",
                        "src/modules/samples/rust/src/lib.rs",
                    ),
                    12u32,
                    ::log::__private_api::Option::None,
                );
            }
        };
        return ::libc::EINVAL;
    }
    unsafe {
        *payload = ::std::ptr::null_mut();
        *payload_size = 0;
    }
    let meta = ::osc::module::Meta {
        name: "Sample".to_string(),
        description: Some("Sample module for...".to_string()),
        manufacturer: Some("Microsoft".to_string()),
        version: ::osc::module::Version {
            major: 1u32,
            minor: 0u32,
            patch: 0u32,
            tweak: 0u32,
        },
        components: <[_]>::into_vec(
            #[rustc_box]
            ::alloc::boxed::Box::new(["Sample".to_string()]),
        ),
        lifetime: ::osc::module::Lifetime::Long,
        user_account: ::osc::module::UserAccount::Root,
    };
    let json = ::serde_json::to_string(&meta).unwrap();
    let json = ::std::ffi::CString::new(json.as_str()).unwrap();
    let size = json.as_bytes().len() as ::std::ffi::c_int;
    unsafe {
        *payload = json.into_raw();
        *payload_size = size as ::std::ffi::c_int;
    };
    0
}
#[no_mangle]
pub extern "C" fn MmiOpen(
    client: *const ::std::ffi::c_char,
    max_size: ::std::ffi::c_uint,
) -> *mut ::std::ffi::c_void {
    if client.is_null() {
        {
            let lvl = ::log::Level::Error;
            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                ::log::__private_api::log(
                    format_args!("null client name"),
                    lvl,
                    &(
                        "rust_sample",
                        "rust_sample",
                        "src/modules/samples/rust/src/lib.rs",
                    ),
                    12u32,
                    ::log::__private_api::Option::None,
                );
            }
        };
        return std::ptr::null_mut();
    }
    let module = Box::new(Sample::default());
    Box::into_raw(module) as *mut ::std::ffi::c_void
}
#[no_mangle]
pub extern "C" fn MmiClose(handle: *mut ::std::ffi::c_void) {
    if !handle.is_null() {
        let _ = unsafe { Box::from_raw(handle as *mut Sample) };
    }
}
#[no_mangle]
pub extern "C" fn MmiSet(
    handle: *mut ::std::ffi::c_void,
    component: *const ::std::ffi::c_char,
    property: *const ::std::ffi::c_char,
    payload: *mut ::std::ffi::c_char,
    payload_size: ::std::ffi::c_int,
) -> ::std::ffi::c_int {
    if handle.is_null() {
        {
            let lvl = ::log::Level::Error;
            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                ::log::__private_api::log(
                    format_args!("null handle"),
                    lvl,
                    &(
                        "rust_sample",
                        "rust_sample",
                        "src/modules/samples/rust/src/lib.rs",
                    ),
                    12u32,
                    ::log::__private_api::Option::None,
                );
            }
        };
        return ::libc::EINVAL;
    }
    if component.is_null() {
        {
            let lvl = ::log::Level::Error;
            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                ::log::__private_api::log(
                    format_args!("null component"),
                    lvl,
                    &(
                        "rust_sample",
                        "rust_sample",
                        "src/modules/samples/rust/src/lib.rs",
                    ),
                    12u32,
                    ::log::__private_api::Option::None,
                );
            }
        };
        return ::libc::EINVAL;
    }
    if property.is_null() {
        {
            let lvl = ::log::Level::Error;
            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                ::log::__private_api::log(
                    format_args!("null property"),
                    lvl,
                    &(
                        "rust_sample",
                        "rust_sample",
                        "src/modules/samples/rust/src/lib.rs",
                    ),
                    12u32,
                    ::log::__private_api::Option::None,
                );
            }
        };
        return ::libc::EINVAL;
    }
    let component = unsafe { std::ffi::CStr::from_ptr(component) };
    let component = component.to_str().unwrap();
    if "Sample" != component {
        {
            let lvl = ::log::Level::Error;
            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                ::log::__private_api::log(
                    format_args!("invalid component: {0}", component),
                    lvl,
                    &(
                        "rust_sample",
                        "rust_sample",
                        "src/modules/samples/rust/src/lib.rs",
                    ),
                    12u32,
                    ::log::__private_api::Option::None,
                );
            }
        };
        return ::libc::EINVAL;
    }
    let property = unsafe { std::ffi::CStr::from_ptr(property) };
    let property = property.to_str().unwrap();
    let payload = unsafe {
        std::slice::from_raw_parts(payload as *const u8, payload_size as usize)
    };
    let payload = String::from_utf8_lossy(payload).to_string();
    let module: &mut Sample = unsafe { &mut *(handle as *mut Sample) };
    match property {
        "desired_simple" => {
            Sample::desired_simple(
                module,
                ::serde_json::from_str::<i32>(&payload).unwrap(),
            );
        }
        _ => {
            {
                let lvl = ::log::Level::Error;
                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                    ::log::__private_api::log(
                        format_args!("invalid property: {0}.{1}", component, property),
                        lvl,
                        &(
                            "rust_sample",
                            "rust_sample",
                            "src/modules/samples/rust/src/lib.rs",
                        ),
                        12u32,
                        ::log::__private_api::Option::None,
                    );
                }
            };
        }
    }
    ::libc::EXIT_SUCCESS
}
#[no_mangle]
pub extern "C" fn MmiGet(
    handle: *mut ::std::ffi::c_void,
    component: *const ::std::ffi::c_char,
    property: *const ::std::ffi::c_char,
    payload: *mut *mut ::std::ffi::c_char,
    payload_size: *mut ::std::ffi::c_int,
) -> ::std::ffi::c_int {
    if handle.is_null() {
        {
            let lvl = ::log::Level::Error;
            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                ::log::__private_api::log(
                    format_args!("null handle"),
                    lvl,
                    &(
                        "rust_sample",
                        "rust_sample",
                        "src/modules/samples/rust/src/lib.rs",
                    ),
                    12u32,
                    ::log::__private_api::Option::None,
                );
            }
        };
        return ::libc::EINVAL;
    }
    if component.is_null() {
        {
            let lvl = ::log::Level::Error;
            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                ::log::__private_api::log(
                    format_args!("null component"),
                    lvl,
                    &(
                        "rust_sample",
                        "rust_sample",
                        "src/modules/samples/rust/src/lib.rs",
                    ),
                    12u32,
                    ::log::__private_api::Option::None,
                );
            }
        };
        return ::libc::EINVAL;
    }
    if property.is_null() {
        {
            let lvl = ::log::Level::Error;
            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                ::log::__private_api::log(
                    format_args!("null property"),
                    lvl,
                    &(
                        "rust_sample",
                        "rust_sample",
                        "src/modules/samples/rust/src/lib.rs",
                    ),
                    12u32,
                    ::log::__private_api::Option::None,
                );
            }
        };
        return ::libc::EINVAL;
    }
    if payload.is_null() {
        {
            let lvl = ::log::Level::Error;
            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                ::log::__private_api::log(
                    format_args!("null payload"),
                    lvl,
                    &(
                        "rust_sample",
                        "rust_sample",
                        "src/modules/samples/rust/src/lib.rs",
                    ),
                    12u32,
                    ::log::__private_api::Option::None,
                );
            }
        };
        return ::libc::EINVAL;
    }
    if payload_size.is_null() {
        {
            let lvl = ::log::Level::Error;
            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                ::log::__private_api::log(
                    format_args!("null payload size"),
                    lvl,
                    &(
                        "rust_sample",
                        "rust_sample",
                        "src/modules/samples/rust/src/lib.rs",
                    ),
                    12u32,
                    ::log::__private_api::Option::None,
                );
            }
        };
        return ::libc::EINVAL;
    }
    unsafe {
        *payload = std::ptr::null_mut();
        *payload_size = 0;
    }
    let component = unsafe { ::std::ffi::CStr::from_ptr(component) };
    let component = component.to_str().unwrap();
    if "Sample" != component {
        {
            let lvl = ::log::Level::Error;
            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                ::log::__private_api::log(
                    format_args!("invalid component: {0}", component),
                    lvl,
                    &(
                        "rust_sample",
                        "rust_sample",
                        "src/modules/samples/rust/src/lib.rs",
                    ),
                    12u32,
                    ::log::__private_api::Option::None,
                );
            }
        };
        return ::libc::EINVAL;
    }
    let property = unsafe { ::std::ffi::CStr::from_ptr(property) };
    let property = property.to_str().unwrap();
    let module: &Sample = unsafe { &*(handle as *const Sample) };
    let res = match property {
        "simple" => {
            let res = ::osc::module::IntoPropertyResult::into_result(
                Sample::simple(&module),
            );
            ::osc::module::IntoResolvable::into_resolvable(res)
        }
        "complex_1" => {
            let res = ::osc::module::IntoPropertyResult::into_result(
                Sample::complex_1(),
            );
            ::osc::module::IntoResolvable::into_resolvable(res)
        }
        _ => {
            ::osc::module::PropertyResult::Err(
                ::osc::module::PropertyError::new({
                    let res = ::alloc::fmt::format(
                        format_args!("invalid property: {0}.{1}", component, property),
                    );
                    res
                }),
            )
        }
    };
    match res {
        Ok(val) => {
            let json = ::std::ffi::CString::new(val.to_string().unwrap()).unwrap();
            let size = json.as_bytes().len() as ::std::ffi::c_int;
            unsafe {
                *payload = json.into_raw();
                *payload_size = size as ::std::ffi::c_int;
            };
            ::libc::EXIT_SUCCESS
        }
        Err(err) => {
            {
                let lvl = ::log::Level::Error;
                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                    ::log::__private_api::log(
                        format_args!("{0}", err),
                        lvl,
                        &(
                            "rust_sample",
                            "rust_sample",
                            "src/modules/samples/rust/src/lib.rs",
                        ),
                        12u32,
                        ::log::__private_api::Option::None,
                    );
                }
            };
            -1
        }
    }
}
struct Foo {
    x: i32,
    y: String,
}
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for Foo {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = _serde::Serializer::serialize_struct(
                __serializer,
                "Foo",
                false as usize + 1 + 1,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "x",
                &self.x,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "y",
                &self.y,
            )?;
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
#[automatically_derived]
impl ::core::default::Default for Foo {
    #[inline]
    fn default() -> Foo {
        Foo {
            x: ::core::default::Default::default(),
            y: ::core::default::Default::default(),
        }
    }
}
enum CustomError {
    #[error("Something went wrong")]
    Something,
}
#[allow(unused_qualifications)]
impl std::error::Error for CustomError {}
#[allow(unused_qualifications)]
impl ::core::fmt::Display for CustomError {
    fn fmt(&self, __formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        #[allow(unused_variables, deprecated, clippy::used_underscore_binding)]
        match self {
            CustomError::Something {} => {
                __formatter.write_fmt(format_args!("Something went wrong"))
            }
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for CustomError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(f, "Something")
    }
}
