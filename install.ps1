param(
    [string]$Version = "latest",
    [string]$Repo = "nixval/declarch"
)

$ErrorActionPreference = "Stop"
$BinName = "declarch"
$BinAlias = "decl"
$AssetPrefix = "declarch"
$StableId = "declarch"

Write-Warning "Windows installer path is experimental (alpha)."
Write-Warning "Use on non-production machines first and validate with '$BinName info' and '$BinName lint'."

$arch = $env:PROCESSOR_ARCHITECTURE
switch ($arch.ToLower()) {
    "amd64" { $target = "x86_64-pc-windows-msvc" }
    default {
        Write-Error "Unsupported Windows architecture: $arch"
        exit 1
    }
}

$asset = "$AssetPrefix-$target.zip"
if ($Version -eq "latest") {
    $baseUrl = "https://github.com/$Repo/releases/latest/download"
} else {
    $baseUrl = "https://github.com/$Repo/releases/download/v$Version"
}
$url = "$baseUrl/$asset"
$checksumsUrl = "$baseUrl/checksums.txt"

$installRoot = Join-Path $env:LOCALAPPDATA "Programs\$BinName\bin"
New-Item -ItemType Directory -Path $installRoot -Force | Out-Null
$metaRoot = Join-Path $env:LOCALAPPDATA $StableId
New-Item -ItemType Directory -Path $metaRoot -Force | Out-Null

$tmpDir = New-Item -ItemType Directory -Path (Join-Path $env:TEMP ("$BinName-" + [guid]::NewGuid().ToString())) -Force
$zipPath = Join-Path $tmpDir.FullName $asset

if ($Version -eq "latest") {
    Write-Host "Downloading $BinName (latest release) for $target..."
} else {
    Write-Host "Downloading $BinName $Version for $target..."
}
Invoke-WebRequest -Uri $url -OutFile $zipPath
$checksumsPath = Join-Path $tmpDir.FullName "checksums.txt"
Invoke-WebRequest -Uri $checksumsUrl -OutFile $checksumsPath

$expectedSha = $null
foreach ($line in Get-Content -Path $checksumsPath) {
    if ($line -match "^\s*([0-9a-fA-F]{64})\s+\*?(.+?)\s*$") {
        $sha = $matches[1].ToLower()
        $name = $matches[2]
        if ($name -eq $asset) {
            $expectedSha = $sha
            break
        }
    }
}
if (-not $expectedSha) {
    Write-Error "Checksum entry for $asset not found in checksums.txt"
    exit 1
}

$actualSha = (Get-FileHash -Path $zipPath -Algorithm SHA256).Hash.ToLower()
if ($actualSha -ne $expectedSha) {
    Write-Error "Checksum verification failed for $asset. Expected: $expectedSha, Actual: $actualSha"
    exit 1
}
Write-Host "Checksum verified: $asset"

Expand-Archive -Path $zipPath -DestinationPath $tmpDir.FullName -Force

$exePath = Join-Path $tmpDir.FullName "$BinName.exe"
if (-not (Test-Path $exePath)) {
    Write-Error "$BinName.exe not found in downloaded archive"
    exit 1
}

Copy-Item $exePath (Join-Path $installRoot "$BinName.exe") -Force

# Optional short alias binary if release includes alias executable
$declExe = Join-Path $tmpDir.FullName "$BinAlias.exe"
if (Test-Path $declExe) {
    Copy-Item $declExe (Join-Path $installRoot "$BinAlias.exe") -Force
}

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$installRoot*") {
    [Environment]::SetEnvironmentVariable("Path", ($userPath.TrimEnd(';') + ";" + $installRoot), "User")
    Write-Host "Added $installRoot to User PATH."
    Write-Host "Open a new terminal to use $BinName."
}

Write-Host "Installed $BinName to $installRoot"
& (Join-Path $installRoot "$BinName.exe") --version

# Persist installation channel marker for update guidance (best-effort)
$markerPath = Join-Path $metaRoot "install-channel.json"
@{
    channel = "script"
    installed_at = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
} | ConvertTo-Json -Compress | Set-Content -Path $markerPath -Encoding UTF8

# Lightweight smoke checks (safe on fresh machines, no config required)
Write-Host "Running smoke checks..."
& (Join-Path $installRoot "$BinName.exe") --help | Out-Null
try {
    & (Join-Path $installRoot "$BinName.exe") info | Out-Null
} catch {
    # Keep installer non-blocking for first-run state/config scenarios
}
Write-Host "Smoke checks complete."
