use std::env;
use std::path::PathBuf;
fn main() {
    
    //println!("cargo:rerun-if-changed=build.rs"); kj
    println!("cargo:rustc-env=LD_LIBRARY_PATH=/home/alec/graalvm-ce-java11-21.0.0.2/languages/llvm/native/lib");


    let bindings = bindgen::Builder::default().header("src/header.h")
    .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    .clang_arg("-I/home/alec/graalvm-ce-java11-21.0.0.2/languages/llvm/include")
    .ctypes_prefix("crate::types::ctypes")
    .generate()
    .expect("Failed to generate graal bindings.");



        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");    
        
}
