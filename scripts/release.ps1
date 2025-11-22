<#
.SYNOPSIS
  Build MelliLex release, generate latest.json, and upload to GitHub Releases.

.DESCRIPTION
  Full local release workflow:
    1. Reads version from tauri.conf.json
    2. Signs the build with Tauri updater keys
    3. Signs with Windows code signing certificate (if configured in tauri.conf.json)
    4. Builds NSIS installer (.exe) with `tauri build`
    5. Generates latest.json for the Tauri updater
    6. Uploads everything to a GitHub Release via `gh` CLI

  UPDATER SIGNING (required):
    The Tauri updater uses minisign keys to verify update authenticity.
    Run `.\scripts\generate-keys.ps1` once to create them.
    The private key is loaded automatically from ~/.tauri/MelliLex.key
    Or set TAURI_SIGNING_PRIVATE_KEY env var with the key content.

  WINDOWS CODE SIGNING (optional):
    Tauri handles code signing natively via tauri.conf.json > bundle > windows.
    To enable:
      1. Get a code signing certificate (.pfx)
      2. Import it: Import-PfxCertificate -FilePath cert.pfx -CertStoreLocation Cert:\CurrentUser\My -Password (ConvertTo-SecureString -String 'PASSWORD' -Force -AsPlainText)
      3. Open certmgr.msc, find your cert under Personal/Certificates
      4. Copy the Thumbprint (e.g. A1B1A2B2A3B3A4B4A5B5A6B6A7B7A8B8A9B9A0B0)
      5. Set it in tauri.conf.json > bundle > windows > certificateThumbprint
      6. Tauri will sign automatically during build — no extra env vars needed

.PARAMETER SkipBuild
  Skip the Tauri build step (use existing artifacts).

.PARAMETER SkipUpload
  Build only, don't upload to GitHub.

.PARAMETER Draft
  Create the GitHub release as a draft.

.EXAMPLE
  .\scripts\release.ps1
  .\scripts\release.ps1 -SkipUpload
  .\scripts\release.ps1 -Draft
#>

param(
    [switch]$SkipBuild,
    [switch]$SkipUpload,
    [switch]$Draft
)

$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot

# ─── Read version ───────────────────────────────────────────────────────────────
$tauriConf = Get-Content (Join-Path $root "src-tauri" "tauri.conf.json") -Raw | ConvertFrom-Json
$version = $tauriConf.version
$productName = $tauriConf.productName
$tag = "v$version"

Write-Host ""
Write-Host "  ╔══════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "  ║  MelliLex Release v$version              ║" -ForegroundColor Cyan
Write-Host "  ╚══════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# ─── Check prerequisites ────────────────────────────────────────────────────────
$keyPath = Join-Path $env:USERPROFILE ".tauri" "MelliLex.key"

# Updater signing key
if (-not $env:TAURI_SIGNING_PRIVATE_KEY) {
    if (Test-Path $keyPath) {
        Write-Host "  Loading updater signing key from $keyPath" -ForegroundColor Gray
        $env:TAURI_SIGNING_PRIVATE_KEY = Get-Content $keyPath -Raw
    } else {
        Write-Host "  ERROR: TAURI_SIGNING_PRIVATE_KEY not set and no key found at $keyPath" -ForegroundColor Red
        Write-Host "  Run: .\scripts\generate-keys.ps1" -ForegroundColor Yellow
        exit 1
    }
}
Write-Host "  [OK] Updater signing key loaded" -ForegroundColor Green

# Updater signing password (optional, empty string is fine)
if (-not $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD) {
    $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
}

# Windows code signing check (configured in tauri.conf.json, not env vars)
$thumbprint = $tauriConf.bundle.windows.certificateThumbprint
if ($thumbprint) {
    # Verify the certificate is installed in the cert store
    $cert = Get-ChildItem Cert:\CurrentUser\My | Where-Object { $_.Thumbprint -eq $thumbprint }
    if ($cert) {
        Write-Host "  [OK] Windows code signing certificate found: $($cert.Subject)" -ForegroundColor Green
    } else {
        Write-Host "  WARNING: Certificate with thumbprint $thumbprint not found in store" -ForegroundColor Yellow
        Write-Host "         Import it: Import-PfxCertificate -FilePath cert.pfx -CertStoreLocation Cert:\CurrentUser\My ..." -ForegroundColor Yellow
    }
} else {
    Write-Host "  [--] Windows code signing not configured (set certificateThumbprint in tauri.conf.json)" -ForegroundColor Gray
}

# gh CLI for upload
if (-not $SkipUpload) {
    $ghExists = Get-Command gh -ErrorAction SilentlyContinue
    if (-not $ghExists) {
        Write-Host "  ERROR: 'gh' CLI not found. Install from https://cli.github.com" -ForegroundColor Red
        Write-Host "  Or use -SkipUpload to build without uploading." -ForegroundColor Yellow
        exit 1
    }
    Write-Host "  [OK] GitHub CLI found" -ForegroundColor Green
}

# ─── Build ──────────────────────────────────────────────────────────────────────
$nsisDir = Join-Path $root "src-tauri" "target" "release" "bundle" "nsis"

if (-not $SkipBuild) {
    Write-Host ""
    Write-Host "  Building $productName v$version (release mode)..." -ForegroundColor Cyan
    Write-Host ""
    
    Push-Location $root
    try {
        npx tauri build
        if ($LASTEXITCODE -ne 0) { throw "Tauri build failed" }
    } finally {
        Pop-Location
    }
    
    Write-Host ""
    Write-Host "  Build complete!" -ForegroundColor Green
} else {
    Write-Host "  Skipping build (using existing artifacts)" -ForegroundColor Yellow
}

# ─── Locate artifacts ──────────────────────────────────────────────────────────
$nsisExe = Get-ChildItem (Join-Path $nsisDir "*-setup.exe") -ErrorAction SilentlyContinue | Select-Object -First 1
$nsisSig = Get-ChildItem (Join-Path $nsisDir "*-setup.exe.sig") -ErrorAction SilentlyContinue | Select-Object -First 1

if (-not $nsisExe) {
    Write-Host "  ERROR: No NSIS installer found in $nsisDir" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "  Artifacts:" -ForegroundColor Cyan
Write-Host "    Installer: $($nsisExe.Name)" -ForegroundColor White
if ($nsisSig) {
    Write-Host "    Signature: $($nsisSig.Name)" -ForegroundColor White
} else {
    Write-Host "    Signature: NOT FOUND (updater will not work!)" -ForegroundColor Red
    exit 1
}

# ─── Generate latest.json ──────────────────────────────────────────────────────
$signature = Get-Content $nsisSig.FullName -Raw
$downloadUrl = "https://github.com/trrahul/MelliLex/releases/download/$tag/$($nsisExe.Name)"
$pubDate = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

$latestJson = @{
    version = $version
    notes = "MelliLex v$version"
    pub_date = $pubDate
    platforms = @{
        "windows-x86_64" = @{
            signature = $signature.Trim()
            url = $downloadUrl
        }
    }
} | ConvertTo-Json -Depth 4

$latestJsonPath = Join-Path $nsisDir "latest.json"
$latestJson | Set-Content $latestJsonPath -Encoding utf8NoBOM

Write-Host ""
Write-Host "  Generated: latest.json" -ForegroundColor Green
Write-Host "    Version:   $version" -ForegroundColor White
Write-Host "    URL:       $downloadUrl" -ForegroundColor White
Write-Host "    Pub date:  $pubDate" -ForegroundColor White

# ─── Upload to GitHub ───────────────────────────────────────────────────────────
if (-not $SkipUpload) {
    Write-Host ""
    Write-Host "  Creating GitHub release $tag..." -ForegroundColor Cyan
    
    $draftFlag = if ($Draft) { "--draft" } else { "" }
    
    # Create the release
    $releaseArgs = @("release", "create", $tag,
        "--repo", "trrahul/MelliLex",
        "--title", "MelliLex v$version",
        "--notes", "MelliLex v$version release.",
        "--latest"
    )
    if ($Draft) { $releaseArgs += "--draft" }
    
    & gh @releaseArgs 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host "  Release may already exist, trying to upload assets..." -ForegroundColor Yellow
    }
    
    # Upload assets
    $uploadArgs = @("release", "upload", $tag,
        "--repo", "trrahul/MelliLex",
        "--clobber",
        $nsisExe.FullName,
        $nsisSig.FullName,
        $latestJsonPath
    )
    
    & gh @uploadArgs
    if ($LASTEXITCODE -ne 0) {
        Write-Host "  ERROR: Failed to upload assets" -ForegroundColor Red
        exit 1
    }
    
    Write-Host ""
    Write-Host "  ╔══════════════════════════════════════════╗" -ForegroundColor Green
    Write-Host "  ║  Release v$version published!            ║" -ForegroundColor Green
    Write-Host "  ╚══════════════════════════════════════════╝" -ForegroundColor Green
    Write-Host ""
    Write-Host "  https://github.com/trrahul/MelliLex/releases/tag/$tag" -ForegroundColor Cyan
} else {
    Write-Host ""
    Write-Host "  Build artifacts ready in:" -ForegroundColor Green
    Write-Host "    $nsisDir" -ForegroundColor White
    Write-Host ""
    Write-Host "  Files to upload manually:" -ForegroundColor Yellow
    Write-Host "    - $($nsisExe.Name)" -ForegroundColor White
    Write-Host "    - $($nsisSig.Name)" -ForegroundColor White
    Write-Host "    - latest.json" -ForegroundColor White
}

Write-Host ""
