#Requires -Version 5.1
<#
.SYNOPSIS
    Installs reqx CLI for Windows
.DESCRIPTION
    Downloads and installs reqx to the user's local bin directory
.EXAMPLE
    iwr -useb https://reqx.dev/install.ps1 | iex
.EXAMPLE
    $env:REQX_VERSION = "1.0.0"; iwr -useb https://reqx.dev/install.ps1 | iex
#>

$ErrorActionPreference = "Stop"

$ReqxVersion = if ($env:REQX_VERSION) { $env:REQX_VERSION } else { "latest" }
$InstallDir = if ($env:REQX_INSTALL_DIR) { $env:REQX_INSTALL_DIR } else { "$env:LOCALAPPDATA\reqx\bin" }
$GithubRepo = "reqx/reqx"

function Write-Info { param($Message) Write-Host $Message -ForegroundColor Cyan }
function Write-Success { param($Message) Write-Host $Message -ForegroundColor Green }
function Write-Err { param($Message) Write-Host $Message -ForegroundColor Red; exit 1 }

Write-Info "Installing reqx $ReqxVersion..."

# Create install directory
if (!(Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

# Determine download URL
if ($ReqxVersion -eq "latest") {
    $DownloadUrl = "https://github.com/$GithubRepo/releases/latest/download/reqx-x86_64-pc-windows-msvc.zip"
} else {
    $DownloadUrl = "https://github.com/$GithubRepo/releases/download/v$ReqxVersion/reqx-x86_64-pc-windows-msvc.zip"
}

Write-Info "Downloading from: $DownloadUrl"

# Download
$TempZip = Join-Path $env:TEMP "reqx.zip"
try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempZip -UseBasicParsing
} catch {
    Write-Err "Failed to download: $_"
}

# Extract
$TempDir = Join-Path $env:TEMP "reqx-extract"
if (Test-Path $TempDir) { Remove-Item $TempDir -Recurse -Force }
Expand-Archive -Path $TempZip -DestinationPath $TempDir -Force

# Install
$ExePath = Join-Path $InstallDir "reqx.exe"
Move-Item -Path (Join-Path $TempDir "reqx.exe") -Destination $ExePath -Force

# Cleanup
Remove-Item $TempZip -Force -ErrorAction SilentlyContinue
Remove-Item $TempDir -Recurse -Force -ErrorAction SilentlyContinue

Write-Success "Installed to: $ExePath"

# Add to PATH
$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($CurrentPath -notlike "*$InstallDir*") {
    Write-Info "Adding to PATH..."
    [Environment]::SetEnvironmentVariable("Path", "$CurrentPath;$InstallDir", "User")
    $env:Path = "$env:Path;$InstallDir"
}

# Verify
Write-Host ""
& $ExePath --version
Write-Success "Installation complete!"
Write-Host ""
Write-Host "You may need to restart your terminal for PATH changes to take effect."
