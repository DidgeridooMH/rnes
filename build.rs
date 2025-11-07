use std::env;
use std::path::PathBuf;

fn main() {
    // Get the project root directory
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let vcpkg_root = PathBuf::from(&manifest_dir).join("vcpkg_installed");

    // Determine the target architecture
    let target = env::var("TARGET").unwrap();

    let vcpkg_target = if target.contains("x86_64") && target.contains("windows") {
        "x64-windows"
    } else if target.contains("i686") && target.contains("windows") {
        "x86-windows"
    } else if target.contains("aarch64") && target.contains("windows") {
        "arm64-windows"
    } else {
        panic!("Unsupported target: {}", target);
    };

    let lib_dir = vcpkg_root.join(vcpkg_target).join("lib");
    let bin_dir = vcpkg_root.join(vcpkg_target).join("bin");
    let include_dir = vcpkg_root.join(vcpkg_target).join("include");

    // Check if the directories exist
    if !lib_dir.exists() {
        panic!(
            "vcpkg SDL3 lib directory not found: {:?}\nRun 'vcpkg install sdl3' first",
            lib_dir
        );
    }

    // Tell cargo to link the SDL3 library
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=SDL3");

    // Also add the bin directory to the PATH for finding DLLs at runtime
    println!("cargo:rustc-link-search=native={}", bin_dir.display());

    // Set include path for any C/C++ code that might need it
    println!("cargo:include={}", include_dir.display());

    // Tell cargo to re-run this script if vcpkg directories change
    println!("cargo:rerun-if-changed={}", lib_dir.display());
    println!("cargo:rerun-if-changed={}", bin_dir.display());

    println!("cargo:warning=Using SDL3 from vcpkg: {}", lib_dir.display());
}
