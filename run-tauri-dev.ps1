$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User") + ";$env:APPDATA\npm"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptDir

Write-Host "Starting Tauri development server..." -ForegroundColor Green
Write-Host ""

& "$env:APPDATA\npm\pnpm.cmd" tauri dev
