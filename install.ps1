# Scheduler Installation Script for Windows
# PowerShell 5.1+

$ErrorActionPreference = "Stop"

Write-Host "🗓️  Installing Scheduler..." -ForegroundColor Green
Write-Host ""

# Check for Rust
try {
    $rustVersion = cargo --version
    Write-Host "✓ Rust detected: $rustVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ Rust is not installed." -ForegroundColor Red
    Write-Host "Please install Rust from https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

# Build release
Write-Host ""
Write-Host "📦 Building release binary..." -ForegroundColor Cyan
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed" -ForegroundColor Red
    exit 1
}

Write-Host "✓ Build successful" -ForegroundColor Green

# Determine install location
$InstallDir = "$env:LOCALAPPDATA\Programs\Scheduler"

# Create install directory
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

# Copy binary
Write-Host ""
Write-Host "📥 Installing to $InstallDir..." -ForegroundColor Cyan
Copy-Item "target\release\scheduler.exe" "$InstallDir\sched.exe" -Force

Write-Host "✓ Binary installed as 'sched.exe'" -ForegroundColor Green

# Add to PATH
$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($CurrentPath -notlike "*$InstallDir*") {
    Write-Host ""
    Write-Host "Adding to PATH..." -ForegroundColor Cyan
    [Environment]::SetEnvironmentVariable(
        "Path",
        "$CurrentPath;$InstallDir",
        "User"
    )
    Write-Host "✓ Added to PATH (restart your terminal to use)" -ForegroundColor Green
} else {
    Write-Host "✓ Installation directory already in PATH" -ForegroundColor Green
}

# Create config directory
$ConfigDir = "$env:APPDATA\scheduler"
if (-not (Test-Path $ConfigDir)) {
    New-Item -ItemType Directory -Path $ConfigDir -Force | Out-Null
}
Write-Host "✓ Config directory created at $ConfigDir" -ForegroundColor Green

# Create data directory
$DataDir = "$env:APPDATA\scheduler\scheduler\data"
if (-not (Test-Path $DataDir)) {
    New-Item -ItemType Directory -Path $DataDir -Force | Out-Null
}
$HistoryDir = "$DataDir\history"
if (-not (Test-Path $HistoryDir)) {
    New-Item -ItemType Directory -Path $HistoryDir -Force | Out-Null
}
Write-Host "✓ Data directory created at $DataDir" -ForegroundColor Green

Write-Host ""
Write-Host "✅ Installation complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Quick start:" -ForegroundColor Cyan
Write-Host "  sched add `"My Task`" --start 09:00 --end 10:00"
Write-Host "  sched start"
Write-Host "  sched ui"
Write-Host ""
Write-Host "For help:" -ForegroundColor Cyan
Write-Host "  sched --help"
Write-Host ""
Write-Host "⚠️  Please restart your terminal to use 'sched' command" -ForegroundColor Yellow
Write-Host ""
