use std::env;

fn main() {
    println!("cargo:rustc-link-lib=ipset");
    println!("cargo:rerun-if-changed=wrapper.c");
    println!("cargo:rerun-if-changed=wrapper.h");

    cc::Build::new().file("wrapper.c").compile("aux");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .use_core()
        .ctypes_prefix("libc")
        .generate()
        .expect("Unable to generate bindings");
    let mut out_file = env::var("OUT_DIR").unwrap();
    out_file.push_str("/binding.rs");
    bindings
        .write_to_file(out_file)
        .expect("Unable to write binding.rs");
}
