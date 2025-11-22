<#
.SYNOPSIS
  Generates Tauri updater signing keys.

.DESCRIPTION
  Runs `tauri signer generate` to create a minisign keypair for update signing.
  - Private key: ~/.tauri/MelliLex.key
  - Public key:  ~/.tauri/MelliLex.key.pub

  After generation, update the pubkey in tauri.conf.json and set the env var
  TAURI_SIGNING_PRIVATE_KEY before building.

.NOTES
  Only needs to be run ONCE. Keep the private key safe — if you lose it,
  existing users cannot receive updates.
#>

$keyPath = Join-Path $env:USERPROFILE ".tauri" "MelliLex.key"
$keyDir = Split-Path $keyPath

if (Test-Path $keyPath) {
    Write-Host ""
    Write-Host "  Keys already exist at:" -ForegroundColor Yellow
    Write-Host "    Private: $keyPath" -ForegroundColor Cyan
    Write-Host "    Public:  $keyPath.pub" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  Current public key:" -ForegroundColor Yellow
    Get-Content "$keyPath.pub" | Write-Host -ForegroundColor Green
    Write-Host ""
    Write-Host "  To use during builds, run:" -ForegroundColor Yellow
    Write-Host "    `$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content '$keyPath' -Raw" -ForegroundColor White
    Write-Host ""
    exit 0
}

if (-not (Test-Path $keyDir)) {
    New-Item -ItemType Directory -Path $keyDir -Force | Out-Null
}

Write-Host ""
Write-Host "  Generating Tauri updater signing keys..." -ForegroundColor Cyan
Write-Host "  You will be prompted for an optional password." -ForegroundColor Gray
Write-Host ""

npx tauri signer generate -w $keyPath

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "  Keys generated successfully!" -ForegroundColor Green
    Write-Host ""
    Write-Host "  Private key: $keyPath" -ForegroundColor Cyan
    Write-Host "  Public key:  $keyPath.pub" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  IMPORTANT:" -ForegroundColor Red
    Write-Host "    1. Copy the PUBLIC key content into tauri.conf.json > plugins > updater > pubkey" -ForegroundColor Yellow
    Write-Host "    2. Keep the PRIVATE key safe. If lost, users cannot receive updates." -ForegroundColor Yellow
    Write-Host "    3. Never commit the private key to git." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "  Public key content:" -ForegroundColor Yellow
    Get-Content "$keyPath.pub" | Write-Host -ForegroundColor Green
    Write-Host ""
} else {
    Write-Host "  Key generation failed!" -ForegroundColor Red
    exit 1
}
