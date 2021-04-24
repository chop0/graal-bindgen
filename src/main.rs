#![no_main]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]


use builtins::ArrayList;
use ruesti::{java_type, Value};
pub mod ruesti;
pub mod builtins;
pub mod types;

#[no_mangle]
pub extern "C" fn main() {
    let test: ArrayList<i32> = ArrayList::new();
    
    test.add(69);
    println!("{}", test.get(0));
}
