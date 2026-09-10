fn main() {
    let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let dir = std::env::var("TDLIB_DIR").unwrap_or(format!("{root}/tdlib-install"));
    println!("cargo:rustc-link-search=native={dir}/lib");
    println!("cargo:rustc-link-lib=dylib=tdjson");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{dir}/lib");
    println!("cargo:rerun-if-env-changed=TDLIB_DIR");
}
