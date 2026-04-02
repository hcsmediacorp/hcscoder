# hcscoder Install Script for Windows (PowerShell)
# 
# This script installs hcscoder to %APPDATA%\hcscoder and adds it to PATH.
#
# Usage: iwr -useb https://raw.githubusercontent.com/hcsmedia/hcscoder/main/install.ps1 | iex
#
# MIT License (c) 2026 hcsmedia
# Attribution to hcsmedia is mandatory for all modifications and distributions.

param(
    [switch]$Local,
    [string]$Version = "latest",
    [switch]$Help
)

if ($Help) {
    Write-Host @"
Usage: .\install.ps1 [-Local] [-Version X.Y.Z] [-Help]

Options:
  -Local      Install to user directory instead of system-wide
  -Version    Install specific version (default: latest)
  -Help       Show this help message

MIT License (c) 2026 hcsmedia
"@
    exit 0
}

# Colors
$Red = [ConsoleColor]::Red
$Green = [ConsoleColor]::Green
$Yellow = [ConsoleColor]::Yellow
$Blue = [ConsoleColor]::Blue

function Write-Info {
    param([string]$Message)
    $Host.UI.RawUI.ForegroundColor = $Blue
    Write-Host "[INFO] $Message"
    $Host.UI.RawUI.ForegroundColor = [ConsoleColor]::White
}

function Write-Success {
    param([string]$Message)
    $Host.UI.RawUI.ForegroundColor = $Green
    Write-Host "[SUCCESS] $Message"
    $Host.UI.RawUI.ForegroundColor = [ConsoleColor]::White
}

function Write-Warning {
    param([string]$Message)
    $Host.UI.RawUI.ForegroundColor = $Yellow
    Write-Host "[WARNING] $Message"
    $Host.UI.RawUI.ForegroundColor = [ConsoleColor]::White
}

function Write-Error-Custom {
    param([string]$Message)
    $Host.UI.RawUI.ForegroundColor = $Red
    Write-Host "[ERROR] $Message"
    $Host.UI.RawUI.ForegroundColor = [ConsoleColor]::White
}

# Configuration
$Repo = "hcsmedia/hcscoder"
$InstallDir = if ($Local) { 
    "$env:APPDATA\hcscoder\bin" 
} else { 
    "$env:ProgramFiles\hcscoder\bin" 
}
$ConfigDir = "$env:APPDATA\hcscoder\config"
$DataDir = "$env:APPDATA\hcscoder\data"

Write-Host ""
Write-Host "=========================================="
Write-Host "  hcscoder Installer for Windows"
Write-Host "  MIT License (c) 2026 hcsmedia"
Write-Host "=========================================="
Write-Host ""

# Check for admin privileges if not local install
if (-not $Local) {
    $isAdmin = ([Security.Principal.WindowsPrincipal] `
        [Security.Principal.WindowsIdentity]::GetCurrent()).`
        IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    
    if (-not $isAdmin) {
        Write-Warning "Not running as Administrator. Will install to user directory."
        $InstallDir = "$env:APPDATA\hcscoder\bin"
        $Local = $true
    }
}

# Create directories
Write-Info "Creating directories..."
try {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    New-Item -ItemType Directory -Force -Path $ConfigDir | Out-Null
    New-Item -ItemType Directory -Force -Path $DataDir | Out-Null
    Write-Success "Directories created"
} catch {
    Write-Error-Custom "Failed to create directories: $_"
    exit 1
}

# Detect architecture
$Arch = switch ($env:PROCESSOR_ARCHITECTURE) {
    "AMD64" { "x86_64" }
    "ARM64" { "aarch64" }
    default { 
        Write-Error-Custom "Unsupported architecture: $env:PROCESSOR_ARCHITECTURE"
        exit 1 
    }
}

$OS = "pc-windows-msvc"

Write-Info "Detecting platform: $OS ($Arch)"

# Get latest release version
if ($Version -eq "latest") {
    try {
        $Release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
        $Version = $Release.tag_name
    } catch {
        Write-Error-Custom "Failed to fetch latest release: $_"
        exit 1
    }
}

Write-Info "Installing version: $Version"

# Construct download URL
$BinaryName = "hcscoder-$Version-$Arch-$OS"
$DownloadUrl = "https://github.com/$Repo/releases/download/$Version/$BinaryName.exe"

Write-Info "Downloading from: $DownloadUrl"

# Download binary
try {
    $TempFile = "$env:TEMP\hcscoder.exe"
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempFile -UseBasicParsing
} catch {
    Write-Error-Custom "Failed to download binary: $_"
    Write-Error-Custom "Please check if the release exists for your platform"
    exit 1
}

# Move to install directory
try {
    Move-Item -Force $TempFile "$InstallDir\hcscoder.exe"
    Write-Success "Binary installed to $InstallDir\hcscoder.exe"
} catch {
    Write-Error-Custom "Failed to move binary: $_"
    exit 1
}

# Add to PATH if not already present
$CurrentPath = [Environment]::GetEnvironmentVariable("Path", 
    if ($Local) { "User" } else { "Machine" })

if ($CurrentPath -notlike "*$InstallDir*") {
    Write-Info "Adding $InstallDir to PATH..."
    try {
        $NewPath = "$CurrentPath;$InstallDir"
        [Environment]::SetEnvironmentVariable("Path", 
            $NewPath, 
            if ($Local) { "User" } else { "Machine" })
        Write-Success "PATH updated (restart terminal to apply)"
    } catch {
        Write-Warning "Failed to update PATH: $_"
        Write-Info "Please manually add $InstallDir to your PATH"
    }
}

# Create default config file
$ConfigFile = "$ConfigDir\config.toml"
if (-not (Test-Path $ConfigFile)) {
    Write-Info "Creating default configuration..."
    @'
# hcscoder Configuration
# MIT License (c) 2026 hcsmedia

# OpenRouter API Key (get yours at https://openrouter.ai/keys)
# You can also set this via environment variable: OPENROUTER_API_KEY
# api_key = ""

# Default model to use
model = "anthropic/claude-sonnet-4-20250514"

# Temperature for completions (0.0 - 2.0)
temperature = 0.7

# Maximum tokens for completions
max_tokens = 4096

# Enable verbose logging
verbose = false

# Working directory (default: current directory)
# working_dir = ""
'@ | Set-Content -Path $ConfigFile
    Write-Success "Default configuration created at $ConfigFile"
} else {
    Write-Info "Configuration already exists at $ConfigFile"
}

# Verify installation
Write-Info "Verifying installation..."
$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")

try {
    $VersionOutput = & "$InstallDir\hcscoder.exe" --version 2>&1
    Write-Success "hcscoder installed successfully! Version: $VersionOutput"
    Write-Host ""
    Write-Host "To get started:"
    Write-Host "  1. Set your OpenRouter API key:"
    Write-Host "     `$env:OPENROUTER_API_KEY='your-key-here'"
    Write-Host "  2. Or edit the config file: $ConfigFile"
    Write-Host "  3. Run: hcscoder --help"
    Write-Host ""
} catch {
    Write-Error-Custom "Installation verification failed"
    Write-Error-Custom "Please restart your terminal and ensure $InstallDir is in PATH"
    exit 1
}

Write-Success "Installation complete!"
Write-Host ""
Write-Host "Legal Notice:"
Write-Host "  MIT License (c) 2026 hcsmedia"
Write-Host "  Attribution to hcsmedia is mandatory for all modifications and distributions."
Write-Host ""
