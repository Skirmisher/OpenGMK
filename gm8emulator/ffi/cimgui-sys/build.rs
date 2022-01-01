use std::{fs, env};

fn main() -> Result<(), Box<dyn ::std::error::Error>> {
    println!("cargo:rerun-if-changed=bindings.c");
    let dest = cmake::Config::new("../cimgui").define("IMGUI_STATIC", "yes").build();
    if env::var("CARGO_CFG_TARGET_ENV").unwrap() != "msvc" {
        // cimgui's CMakeLists removes the `lib` prefix for no apparent reason, and we can't tell Rust the exact name,
        // so just rename it to be correct instead
        fs::rename(dest.join("cimgui.a"), dest.join("libcimgui.a")).unwrap();
    }
    println!("cargo:rustc-link-search=native={}", dest.display());
    println!("cargo:rustc-link-lib=static=cimgui");
    Ok(())
}
