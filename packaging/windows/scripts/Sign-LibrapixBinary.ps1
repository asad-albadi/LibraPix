param(
    [Parameter(Mandatory = $true)]
    [string]$BinaryPath,
    [Parameter(Mandatory = $true)]
    [string]$PfxPath,
    [Parameter(Mandatory = $true)]
    [string]$Password,
    [string]$TimestampUrl = "http://timestamp.digicert.com"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $BinaryPath)) {
    throw "Binary not found: $BinaryPath"
}
if (-not (Test-Path $PfxPath)) {
    throw "PFX not found: $PfxPath"
}

$signtool = Get-Command signtool.exe -ErrorAction Stop

& $signtool.Source sign `
    /fd SHA256 `
    /f $PfxPath `
    /p $Password `
    /tr $TimestampUrl `
    /td SHA256 `
    $BinaryPath

& $signtool.Source verify /pa /v $BinaryPath

Write-Host "Signed binary: $BinaryPath"
