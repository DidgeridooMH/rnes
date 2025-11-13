use std::env;
use std::path::PathBuf;

fn main() {
    // Get the project root directory
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let target = env::var("TARGET").unwrap();

    // On Linux, use system SDL3 (installed via package manager)
    if target.contains("darwin") || target.contains("linux") {
        // On Linux in CI, pkg-config will handle SDL3 linking automatically
        // The sdl3 crate should handle this via its build script
        return;
    }

    // For Windows and macOS, use vcpkg
    let vcpkg_root = PathBuf::from(&manifest_dir).join("vcpkg_installed");

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
    if lib_dir.exists() {
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
    }

    if bin_dir.exists() {
        // Also add the bin directory to the PATH for finding DLLs at runtime
        println!("cargo:rustc-link-search=native={}", bin_dir.display());
    }

    // Set include path for any C/C++ code that might need it
    println!("cargo:include={}", include_dir.display());

    // Tell cargo to re-run this script if vcpkg directories change
    println!("cargo:rerun-if-changed={}", lib_dir.display());
    println!("cargo:rerun-if-changed={}", bin_dir.display());
}
