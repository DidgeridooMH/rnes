# RNES - NES Emulator

RNES is a high-performance NES emulator written in Rust, designed to faithfully replicate the classic Nintendo Entertainment System experience.

## Features

- Accurate Emulation: Faithfully emulates the NES hardware for a true retro gaming experience.
- Cross-Platform: Runs seamlessly on both Windows and Linux.
- High Performance: Leverages Rust's performance and safety features for fast and reliable emulation.
- User-Friendly: Simple command-line interface for easy use.

## Installation

### Pre-built Binaries

Download the latest release for your platform from the [Releases](https://github.com/DidgeridooMH/rnes/releases) page:

- **Windows**: `rnes-windows-x64.zip` (includes SDL3.dll)
- **macOS**: `rnes-macos-x64.zip` (includes SDL3 libraries)
- **Linux**: `rnes_0.1.0_amd64.deb` (requires SDL3 from package manager)

### Building from Source

#### Prerequisites

- [Rust](https://rust-lang.org) (latest stable version)
- Platform-specific dependencies:

**Windows:**

```powershell
# Install vcpkg
git clone https://github.com/Microsoft/vcpkg.git
.\vcpkg\bootstrap-vcpkg.bat
.\vcpkg\vcpkg.exe install sdl3:x64-windows
```

**macOS:**

```bash
# Install vcpkg
git clone https://github.com/Microsoft/vcpkg.git
./vcpkg/bootstrap-vcpkg.sh
./vcpkg/vcpkg install sdl3
```

**Linux:**

```bash
# Install SDL3 development libraries
# Note: SDL3 might need to be built from source or installed via a PPA
sudo apt-get update
sudo apt-get install build-essential pkg-config libssl-dev

# For now, SDL3 may need to be installed manually from:
# https://github.com/libsdl-org/SDL/releases
```

#### Build Steps

```bash
git clone https://github.com/DidgeridooMH/rnes.git
cd rnes
cargo build --release
```

The compiled binary will be in `target/release/rnes` (or `rnes.exe` on Windows).

## Usage

Run RNES with no arguments to open an empty window, then load a game with
**File > Open ROM** (or `Ctrl+O`, `Cmd+O` on macOS). **File > Close ROM**
stops the current game. Opening a new ROM while one is running replaces it.

You can also load a ROM directly at startup:

```bash
./rnes --rom <ROM_FILE>
```

The menu bar is available on Windows and macOS; on Linux use `Ctrl+O`.

### Command-line Options

```bash
rnes                      # Open with no ROM loaded
rnes --rom <ROM_FILE>     # Run the emulator with specified ROM
rnes --sav <SAV_FILE>     # Preload battery-backed PRG-RAM from a save file
```

### Headless Mode

For debugging and automated testing, `--headless` runs a ROM without a window
and writes PNG frames:

```bash
rnes --rom game.nes --headless --frames 600 \
     --input "120:S,126:" --dump 300-600/30 --sheet --out headless_out
```

- `--input` scripted presses as `frame:buttons` (A B s=Select S=Start U D L R; empty releases)
- `--dump` frames to save (`100`, `200-300`, `200-300/10`); `--sheet` also tiles them into `sheet.png`
- `--trace A-B` logs PPU register writes and executed instructions for those frames
- `--watch 0010,0011` prints RAM addresses every frame

## Continuous Integration

This project uses GitHub Actions for automated building and testing across multiple platforms:

- **Windows**: Builds with vcpkg-installed SDL3, creates ZIP with DLL included
- **macOS**: Builds with vcpkg-installed SDL3, creates ZIP with dylib included
- **Linux**: Builds Debian package that depends on system SDL3

Releases are automatically created when you push a tag starting with `v` (e.g., `v0.1.0`).

## Contributing

Contributions are welcome! Feel free to submit issues, fork the repository, and send pull requests.

## License

RNES is licensed under the MIT License. See LICENSE for more information.

## Acknowledgements

- Thanks to the Rust community for their support and tools.
- Special thanks to the NESdev community for their invaluable resources and documentation.
