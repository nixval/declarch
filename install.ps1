param(
    [string]$Version = "0.8.1",
    [string]$Repo = "nixval/declarch"
)

$ErrorActionPreference = "Stop"

Write-Warning "Windows installer path is experimental (alpha)."
Write-Warning "Use on non-production machines first and validate with 'declarch info' and 'declarch lint'."

$arch = $env:PROCESSOR_ARCHITECTURE
switch ($arch.ToLower()) {
    "amd64" { $target = "x86_64-pc-windows-msvc" }
    default {
        Write-Error "Unsupported Windows architecture: $arch"
        exit 1
    }
}

$asset = "declarch-$target.zip"
$url = "https://github.com/$Repo/releases/download/v$Version/$asset"

$installRoot = Join-Path $env:LOCALAPPDATA "Programs\declarch\bin"
New-Item -ItemType Directory -Path $installRoot -Force | Out-Null
$metaRoot = Join-Path $env:LOCALAPPDATA "declarch"
New-Item -ItemType Directory -Path $metaRoot -Force | Out-Null

$tmpDir = New-Item -ItemType Directory -Path (Join-Path $env:TEMP ("declarch-" + [guid]::NewGuid().ToString())) -Force
$zipPath = Join-Path $tmpDir.FullName $asset

Write-Host "Downloading declarch $Version for $target..."
Invoke-WebRequest -Uri $url -OutFile $zipPath
Expand-Archive -Path $zipPath -DestinationPath $tmpDir.FullName -Force

$exePath = Join-Path $tmpDir.FullName "declarch.exe"
if (-not (Test-Path $exePath)) {
    Write-Error "declarch.exe not found in downloaded archive"
    exit 1
}

Copy-Item $exePath (Join-Path $installRoot "declarch.exe") -Force

# Optional short alias binary if release includes decl.exe
$declExe = Join-Path $tmpDir.FullName "decl.exe"
if (Test-Path $declExe) {
    Copy-Item $declExe (Join-Path $installRoot "decl.exe") -Force
}

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$installRoot*") {
    [Environment]::SetEnvironmentVariable("Path", ($userPath.TrimEnd(';') + ";" + $installRoot), "User")
    Write-Host "Added $installRoot to User PATH."
    Write-Host "Open a new terminal to use declarch."
}

Write-Host "Installed declarch to $installRoot"
& (Join-Path $installRoot "declarch.exe") --version

# Persist installation channel marker for update guidance (best-effort)
$markerPath = Join-Path $metaRoot "install-channel.json"
@{
    channel = "curl"
    installed_at = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
} | ConvertTo-Json -Compress | Set-Content -Path $markerPath -Encoding UTF8

# Lightweight smoke checks (safe on fresh machines, no config required)
Write-Host "Running smoke checks..."
& (Join-Path $installRoot "declarch.exe") --help | Out-Null
try {
    & (Join-Path $installRoot "declarch.exe") info | Out-Null
} catch {
    # Keep installer non-blocking for first-run state/config scenarios
}
Write-Host "Smoke checks complete."
