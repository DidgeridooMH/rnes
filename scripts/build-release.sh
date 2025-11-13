#!/bin/bash
# Local build script for testing release packaging

set -e

echo "Building RNES locally..."

# Detect platform
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    PLATFORM="linux"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    PLATFORM="macos"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
    PLATFORM="windows"
else
    echo "Unknown platform: $OSTYPE"
    exit 1
fi

echo "Detected platform: $PLATFORM"

# Build the project
echo "Running cargo build --release..."
cargo build --release

# Create release package based on platform
case $PLATFORM in
    windows)
        echo "Creating Windows release package..."
        mkdir -p release
        cp target/release/rnes.exe release/
        cp vcpkg_installed/x64-windows/bin/SDL3.dll release/
        cd release
        powershell Compress-Archive -Path * -DestinationPath ../rnes-windows-x64.zip -Force
        cd ..
        echo "Created: rnes-windows-x64.zip"
        ;;
        
    macos)
        echo "Creating macOS release package..."
        mkdir -p release
        cp target/release/rnes release/
        find vcpkg_installed -name "*.dylib" -exec cp {} release/ \;
        cd release
        zip -r ../rnes-macos-x64.zip *
        cd ..
        echo "Created: rnes-macos-x64.zip"
        ;;
        
    linux)
        echo "Creating Linux Debian package..."
        mkdir -p debian-pkg/DEBIAN
        mkdir -p debian-pkg/usr/bin
        mkdir -p debian-pkg/usr/share/doc/rnes
        
        cp target/release/rnes debian-pkg/usr/bin/
        
        cat > debian-pkg/DEBIAN/control << EOF
Package: rnes
Version: 0.1.0
Section: games
Priority: optional
Architecture: amd64
Depends: libsdl3-0 | libsdl3-dev
Maintainer: DidgeridooMH
Description: NES Emulator written in Rust
 A Nintendo Entertainment System (NES) emulator implementation
 written in Rust with SDL3 support.
EOF
        
        dpkg-deb --build debian-pkg rnes_0.1.0_amd64.deb
        echo "Created: rnes_0.1.0_amd64.deb"
        ;;
esac

echo "Build complete!"
