# Build Pipeline Documentation

This document describes the CI/CD pipeline for the RNES project.

## Overview

The project uses GitHub Actions to automatically build release artifacts for three platforms:

- Windows (x64)
- macOS (x64)
- Linux (amd64 Debian package)

## Pipeline Structure

### 1. Test Job

Runs first on every push and pull request:

- Code formatting check (`cargo fmt`)
- Linting with Clippy (`cargo clippy`)
- Unit tests (`cargo test --lib`)

All build jobs depend on this passing.

### 2. Platform Build Jobs

#### Windows Build (`build-windows`)

- **Runner**: `windows-latest`
- **Dependencies**: vcpkg with SDL3
- **Caching**: vcpkg installation and packages
- **Output**: `rnes-windows-x64.zip`
  - Contains: `rnes.exe` and `SDL3.dll`

**Process**:

1. Install/cache vcpkg
2. Install SDL3 via vcpkg (`sdl3:x64-windows`)
3. Build Rust project with cargo
4. Package binary and SDL3.dll into ZIP

#### macOS Build (`build-macos`)

- **Runner**: `macos-latest`
- **Dependencies**: vcpkg with SDL3
- **Caching**: vcpkg installation and packages
- **Output**: `rnes-macos-x64.zip`
  - Contains: `rnes` binary and SDL3 `.dylib` files

**Process**:

1. Install/cache vcpkg
2. Install SDL3 via vcpkg
3. Build Rust project with cargo
4. Find and package binary with all .dylib files

#### Linux Build (`build-linux`)

- **Runner**: `ubuntu-latest`
- **Dependencies**: System build tools
- **Output**: `rnes_0.1.0_amd64.deb`
  - Debian package with SDL3 dependency

**Process**:

1. Install build dependencies (no SDL3 build-time dependency)
2. Build Rust project with cargo
3. Create Debian package structure
4. Generate control file with SDL3 dependency
5. Create postinst script to warn if SDL3 missing
6. Build .deb package with dpkg-deb

### 3. Release Job (`release`)

- **Trigger**: Only runs when a tag starting with `v` is pushed
- **Dependencies**: Requires all three build jobs to complete
- **Process**:
  1. Download all artifacts
  2. Create GitHub Release
  3. Attach all three platform packages

## SDL3 Handling

### Windows & macOS

- SDL3 is built and included via vcpkg
- Dynamic libraries are bundled with the executable
- Users can run the application immediately after extraction
- No additional installation required

### Linux

- SDL3 is **not** bundled in the package
- Debian package declares dependency: `libsdl3-0 | libsdl3-dev`
- Post-installation script checks for SDL3 and warns if missing
- Users must install SDL3 separately (from source or PPA)

**Rationale**:

- Linux follows the package manager convention
- SDL3 is not yet in standard repositories
- Avoids library version conflicts
- Allows system-wide SDL3 updates

## Triggering Builds

### Automatic Triggers

1. **Push to main branch**

   - Runs tests and builds all platforms
   - Creates artifacts (available for 90 days)
   - Does NOT create a release

2. **Pull Request to main**

   - Runs tests and builds all platforms
   - Validates changes work on all platforms

3. **Tag Push (v\*)**
   - Runs full pipeline
   - Creates GitHub Release
   - Attaches all platform packages

### Manual Trigger

- Use "Run workflow" button in Actions tab
- Useful for testing or rebuilding without code changes

## Creating a Release

1. **Update Version**:

   ```bash
   # Update Cargo.toml
   version = "0.2.0"

   # Update workflow (debian package version)
   # In .github/workflows/main.yml line ~165
   Version: 0.2.0
   ```

2. **Commit Changes**:

   ```bash
   git add Cargo.toml .github/workflows/main.yml
   git commit -m "Release v0.2.0"
   git push origin main
   ```

3. **Create and Push Tag**:

   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

4. **Monitor Actions**:

   - Go to repository → Actions tab
   - Watch the workflow run
   - Check for any failures

5. **Verify Release**:
   - Go to repository → Releases
   - New release should appear with three attachments

## Local Testing

### Windows

```powershell
.\scripts\build-release.ps1
```

### Linux/macOS

```bash
chmod +x scripts/build-release.sh
./scripts/build-release.sh
```

## Troubleshooting

### vcpkg Cache Issues

**Problem**: Stale vcpkg cache causing build failures

**Solution**:

1. Go to Actions tab
2. Select "Caches" in left sidebar
3. Delete relevant caches
4. Re-run workflow

### SDL3 Not Found (Windows/macOS)

**Problem**: vcpkg installation fails or SDL3 not found

**Solution**:

- Check vcpkg.json is correct
- Verify vcpkg install command uses correct triplet
- Clear cache and retry

### Debian Package Won't Install

**Problem**: Dependency issues on Debian/Ubuntu

**Solution**:

- SDL3 must be installed separately
- Check post-install script output
- May need to install from source:
  ```bash
  git clone https://github.com/libsdl-org/SDL
  cd SDL
  mkdir build && cd build
  cmake .. -DCMAKE_BUILD_TYPE=Release
  sudo make install
  ```

### Build Failing on One Platform

**Problem**: Code works locally but fails in CI

**Solution**:

- Check platform-specific code
- Verify dependencies in Cargo.toml
- Test with exact same Rust version as CI
- Check build.rs for platform detection issues

## Customization

### Adding a New Platform

1. Add new job in `.github/workflows/main.yml`
2. Update `build.rs` for target detection
3. Add to release job dependencies
4. Update release file list

### Changing Package Format

- Windows: Modify ZIP creation in workflow
- macOS: Change dylib collection or use .app bundle
- Linux: Switch from .deb to .rpm or AppImage

### Adding More Tests

Edit the `test` job in workflow:

- Add more cargo commands
- Add integration tests
- Add benchmark runs
- Add coverage reporting

## Performance Considerations

### Caching Strategy

- vcpkg installation: ~5-10 minutes saved per build
- Cargo dependencies: Automatic via Rust toolchain
- Total time with cache: ~5-10 minutes per platform

### Parallel Execution

- All three platform builds run in parallel
- Test job runs first (fastest)
- Release only runs after all builds complete

### Artifact Storage

- Artifacts retained for 90 days by default
- Can be adjusted in workflow `retention-days`
- Releases are permanent until manually deleted
