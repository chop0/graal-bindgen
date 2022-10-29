use core::intrinsics::transmute;
use core::marker::PhantomData;
use core::u64;
use std::ffi::CString;

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

trait __never_trait {
    type Output;
}

#[repr(C)]
pub struct Value {
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

pub fn expect_variadic<U: Passable, T: Pass<U>>(value: T) -> U {
    value.pass()
}

/// these macros were taken from https://github.com/ruestigraben/ruesti-base/blob/master/src/main/rust/polyglot.rs
#[macro_export]
macro_rules! new_instance {
    ($constructor: expr) => {{
        unsafe {
            $crate::polyglot::polyglot_new_instance(
                $constructor as *mut _
            ) as *mut _
        }
    }};
    ($constructor: expr, $($args: expr),*) => {{
        unsafe {
            $crate::polyglot::polyglot_new_instance(
                $constructor as *mut _,
                $($crate::polyglot::expect_variadic($args)),*
            )
        }
    }}
}

#[macro_export]
macro_rules! invoke_method {
    ($value: expr, $method: expr) => {{
        unsafe {
            $crate::polyglot::polyglot_invoke(
                $value,
                $crate::polyglot::make_cstr($method).as_ptr()
            )
        }
    }};
    ($value: expr, $method: expr, $($args: expr),+) => {{
        unsafe {
            $crate::polyglot::polyglot_invoke(
                $value,
                    $crate::polyglot::make_cstr($method).as_ptr(),
                $($crate::polyglot::expect_variadic($args)),*
            )
        }
    }}
}


pub fn make_cstr(name: &str) -> CString {
    CString::new(name).unwrap()
}

pub fn java_type(name: &str) -> *mut Constructor {
    let c_str = make_cstr(name);
    let value = unsafe { polyglot_java_type(c_str.as_ptr()) };
    if unsafe { polyglot_is_null(value) } || !unsafe { polyglot_can_instantiate(value) } {
        panic!("Not a type")
    }
    unsafe { transmute(value) }
}
