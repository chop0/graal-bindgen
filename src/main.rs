#![no_main]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

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
    let array_from_list = list.get(0).toArray();
    for i in 0..100 {
        println!("{}", array_from_list.get(i).unwrap());
    }

    // let mut vec = Vec::new();
    // let mut vec_in_vec = Vec::new();
    // for i in 0..100 {
    //     vec_in_vec.push(i); 
    // }
    // vec.push(vec_in_vec);
    // let slice_from_vec = vec.get(0).unwrap().as_slice();
    // for i in 0..100 {
    //     println!("{}", slice_from_vec.get(i).unwrap());
    // }
}
