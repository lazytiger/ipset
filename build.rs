fn main() {
    cc::Build::new().file("wrapper.c").compile("aux");
    println!("cargo:rustc-link-lib=ipset");
    println!("cargo:rerun-if-changed=wrapper.c");
}
