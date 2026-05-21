Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$Root = if ($env:ROOT) { $env:ROOT } else { (Resolve-Path (Join-Path $ScriptDir '..')).Path }
$Dist = Join-Path $Root 'dist'
$Uploads = Join-Path $Dist 'upload'
$RawDir = Join-Path $Dist 'alacritty-config-ui'
$TargetDir = Join-Path $Root 'target\release'
$BundleDir = Join-Path $TargetDir 'bundle'
$AssetPrefix = if ($env:RELEASE_ASSET_PREFIX) { $env:RELEASE_ASSET_PREFIX } else { 'alacritty-config-ui' }
$SetupPath = Join-Path $Uploads "$AssetPrefix-windows-setup.exe"
$MsiPath = Join-Path $Uploads "$AssetPrefix-windows-installer.msi"
$PortablePath = Join-Path $Uploads "$AssetPrefix-windows-portable.zip"

function Require-Path {
  param(
    [string]$Path,
    [string]$Description
  )

  if (-not (Test-Path -LiteralPath $Path)) {
    throw "missing $Description at $Path"
  }
}

function Write-Sha256 {
  param([string]$Path)

  $hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $Path).Hash.ToLowerInvariant()
  Set-Content -LiteralPath "$Path.sha256" -Value "$hash  $([IO.Path]::GetFileName($Path))"
}

function Copy-PortablePayload {
  Copy-Item -LiteralPath (Join-Path $TargetDir 'alacritty-config-ui.exe') -Destination (Join-Path $RawDir 'alacritty-config-ui.exe')
  Copy-Item -LiteralPath (Join-Path $Root 'vendor\alacritty-theme\themes') -Destination (Join-Path $RawDir 'themes') -Recurse
  Copy-Item -LiteralPath (Join-Path $Root 'LICENSE-APACHE') -Destination (Join-Path $RawDir 'LICENSE-APACHE')
  Copy-Item -LiteralPath (Join-Path $Root 'LICENSE-MIT') -Destination (Join-Path $RawDir 'LICENSE-MIT')
}

function Assert-ZipContains {
  param(
    [string]$ZipPath,
    [string]$Entry,
    [string]$Description
  )

  Add-Type -AssemblyName System.IO.Compression.FileSystem
  $archive = [IO.Compression.ZipFile]::OpenRead($ZipPath)
  try {
    $entries = $archive.Entries.FullName
    if ($entries -notcontains $Entry) {
      throw "missing $Description in $ZipPath"
    }
  }
  finally {
    $archive.Dispose()
  }
}

function Assert-ZipContainsPattern {
  param(
    [string]$ZipPath,
    [string]$Pattern,
    [string]$Description
  )

  Add-Type -AssemblyName System.IO.Compression.FileSystem
  $archive = [IO.Compression.ZipFile]::OpenRead($ZipPath)
  try {
    $entries = $archive.Entries.FullName
    if (-not ($entries | Where-Object { $_ -like $Pattern })) {
      throw "missing $Description in $ZipPath"
    }
  }
  finally {
    $archive.Dispose()
  }
}

function Assert-InstallerListingContains {
  param(
    [string]$Archive,
    [string]$Needle,
    [string]$Description
  )

  $sevenZip = Get-Command 7z -ErrorAction SilentlyContinue
  if (-not $sevenZip) {
    return
  }

  $listing = (& $sevenZip.Source l $Archive 2>$null | Out-String)
  if ($LASTEXITCODE -ne 0) {
    return
  }
  if ($listing -notmatch $Needle) {
    throw "missing $Description in $Archive"
  }
}

Remove-Item -LiteralPath $Dist -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $Uploads | Out-Null
New-Item -ItemType Directory -Path $RawDir | Out-Null

Push-Location $Root
try {
  npm run tauri build -- --bundles nsis,msi
}
finally {
  Pop-Location
}

$SetupSource = Get-ChildItem -LiteralPath (Join-Path $BundleDir 'nsis') -Filter *.exe -File | Select-Object -First 1
$MsiSource = Get-ChildItem -LiteralPath (Join-Path $BundleDir 'msi') -Filter *.msi -File | Select-Object -First 1

if (-not $SetupSource) {
  throw "missing NSIS output in $(Join-Path $BundleDir 'nsis')"
}
if (-not $MsiSource) {
  throw "missing MSI output in $(Join-Path $BundleDir 'msi')"
}

Copy-PortablePayload
Compress-Archive -LiteralPath $RawDir -DestinationPath $PortablePath
Assert-ZipContains $PortablePath 'alacritty-config-ui/alacritty-config-ui.exe' 'portable executable'
Assert-ZipContains $PortablePath 'alacritty-config-ui/LICENSE-APACHE' 'portable LICENSE-APACHE'
Assert-ZipContains $PortablePath 'alacritty-config-ui/LICENSE-MIT' 'portable LICENSE-MIT'
Assert-ZipContainsPattern $PortablePath 'alacritty-config-ui/themes/*' 'portable bundled themes'
Write-Sha256 $PortablePath

Copy-Item -LiteralPath $SetupSource.FullName -Destination $SetupPath
Copy-Item -LiteralPath $MsiSource.FullName -Destination $MsiPath
Assert-InstallerListingContains $SetupPath 'themes[\\/]' 'bundled themes in NSIS installer'
Assert-InstallerListingContains $SetupPath 'LICENSE-APACHE' 'LICENSE-APACHE in NSIS installer'
Assert-InstallerListingContains $SetupPath 'LICENSE-MIT' 'LICENSE-MIT in NSIS installer'
Assert-InstallerListingContains $MsiPath 'themes[\\/]' 'bundled themes in MSI installer'
Assert-InstallerListingContains $MsiPath 'LICENSE-APACHE' 'LICENSE-APACHE in MSI installer'
Assert-InstallerListingContains $MsiPath 'LICENSE-MIT' 'LICENSE-MIT in MSI installer'
Write-Sha256 $SetupPath
Write-Sha256 $MsiPath

Write-Output "created_artifact=$SetupPath"
Write-Output "created_artifact=$MsiPath"
Write-Output "created_artifact=$PortablePath"
