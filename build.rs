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

        // Copy SDL3.dll next to the executable so it can be found at runtime.
        // OUT_DIR is <target>/<profile>/build/<pkg>-<hash>/out, so the
        // profile directory (where the exe lands) is three levels up.
        let dll = bin_dir.join("SDL3.dll");
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        if let Some(profile_dir) = out_dir.ancestors().nth(3) {
            if dll.exists() {
                let dest = profile_dir.join("SDL3.dll");
                if let Err(e) = std::fs::copy(&dll, &dest) {
                    println!("cargo:warning=Failed to copy SDL3.dll to {}: {}", dest.display(), e);
                }
            } else {
                println!(
                    "cargo:warning=SDL3.dll not found at {}. Run `vcpkg install` to fetch SDL3.",
                    dll.display()
                );
            }
        }
    } else {
        println!(
            "cargo:warning=vcpkg SDL3 not found at {}. Run `vcpkg install` in the project root.",
            bin_dir.display()
        );
    }

    // Set include path for any C/C++ code that might need it
    println!("cargo:include={}", include_dir.display());

    // Tell cargo to re-run this script if vcpkg directories change
    println!("cargo:rerun-if-changed={}", lib_dir.display());
    println!("cargo:rerun-if-changed={}", bin_dir.display());
}
