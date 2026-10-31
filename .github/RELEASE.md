# Release Process

## Automated Releases

GitHub Actions builds Windows, Linux, and macOS releases. Builds are not
code-signed.

### Creating a Release

1. **Update the version** in `Cargo.toml`. The Debian package version is read
   from it automatically.

2. **Commit and push changes:**

   ```bash
   git add .
   git commit -m "Bump version to X.Y.Z"
   git push origin main
   ```

3. **Create and push a version tag:**

   ```bash
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```

4. **GitHub Actions will automatically:**
   - Build `rnes-windows-x64.zip` (`rnes.exe` + `SDL3.dll`)
   - Build `rnes-linux-x64.tar.gz` and `rnes_X.Y.Z_amd64.deb` (SDL3 built in)
   - Build `rnes-macos-arm64.zip` (SDL3 built in)
   - Create a GitHub Release with all of them attached

### Manual Testing

To test the workflow without creating a release:

1. Push to `main`, open a pull request, or run the workflow manually from the
   Actions tab (**Run workflow**, any branch).
2. The workflow builds all platforms and uploads the packages as artifacts.
3. Download the artifacts from the run's summary page to test.

### Platform-Specific Notes

#### Windows

- The ZIP includes `rnes.exe` and `SDL3.dll`.
- The executable is unsigned, so SmartScreen may warn on first launch.

#### Linux

- SDL3 is compiled in and statically linked, so no SDL3 package is needed.
- The binary only links against glibc and libudev.
- Install the `.deb` with `sudo apt install ./rnes_X.Y.Z_amd64.deb`, or run
  `rnes` straight from the tarball.

#### macOS

- Built on Apple Silicon (arm64) with SDL3 statically linked.
- The binary is unsigned; users may need to allow it in System Settings >
  Privacy & Security, or run `xattr -d com.apple.quarantine rnes`.

## Troubleshooting

**Formatting or clippy failures:** run `cargo fmt` and
`cargo clippy -- -D warnings` locally before pushing. CI also builds with
`RUSTFLAGS=-D warnings`, so deprecation warnings fail the build.

**vcpkg/SDL3 issues on Windows:** check the "Install SDL3" step and
`vcpkg.json`, then clear the Actions cache and re-run.

**SDL3 build failures on Linux:** SDL3 is compiled with CMake; make sure the
packages in `LINUX_DEPS` in `.github/workflows/main.yml` are installed.
