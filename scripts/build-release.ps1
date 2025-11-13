# Local Windows build script for testing release packaging

Write-Host "Building RNES for Windows..." -ForegroundColor Green

# Build the project
Write-Host "Running cargo build --release..." -ForegroundColor Yellow
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

# Create release package
Write-Host "Creating Windows release package..." -ForegroundColor Yellow

# Create release directory
if (Test-Path "release") {
    Remove-Item -Recurse -Force release
}
New-Item -ItemType Directory -Force -Path release | Out-Null

# Copy files
Copy-Item target\release\rnes.exe release\

# Check if SDL3.dll exists and copy it
$sdl3Path = "vcpkg_installed\x64-windows\bin\SDL3.dll"
if (Test-Path $sdl3Path) {
    Copy-Item $sdl3Path release\
    Write-Host "✓ SDL3.dll included" -ForegroundColor Green
} else {
    Write-Host "⚠ Warning: SDL3.dll not found at $sdl3Path" -ForegroundColor Yellow
    Write-Host "  Make sure to run: .\vcpkg\vcpkg.exe install sdl3:x64-windows" -ForegroundColor Yellow
}

# Create ZIP file
if (Test-Path "rnes-windows-x64.zip") {
    Remove-Item -Force rnes-windows-x64.zip
}

Compress-Archive -Path release\* -DestinationPath rnes-windows-x64.zip

Write-Host "✓ Created: rnes-windows-x64.zip" -ForegroundColor Green
Write-Host ""
Write-Host "Build complete! Package contents:" -ForegroundColor Green
Get-ChildItem release\ | Format-Table Name, Length -AutoSize
