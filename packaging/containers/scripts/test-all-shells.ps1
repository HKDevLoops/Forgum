# Test script to verify thorough Forgum initialization on Windows
$ErrorActionPreference = 'Stop'

Write-Host "=== 1. Testing forgum-engine binary execution ===" -ForegroundColor Cyan
& forgum-engine.exe --version
& forgum-engine.exe checkhealth

Write-Host "=== 2. Testing PowerShell initialization hook ===" -ForegroundColor Cyan
& forgum-engine.exe init pwsh | Out-String | Invoke-Expression
& forgum-engine.exe say "PowerShell Hook Test: Success" --cow tux

Write-Host "=== 3. Testing Windows PowerShell 5.1 compatibility hook ===" -ForegroundColor Cyan
& forgum-engine.exe init powershell | Out-String | Invoke-Expression

Write-Host "=== 4. Testing Subcommands & Timer ===" -ForegroundColor Cyan
& forgum-engine.exe herd list
& forgum-engine.exe theme list
& forgum-engine.exe timer echo "Windows Container Shell Hook Test Completed"

Write-Host ""
Write-Host "🎉 ALL WINDOWS CONTAINER SHELL INITIALIZATIONS VERIFIED!" -ForegroundColor Green
