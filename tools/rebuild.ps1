# ALVR localization: rebuild the patched Dashboard / Launcher for another ALVR version.
# ASCII only on purpose: Windows PowerShell 5.1 mis-parses non-BOM UTF-8 scripts.
#
#   .\rebuild.ps1 -Tag v20.14.1 -Install
#
# Requirements: Rust toolchain, Visual Studio Build Tools (C++), Python 3, network access.

param(
    [string]$Tag = "v20.14.1",
    [string]$WorkDir = "$env:TEMP\alvr-l10n-rebuild",
    [string]$AlvrDir = "",
    [switch]$Install,
    [switch]$KeepSources
)

$ErrorActionPreference = "Stop"
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$packageRoot = Split-Path -Parent $scriptDir

function Write-Step($message) {
    Write-Host ""
    Write-Host "==> $message" -ForegroundColor Cyan
}

function Fail($message) {
    Write-Host "ERROR: $message" -ForegroundColor Red
    exit 1
}

# ---------------------------------------------------------------- prerequisites
Write-Step "Looking for the toolchain"

$cargo = $null
foreach ($candidate in @("$env:USERPROFILE\.cargo\bin\cargo.exe", "cargo")) {
    $cmd = Get-Command $candidate -ErrorAction SilentlyContinue
    if ($cmd) { $cargo = $cmd.Source; break }
}
if (-not $cargo) { Fail "cargo not found. Install Rust from https://rustup.rs" }
Write-Host "cargo : $cargo"

$python = $null
foreach ($candidate in @("python", "python3", "py")) {
    $cmd = Get-Command $candidate -ErrorAction SilentlyContinue
    if ($cmd) {
        try {
            & $cmd.Source -c "import sys; sys.exit(0 if sys.version_info[0] >= 3 else 1)" 2>$null
            if ($LASTEXITCODE -eq 0) { $python = $cmd.Source; break }
        } catch { }
    }
}
if (-not $python) { Fail "Python 3 not found. Install it from https://www.python.org/downloads/" }
Write-Host "python: $python"

$vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
if (-not (Test-Path $vswhere)) { Fail "Visual Studio installer not found. Install 'Build Tools for Visual Studio' with the C++ workload." }
$vsPath = & $vswhere -products * -latest -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $vsPath) { Fail "Visual Studio C++ build tools not found." }
$vcvars = Join-Path $vsPath "VC\Auxiliary\Build\vcvars64.bat"
if (-not (Test-Path $vcvars)) { Fail "vcvars64.bat not found under $vsPath" }
Write-Host "msvc   : $vsPath"

# --------------------------------------------------------------------- sources
Write-Step "Fetching ALVR $Tag sources"
New-Item -ItemType Directory -Force -Path $WorkDir | Out-Null
$zipPath = Join-Path $WorkDir "alvr-$Tag.zip"
if (-not (Test-Path $zipPath)) {
    Invoke-WebRequest -Uri "https://codeload.github.com/alvr-org/ALVR/zip/refs/tags/$Tag" -OutFile $zipPath -UseBasicParsing
}
$srcDir = Join-Path $WorkDir "src-$Tag"
if (Test-Path $srcDir) { Remove-Item $srcDir -Recurse -Force }
Expand-Archive -Path $zipPath -DestinationPath (Join-Path $WorkDir "unzip-$Tag") -Force
$extracted = Get-ChildItem (Join-Path $WorkDir "unzip-$Tag") -Directory | Select-Object -First 1
Move-Item $extracted.FullName $srcDir
Write-Host "source : $srcDir"

Write-Step "Applying the localization patch"
& $python (Join-Path $scriptDir "apply_localization.py") --source $srcDir
if ($LASTEXITCODE -ne 0) { Fail "patching failed (see the message above)" }

# ----------------------------------------------------------------------- build
Write-Step "Building (this takes a few minutes)"
$buildCmd = "call `"$vcvars`" >nul && `"$cargo`" build -p alvr_dashboard -p alvr_launcher --release"
Push-Location $srcDir
try {
    cmd /c $buildCmd
    if ($LASTEXITCODE -ne 0) { Fail "cargo build failed" }
} finally {
    Pop-Location
}

$outDir = Join-Path $srcDir "target\release"
$dashboard = Join-Path $outDir "alvr_dashboard.exe"
$launcher = Join-Path $outDir "alvr_launcher.exe"
if (-not (Test-Path $dashboard)) { Fail "dashboard binary not produced" }

# --------------------------------------------------------------------- install
Write-Step "Copying the results into the package"
$stageDir = Join-Path $packageRoot "build-$Tag"
New-Item -ItemType Directory -Force -Path $stageDir | Out-Null
Copy-Item $dashboard (Join-Path $stageDir "ALVR Dashboard.exe") -Force
if (Test-Path $launcher) { Copy-Item $launcher (Join-Path $stageDir "ALVR Launcher.exe") -Force }
Write-Host "output : $stageDir"

if ($Install) {
    if (-not $AlvrDir) { $AlvrDir = Split-Path -Parent $packageRoot }
    $installDir = Join-Path $AlvrDir "installations\$Tag"
    if (-not (Test-Path $installDir)) {
        Fail "ALVR $Tag is not installed at $installDir. Install it with ALVR Launcher first."
    }
    $targetDashboard = Join-Path $installDir "ALVR Dashboard.exe"
    if (-not (Test-Path "$targetDashboard.orig")) {
        Copy-Item $targetDashboard "$targetDashboard.orig" -Force
        Write-Host "backup  : $targetDashboard.orig"
    }
    Copy-Item $dashboard $targetDashboard -Force
    Write-Host "installed dashboard -> $targetDashboard"

    $targetLauncher = Join-Path $AlvrDir "ALVR Launcher.exe"
    if ((Test-Path $launcher) -and (Test-Path $targetLauncher)) {
        if (-not (Test-Path "$targetLauncher.orig")) { Copy-Item $targetLauncher "$targetLauncher.orig" -Force }
        Copy-Item $launcher $targetLauncher -Force
        Write-Host "installed launcher  -> $targetLauncher"
    }

    foreach ($item in @("language.ini")) {
        $source = Join-Path $packageRoot $item
        if ((Test-Path $source) -and -not (Test-Path (Join-Path $AlvrDir $item))) {
            Copy-Item $source (Join-Path $AlvrDir $item) -Force
        }
    }
    $languagesSource = Join-Path $packageRoot "languages"
    if ((Test-Path $languagesSource) -and -not (Test-Path (Join-Path $AlvrDir "languages"))) {
        Copy-Item $languagesSource (Join-Path $AlvrDir "languages") -Recurse -Force
    }
}

if (-not $KeepSources) {
    Write-Host ""
    Write-Host "sources kept at $srcDir (delete it to free a few GB)" -ForegroundColor DarkGray
}

Write-Host ""
Write-Host "Done." -ForegroundColor Green
