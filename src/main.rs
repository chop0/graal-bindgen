#![no_main]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::ffi::CString;

use ruesti::{java_type, Receive, Value};
pub mod builtins;
pub mod ruesti;
pub mod types;

use builtins::*;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[no_mangle]
pub extern "C" fn main() {
    let list: ArrayList<i32> = ArrayList::new();
    for i in 0..100 {
        list.add(i);
    }
    assert_eq!(list.size(), 100);

    for i in 0..100 {
        list.remove_at(0);
    }
    assert_eq!(list.size(), 0);
}
