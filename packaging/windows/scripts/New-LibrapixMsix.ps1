param(
    [Parameter(Mandatory = $true)]
    [string]$StageDir,
    [Parameter(Mandatory = $true)]
    [string]$OutputMsix,
    [Parameter(Mandatory = $true)]
    [string]$PfxPath,
    [Parameter(Mandatory = $true)]
    [string]$Password,
    [string]$ManifestPath = "packaging/windows/msix/AppxManifest.xml",
    [string]$TimestampUrl = "http://timestamp.digicert.com"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $StageDir)) {
    throw "Stage directory not found: $StageDir"
}
if (-not (Test-Path $ManifestPath)) {
    throw "Manifest not found: $ManifestPath"
}
if (-not (Test-Path $PfxPath)) {
    throw "PFX not found: $PfxPath"
}

$makeappx = Get-Command makeappx.exe -ErrorAction Stop
$signtool = Get-Command signtool.exe -ErrorAction Stop

$manifestTarget = Join-Path $StageDir "AppxManifest.xml"
Copy-Item -Path $ManifestPath -Destination $manifestTarget -Force

$outputDir = Split-Path -Parent $OutputMsix
if (-not (Test-Path $outputDir)) {
    New-Item -ItemType Directory -Path $outputDir | Out-Null
}

if (Test-Path $OutputMsix) {
    Remove-Item $OutputMsix -Force
}

& $makeappx.Source pack /d $StageDir /p $OutputMsix /o

& $signtool.Source sign `
    /fd SHA256 `
    /f $PfxPath `
    /p $Password `
    /tr $TimestampUrl `
    /td SHA256 `
    $OutputMsix

& $signtool.Source verify /pa /v $OutputMsix

Write-Host "Created and signed MSIX: $OutputMsix"
