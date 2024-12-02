# Requires PowerShell running as Administrator
#Requires -RunAsAdministrator

# Versions to install
$llvmVersion = '18.1.8'
$rustVersion = '1.81.0'
$radixCliVersion = '1.3.0'

# Script configuration
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'Continue'

# Colors for output
function Write-ColorOutput($ForegroundColor) {
    $fc = $host.UI.RawUI.ForegroundColor
    $host.UI.RawUI.ForegroundColor = $ForegroundColor
    if ($args) {
        Write-Output $args
    }
    $host.UI.RawUI.ForegroundColor = $fc
}

function Refresh-EnvironmentVariables {
    # Refresh the PATH variable
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
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
        Refresh-EnvironmentVariables
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
    Refresh-EnvironmentVariables
    # Enable long paths
    git config --system core.longpaths true
}

# 2. Install Visual Studio Build Tools
$vsInstallerPath = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$Name = "Visual Studio Build Tools 2022"
Write-ColorOutput Yellow "Checking for $Name..."
if (!(Test-Path $vsInstallerPath)) {
    Write-ColorOutput Cyan "Downloading $Name..."
    $vsUrl = "https://aka.ms/vs/17/release/vs_buildtools.exe"
    $vsInstaller = "$env:TEMP\vs_buildtools.exe"
    Start-BitsTransfer -Source $vsUrl -Destination $vsInstaller

    Write-ColorOutput Cyan "Installing $Name with C++ support - This may take a long time..."
    Start-Process -Wait -FilePath $vsInstaller  -ArgumentList @(
        "--passive",
        "--wait",
        "--norestart",
        "--nocache",
        "--installPath", "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\BuildTools",
        "--add", "Microsoft.VisualStudio.Workload.VCTools"
        "--includeRecommended"
    )
    
    Remove-Item $vsInstaller
    Refresh-EnvironmentVariables
} else {
    Write-ColorOutput Green "Visual Studio Build Tools are already installed"
}

# 3. Install LLVM
Install-IfNotPresent "LLVM" "clang" {
    Write-ColorOutput Cyan "Downloading LLVM..."

    $llvmUrl = "https://github.com/llvm/llvm-project/releases/download/llvmorg-$llvmVersion/LLVM-$llvmVersion-win64.exe"
    $llvmInstaller = "$env:TEMP\LLVM-$llvmVersion-win64.exe"
    Start-BitsTransfer -Source $llvmUrl -Destination $llvmInstaller
    
    Write-ColorOutput Cyan "Installing LLVM - This may take a long time too..."
    Start-Process -Wait -FilePath $llvmInstaller -ArgumentList "/S", "/D=C:\Program Files\LLVM"
    Remove-Item $llvmInstaller

    # Add LLVM bin directory to the system PATH
    $llvmBinPath = "C:\Program Files\LLVM\bin"
    [Environment]::SetEnvironmentVariable("Path", $env:Path + ";$llvmBinPath", [EnvironmentVariableTarget]::Machine)
    $env:Path += ";$llvmBinPath"
}

# 4. Install Rust
Install-IfNotPresent "Rust" "rustc" {
    Write-ColorOutput Cyan "Downloading and installing Rust..."
    $rustupInit = "$env:TEMP\rustup-init.exe"
    Start-BitsTransfer -Source "https://win.rustup.rs" -Destination $rustupInit
    Start-Process -Wait -FilePath $rustupInit -ArgumentList "-y", "--default-toolchain", $rustVersion
    Remove-Item $rustupInit
}

# 5. Set Rust default version
Write-ColorOutput Cyan "Setting Rust default version to $rustVersion..."
rustup default $rustVersion

# 6. Add WebAssembly target
Write-ColorOutput Cyan "Adding WebAssembly target..."
rustup target add wasm32-unknown-unknown

# 7. Install Radix Engine Simulator and CLI tools
Write-ColorOutput Cyan "Installing Radix Engine Simulator and CLI tools..."
cargo install --force radix-clis@$radixCliVersion
Refresh-EnvironmentVariables

# Final success message
Write-ColorOutput Green "`nInstallation complete! Please restart your terminal to ensure all changes take effect."
Write-ColorOutput Yellow "`nVerifying instillations..."
Write-ColorOutput White "Versions installed:"
Write-ColorOutput White "git"
git --version
Write-ColorOutput White "`nclang"
clang --version
Write-ColorOutput White "`nrustc"
rustc --version
Write-ColorOutput White "`ncargo"
cargo --version
Write-ColorOutput White "`nscrypto"
scrypto --version