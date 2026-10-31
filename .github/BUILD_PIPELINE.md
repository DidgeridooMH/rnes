# Build Pipeline Documentation

This document describes the CI/CD pipeline in `.github/workflows/main.yml`.

## Triggers

- Push to `main`
- Pull requests targeting `main`
- Tags matching `v*` (also creates a GitHub Release)
- Manual runs (**Actions > Build and Release > Run workflow**, on any branch)

## Jobs

### `test` (ubuntu-latest)

Runs first; every build job depends on it.

- `cargo fmt -- --check`
- `cargo clippy -- -D warnings`
- `cargo test`

`actions-rust-lang/setup-rust-toolchain` also sets `RUSTFLAGS=-D warnings`, so
compiler warnings (including deprecations) fail every job.

### `build-windows` (windows-latest)

1. Install SDL3 with vcpkg (manifest in `vcpkg.json`).
2. `cargo build --release`. `build.rs` points the linker at
   `vcpkg_installed/x64-windows` and copies `SDL3.dll` next to the executable.
3. Package `rnes.exe` and `SDL3.dll` into `rnes-windows-x64.zip`.

### `build-linux` (ubuntu-latest)

1. Install the build dependencies listed in `LINUX_DEPS` (CMake plus the
   headers SDL3 needs; audio and windowing libraries are loaded at runtime).
2. `cargo build --release`. On Linux the `sdl3` crate uses the
   `build-from-source-static` feature (see `Cargo.toml`), so SDL3 is compiled
   from source and linked into the binary.
3. Print `ldd` output so the runtime dependencies are visible in the log
   (glibc and libudev only).
4. Package `rnes-linux-x64.tar.gz` and `rnes_<version>_amd64.deb`. The .deb
   version comes from `Cargo.toml`; it depends on `libc6` and `libudev1`.

### `build-macos` (macos-latest, arm64)

1. `cargo build --release`, with SDL3 built from source and statically linked
   (same `Cargo.toml` setting as Linux).
2. Print `otool -L` output.
3. Package `rnes-macos-arm64.zip`.

### `release` (tags only)

Downloads the artifacts from the three build jobs and attaches them to a
GitHub Release with `softprops/action-gh-release`. The job needs
`contents: write` permission, which the workflow grants.

## Artifacts

| Platform | File | SDL3 |
| -------- | ---- | ---- |
| Windows x64 | `rnes-windows-x64.zip` | `SDL3.dll` included |
| Linux x64 | `rnes-linux-x64.tar.gz`, `rnes_<version>_amd64.deb` | statically linked |
| macOS arm64 | `rnes-macos-arm64.zip` | statically linked |

None of the builds are code-signed.

## Running the checks locally

```bash
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
cargo build --release
```

On Windows, run `vcpkg install` in the project root first. On Linux, install
the packages in `LINUX_DEPS` so SDL3 can be built.
