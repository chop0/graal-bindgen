use std::env;
use std::path::PathBuf;
fn main() {
    let graal_home = env!(
        "GRAAL_HOME",
        "Environment variable GRAAL_HOME is not set.  Set it to GraalVM's root directory."
    );

    println!("cargo:rerun-if-changed=src/header.h");
    println!("cargo:rerun-if-changed={}", graal_home);
    println!(
        "cargo:rustc-env=LD_LIBRARY_PATH={}/languages/llvm/native/lib",
        graal_home
    );

    let bindings = bindgen::Builder::default()
        .header("src/header.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .clang_arg(format!("-I{}/languages/llvm/include", graal_home))
        .ctypes_prefix("crate::types::ctypes")
        .generate()
        .expect("Failed to generate graal bindings.");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
