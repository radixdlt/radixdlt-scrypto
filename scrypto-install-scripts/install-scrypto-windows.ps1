# Requires PowerShell running as Administrator
#Requires -RunAsAdministrator

# Script configuration
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"  # Speeds up downloads

# Colors for output
function Write-ColorOutput($ForegroundColor) {
    $fc = $host.UI.RawUI.ForegroundColor
    $host.UI.RawUI.ForegroundColor = $ForegroundColor
    if ($args) {
        Write-Output $args
    }
    $host.UI.RawUI.ForegroundColor = $fc
}

function Install-IfNotPresent {
    param (
        [string]$Name,
        [string]$Command,
        [scriptblock]$InstallScript
    )
    
    Write-ColorOutput Yellow "Checking for $Name..."
    if (!(Get-Command $Command -ErrorAction SilentlyContinue)) {
        Write-ColorOutput Cyan "Installing $Name..."
        & $InstallScript
        if ($LASTEXITCODE -ne 0) {
            Write-ColorOutput Red "Failed to install $Name"
            exit 1
        }
        Write-ColorOutput Green "$Name installed successfully"
    } else {
        Write-ColorOutput Green "$Name is already installed"
    }
}

# Check if running as admin
if (-NOT ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
    Write-ColorOutput Red "Please run this script as Administrator"
    exit 1
}

# 1. Install Git if not present
Install-IfNotPresent "Git" "git" {
    # Using winget for Git installation
    winget install --id Git.Git -e --source winget
    # Enable long paths
    git config --system core.longpaths true
}

# 2. Install Visual Studio Build Tools
$vsInstallerPath = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (!(Test-Path $vsInstallerPath)) {
    Write-ColorOutput Cyan "Downloading Visual Studio Build Tools 2022..."
    $vsUrl = "https://aka.ms/vs/17/release/vs_buildtools.exe"
    $vsInstaller = "$env:TEMP\vs_buildtools.exe"
    Invoke-WebRequest -Uri $vsUrl -OutFile $vsInstaller
    
    Write-ColorOutput Cyan "Installing Visual Studio Build Tools with C++ support..."
    Start-Process -Wait -FilePath $vsInstaller -ArgumentList "--quiet", "--wait", "--norestart", "--nocache", `
        "--installPath", "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\BuildTools", `
        "--add", "Microsoft.VisualStudio.Workload.VCTools", `
        "--includeRecommended"
    
    Remove-Item $vsInstaller
}

# 3. Install LLVM
$llvmVersion = "17.0.6"
Install-IfNotPresent "LLVM" "clang" {
    Write-ColorOutput Cyan "Downloading LLVM..."
    $llvmUrl = "https://github.com/llvm/llvm-project/releases/download/llvmorg-$llvmVersion/LLVM-$llvmVersion-win64.exe"
    $llvmInstaller = "$env:TEMP\LLVM-$llvmVersion-win64.exe"
    Invoke-WebRequest -Uri $llvmUrl -OutFile $llvmInstaller
    
    Write-ColorOutput Cyan "Installing LLVM..."
    Start-Process -Wait -FilePath $llvmInstaller -ArgumentList "/S", "/D=C:\Program Files\LLVM"
    Remove-Item $llvmInstaller
}

# 4. Install Rust
Install-IfNotPresent "Rust" "rustc" {
    Write-ColorOutput Cyan "Downloading and installing Rust..."
    $rustupInit = "$env:TEMP\rustup-init.exe"
    Invoke-WebRequest -Uri "https://win.rustup.rs" -OutFile $rustupInit
    Start-Process -Wait -FilePath $rustupInit -ArgumentList "-y", "--default-toolchain", "1.77.2"
    Remove-Item $rustupInit
}

# 5. Set Rust default version
Write-ColorOutput Cyan "Setting Rust default version to 1.77.2..."
rustup default 1.77.2

# 6. Add WebAssembly target
Write-ColorOutput Cyan "Adding WebAssembly target..."
rustup target add wasm32-unknown-unknown

# 7. Install Radix Engine Simulator and CLI tools
Write-ColorOutput Cyan "Installing Radix Engine Simulator and CLI tools..."
cargo install --force radix-clis@1.2.0

# Final success message
Write-ColorOutput Green "`nInstallation complete! Please restart your terminal to ensure all changes take effect."
Write-ColorOutput Yellow "`nTo verify the installation, you can run:"
Write-ColorOutput White "git --version"
Write-ColorOutput White "cl"
Write-ColorOutput White "clang --version"
Write-ColorOutput White "rustc --version"
Write-ColorOutput White "cargo --version"
