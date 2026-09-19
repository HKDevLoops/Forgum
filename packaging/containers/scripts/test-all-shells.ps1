# Test script to verify thorough Forgum initialization on Windows
$ErrorActionPreference = 'Stop'

Write-Host "=== 1. Testing forgum binary execution ===" -ForegroundColor Cyan
& forgum.exe --version
& forgum.exe doctor
& forgum.exe checkhealth

Write-Host "=== 2. Testing PowerShell initialization hook ===" -ForegroundColor Cyan
& forgum.exe init pwsh | Out-String | Invoke-Expression
& forgum.exe --cow tux render "PowerShell Hook Test: Success"

Write-Host "=== 3. Testing Windows PowerShell 5.1 compatibility hook ===" -ForegroundColor Cyan
& forgum.exe init powershell | Out-String | Invoke-Expression

Write-Host "=== 4. Testing Subcommands & Timer ===" -ForegroundColor Cyan
& forgum.exe herd list
& forgum.exe theme list
& forgum.exe list animals
& forgum.exe timer echo "Windows Container Shell Hook Test Completed"

Write-Host ""
Write-Host "🎉 ALL WINDOWS CONTAINER SHELL INITIALIZATIONS VERIFIED!" -ForegroundColor Green
