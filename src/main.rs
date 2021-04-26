#![no_main]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

pub mod builtins;
pub mod polyglot;
pub mod types;

use builtins::*;
#[no_mangle]
pub extern "C" fn main() {
    let list = ArrayList::new();
    let list_in_list = ArrayList::new();
    for i in 0..100 {
        list_in_list.add(i);
    }

    list.add(list_in_list);
    let array_from_list = list.toArray().get(0).unwrap().toArray();
    let list2 = ArrayList::new();
    list2.add(array_from_list);
    for i in 0..100 {
        println!("{}", list2.get(0).get(i).unwrap());
    }
}
