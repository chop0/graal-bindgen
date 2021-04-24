
use core::intrinsics::transmute;
use core::u64;
use std::ffi::CString;
use std::os::raw::c_char;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

macro_rules! primitive_receive {
    ($typename: ident, $graalfn:ident, $assertfn:ident) => {
        unsafe impl Receive for $typename {
            fn from_polyglot_value(value: *mut Value) -> Self {
                unsafe {
                    debug_assert!($assertfn(value));
                    $graalfn(value)
                }
            }
        }
    };

    ($typename: ident, $graalfn: ident) => {
        impl Receive for $typename {
            fn from_polyglot_value(value: *mut Value) -> Self {
                unsafe {
                    $graalfn(value)
                }
            }
        }
    };
}
#[repr(C)]
pub struct Value {
    _private: [u8; 0],
}

#[repr(C)]
pub struct Executable {
    _private: [u8; 0],
}

#[repr(C)]
pub struct Constructor {
    _private: [u8; 0],
}

/// Receive indicates that this type can be received by Rust from the GraalVM Runtime.  
pub unsafe trait Receive {
    fn from_polyglot_value(value: *mut Value) -> Self;
}

primitive_receive!(i8, polyglot_as_i8, polyglot_fits_in_i8);
primitive_receive!(i16, polyglot_as_i16, polyglot_fits_in_i16);
primitive_receive!(i32, polyglot_as_i32, polyglot_fits_in_i32);
primitive_receive!(i64, polyglot_as_i64, polyglot_fits_in_i64);
primitive_receive!(f32, polyglot_as_float, polyglot_fits_in_float);
primitive_receive!(f64, polyglot_as_double, polyglot_fits_in_double);
primitive_receive!(bool, polyglot_as_boolean, polyglot_is_boolean);


/// Pass is a marker trait that indicates a type can safely be passed to the GraalVM Runtime.
pub unsafe trait Pass {}

unsafe impl Pass for *const super::Value {}
unsafe impl Pass for *mut super::Value {}
unsafe impl Pass for i8 {}
unsafe impl Pass for i16 {}
unsafe impl Pass for i32 {}
unsafe impl Pass for i64 {}

pub mod internal {
    extern "C" {
        pub fn polyglot_invoke(
            value: *mut super::Value,
            name: *const super::c_char,
            ...
        ) -> *mut super::Value;
        pub fn polyglot_new_instance(value: *const super::Constructor, ...) -> *mut super::Value;
    }

    pub fn expect_variadic<T: super::Pass>(value: T) -> T {
        
        value
    }

    pub fn make_cstring(string: &str) -> super::CString {
        super::CString::new(string).expect("Could not convert to CString")
    }

    pub fn transmute_executable(
        executable: *const super::Executable,
    ) -> extern "C" fn(*const super::Value, ...) -> *mut super::Value {
        unsafe { super::transmute(executable) }
    }

    pub fn transmute_executable_nullary(
        executable: *const super::Executable,
    ) -> extern "C" fn() -> *mut super::Value {
        unsafe { super::transmute(executable) }
    }
}

#[macro_export]
macro_rules! new_instance {
    ($constructor: expr) => {{
        unsafe {
            $crate::ruesti::internal::polyglot_new_instance(
                $constructor
            )
        }
    }};
    ($constructor: expr, $($args: expr),*) => {{
        unsafe {
            $crate::ruesti::internal::polyglot_new_instance(
                $constructor,
                $($crate::ruesti::internal::expect_variadic($args)),*
            )
        }
    }}
}

#[macro_export]
macro_rules! invoke_method {
    ($value: expr, $method: expr) => {{
        unsafe {
            $crate::ruesti::internal::polyglot_invoke(
                $value,
                $crate::ruesti::internal::make_cstring($method).as_ptr()
            )
        }
    }};
    ($value: expr, $method: expr, $($args: expr),+) => {{
        unsafe {
            $crate::ruesti::internal::polyglot_invoke(
                $value,
                $crate::ruesti::internal::make_cstring($method).as_ptr(),
                $($crate::ruesti::internal::expect_variadic($args)),*
            )
        }
    }}
}

#[macro_export]
macro_rules! execute {
    ($executable: expr) => {{
        let fnptr = $crate::ruesti::internal::transmute_executable_nullary($executable);
        fnptr()
    }};
    ($executable: expr, $($args: expr),+) => {{
        let fnptr = $crate::ruesti::internal::transmute_executable($executable);
        fnptr($($crate::ruesti::internal::expect_variadic($args)),+)
    }};
}

pub fn from_string(str: &str) -> *mut Value {
    if str.len() > u64::MAX as usize {
        panic!("String is too long");
    }
    let ptr = str.as_ptr();
    let charset = "UTF-8\0".as_ptr();
    let len: u64 = str.len() as u64;
    unsafe { polyglot_from_string_n(ptr as *const i8, len, charset as *const i8) }
}

pub fn is_string(value: *const Value) -> bool {
    unsafe { polyglot_is_string(value) }
}

pub fn get_string_size(value: *const Value) -> u64 {
    if !is_string(value) {
        panic!("Not a string")
    };
    unsafe { polyglot_get_string_size(value) }
}

pub fn is_null(value: *const Value) -> bool {
    unsafe { polyglot_is_null(value) }
}

pub fn import(name: &str) -> *mut Value {
    let c_str = internal::make_cstring(name);
    let value = unsafe { polyglot_import(c_str.as_ptr()) };
    if is_null(value) {
        panic!("Import failed")
    }
    value
}

pub fn export(name: &str, value: *mut Value) {
    let c_str = internal::make_cstring(name);
    unsafe { polyglot_export(c_str.as_ptr(), value) }
}

pub fn as_executable(value: *const Value) -> *const Executable {
    unsafe {
        if !polyglot_can_execute(value) {
            panic!("Value is not executable")
        }
        transmute(value)
    }
}

pub fn java_type(name: &str) -> *mut Constructor {
    let c_str = internal::make_cstring(name);
    let value = unsafe { polyglot_java_type(c_str.as_ptr()) };
    if is_null(value) || !unsafe { polyglot_can_instantiate(value) } {
        panic!("Not a type")
    }
    unsafe { transmute(value) }
}

pub fn as_boolean(value: *const Value) -> bool {
    unsafe {
        if !polyglot_is_boolean(value) {
            panic!("Not a boolean")
        }
        polyglot_as_boolean(value)
    }
}
