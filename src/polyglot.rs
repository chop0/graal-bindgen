/// A good chunk of this file was taken from https://github.com/ruestigraben/ruesti-base/blob/master/src/main/rust/polyglot.rs
use core::intrinsics::transmute;
use core::u64;
use std::{ffi::CString, marker::PhantomData};
use std::{os::raw::c_char};

use internal::make_cstring;

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bindings.rs"));

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
                unsafe { $graalfn(value) }
            }
        }
    };
}

macro_rules! pass_and_passable {
    ($typename: ty) => {
        unsafe impl Passable for $typename {}
        unsafe impl Pass<$typename> for $typename {
            fn pass(&self) -> Self {
                *self
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
pub unsafe trait Pass<T>
where
    T: Passable,
{
    fn pass(&self) -> T;
}

/// A value that can be passed to Graal Polyglot.  This is either a number or a pointer to a polyglot value
pub unsafe trait Passable {}

pass_and_passable!(*const Value);
pass_and_passable!(*mut Value);
pass_and_passable!(i8);
pass_and_passable!(i16);
pass_and_passable!(i32);
pass_and_passable!(i64);

#[derive(Clone, Copy)]
pub struct JavaArray<T, U>
where
    T: Pass<U> + Receive,
    U: Passable,
{
    ptr: *mut Value,
    phantom: PhantomData<T>,
    phantom2: PhantomData<U>,
}

unsafe impl<T, U> Pass<*mut Value> for JavaArray<T, U>
where
    T: Pass<U> + Receive,
    U: Passable,
{
    fn pass(&self) -> *mut Value {
        self.ptr
    }
}
unsafe impl<T, U> Receive for JavaArray<T, U>
where
    T: Pass<U> + Receive,
    U: Passable,
{
    fn from_polyglot_value(value: *mut Value) -> Self {
        Self {
            ptr: value,
            phantom: PhantomData,
            phantom2: PhantomData,
        }
    }
}

impl<T, U> JavaArray<T, U>
where
    T: Pass<U> + Receive,
    U: Passable,
{
    pub fn get(&self, index: u64) -> Option<T> {
        unsafe {
            if index >= polyglot_get_array_size(self.ptr) {
                None
            } else {
                Some(T::from_polyglot_value(polyglot_get_array_element(
                    self.ptr,
                    index as i32,
                )))
            }
        }
    }
}

pub mod internal {
    extern "C" {
        pub fn polyglot_invoke(
            value: *mut super::Value,
            name: *const super::c_char,
            ...
        ) -> *mut super::Value;
        pub fn polyglot_new_instance(value: *const super::Constructor, ...) -> *mut super::Value;
    }

    pub fn expect_variadic<U: super::Passable, T: super::Pass<U>>(value: T) -> U {
        value.pass()
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
            $crate::polyglot::internal::polyglot_new_instance(
                $constructor
            )
        }
    }};
    ($constructor: expr, $($args: expr),*) => {{
        unsafe {
            $crate::polyglot::internal::polyglot_new_instance(
                $constructor,
                $($crate::polyglot::internal::expect_variadic($args)),*
            )
        }
    }}
}

#[macro_export]
macro_rules! invoke_method {
    ($value: expr, $method: expr) => {{
        unsafe {
            $crate::polyglot::internal::polyglot_invoke(
                $value,
                $crate::polyglot::internal::make_cstring($method).as_ptr()
            )
        }
    }};
    ($value: expr, $method: expr, $($args: expr),+) => {{
        unsafe {
            $crate::polyglot::internal::polyglot_invoke(
                $value,
                $crate::polyglot::internal::make_cstring($method).as_ptr(),
                $($crate::polyglot::internal::expect_variadic($args)),*
            )
        }
    }}
}

#[macro_export]
macro_rules! execute {
    ($executable: expr) => {{
        let fnptr = $crate::polyglot::internal::transmute_executable_nullary($executable);
        fnptr()
    }};
    ($executable: expr, $($args: expr),+) => {{
        let fnptr = $crate::polyglot::internal::transmute_executable($executable);
        fnptr($($crate::polyglot::internal::expect_variadic($args)),+)
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

pub fn as_string(value: *mut Value) -> String {
    unsafe {
        let len = get_string_size(value);
        let buf = CString::new(vec![0; len as usize]).unwrap();
        let charset = make_cstring("UTF-8");
        polyglot_as_string(value, buf.as_ptr() as *mut i8, len, charset.as_ptr());
        buf.into_string().unwrap()
    }
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
