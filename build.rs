//! This build script copies the `memory.x` file from the crate root into
//! a directory where the linker can always find it at build time.
//! For many projects this is optional, as the linker always searches the
//! project root directory -- wherever `Cargo.toml` is. However, if you
//! are using a workspace or have a more complicated build setup, this
//! build script becomes required. Additionally, by requesting that
//! Cargo re-run the build script whenever `memory.x` is changed,
//! updating `memory.x` ensures a rebuild of the application with the
//! new memory settings.

use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");

    if cfg!(feature = "rp-pico") {
        println!("cargo:rerun-if-changed=rp_memory.x");

        //println!("cargo:rustc-link-arg-bins=-Tlink-rp.x");

        let out_dir = &PathBuf::from(env::var("OUT_DIR").unwrap());
        println!("out dir: {out_dir:?}");
        std::fs::copy("./rp_memory.x", out_dir.join("memory.x")).unwrap();
        println!("cargo:rustc-link-search={}", out_dir.display());
    }
}
