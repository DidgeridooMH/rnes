# Release Process

## Automated Releases

The project is configured with GitHub Actions to automatically build releases for Windows, macOS, and Linux.

### Creating a Release

1. **Update version numbers:**

   - Update `Cargo.toml` version
   - Update version in `.github/workflows/main.yml` (Debian package version)

2. **Commit and push changes:**

   ```bash
   git add .
   git commit -m "Bump version to X.Y.Z"
   git push origin main
   ```

3. **Create and push a version tag:**

   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

4. **GitHub Actions will automatically:**
   - Build for Windows (x64) with SDL3.dll included in ZIP
   - Build for macOS (x64) with SDL3 dylibs included in ZIP
   - Build Debian package for Linux (amd64) with SDL3 dependency
   - Create a GitHub Release with all artifacts attached

### Manual Testing

To test the workflow without creating a release:

1. Push to `main` branch or create a pull request
2. The workflow will build all platforms and upload artifacts
3. Download artifacts from the Actions tab to test

### Platform-Specific Notes

#### Windows

- ZIP file includes `rnes.exe` and `SDL3.dll`
- No additional installation required

#### macOS

- ZIP file includes `rnes` binary and SDL3 `.dylib` files
- Users may need to allow the app in System Preferences (Security)

#### Linux

- Debian package lists `libsdl3-0 | libsdl3-dev` as dependency
- Users must install SDL3 separately as it's not yet in standard repositories
- Post-installation script warns if SDL3 is not found

## Troubleshooting

### Build Failures

**vcpkg cache issues:**

- Clear the cache in GitHub Actions settings
- Re-run the workflow

**SDL3 not found:**

- Verify vcpkg.json is correct
- Check that vcpkg installation step completed successfully

**Rust compilation errors:**

- Ensure all platform-specific code is properly conditionally compiled
- Check that dependencies are up to date in Cargo.toml
