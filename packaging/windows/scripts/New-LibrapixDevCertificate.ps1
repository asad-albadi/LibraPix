param(
    [string]$Subject = "CN=Asad",
    [string]$CertOutputPath = "packaging/windows/certs/librapix-dev.pfx",
    [string]$Password = "change-me",
    [switch]$TrustLocally
)

$ErrorActionPreference = "Stop"

$certDir = Split-Path -Parent $CertOutputPath
if (-not (Test-Path $certDir)) {
    New-Item -ItemType Directory -Path $certDir | Out-Null
}

$securePassword = ConvertTo-SecureString -String $Password -Force -AsPlainText

$cert = New-SelfSignedCertificate `
    -Type Custom `
    -Subject $Subject `
    -KeyAlgorithm RSA `
    -KeyLength 2048 `
    -HashAlgorithm SHA256 `
    -KeyUsage DigitalSignature `
    -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3") `
    -CertStoreLocation "Cert:\CurrentUser\My"

Export-PfxCertificate `
    -Cert $cert `
    -FilePath $CertOutputPath `
    -Password $securePassword | Out-Null

if ($TrustLocally.IsPresent) {
    Import-Certificate -FilePath $CertOutputPath -CertStoreLocation "Cert:\CurrentUser\TrustedPeople" | Out-Null
}

Write-Host "Created development signing certificate:"
Write-Host "  Subject: $Subject"
Write-Host "  PFX:     $CertOutputPath"
