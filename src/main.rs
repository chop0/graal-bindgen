#![no_main]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::ffi::CString;

pub mod builtins;
pub mod polyglot;
pub mod types;
use polyglot::{java_type, Receive, Value};
 

use builtins::*;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[no_mangle]
pub extern "C" fn main() {
    let list = ArrayList::new();
    for i in 0..100 {
        list.add(i);
    }
    assert_eq!(list.size(), 100);
    
    for i in 0..100 {
        list.remove_at(0);
    }
    assert_eq!(list.size(), 0);
}
