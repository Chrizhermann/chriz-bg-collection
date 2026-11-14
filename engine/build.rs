fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    // `unrar_sys` compiles C++ sources with `.cpp_link_stdlib(None)`. On GNU/Linux
    // targets ensure `libstdc++` is linked so `g++` C++ runtime symbols resolve.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("gnu")
    {
        println!("cargo:rustc-link-lib=stdc++");
    }
}
