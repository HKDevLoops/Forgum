param(
  [switch]$Force,
  [int]$Frames = 60,
  [switch]$Quick
)
$ErrorActionPreference='Stop'
$root = (Resolve-Path "$PSScriptRoot/..").Path
$videoDir = Join-Path $root "test-renders/video"
Write-Host "=== Forgum Whole-Output ffmpeg Capture ===" -ForegroundColor Cyan
Write-Host "Frames: $Frames  Force: $Force  Quick: $Quick"
if ($Force) { $env:FORGUM_REENCODE="1" }
if ($Quick) { $Frames = 30 }
$env:FORGUM_FFMPEG_WHOLE="1"
# Run whole manifest test
cargo test --test ffmpeg_capture ffmpeg_whole_output_capture_and_manifest -- --ignored --nocapture
if ($LASTEXITCODE -ne 0) { Write-Error "capture failed"; exit 1 }
# Verify manifest
$manifest = Join-Path $videoDir "manifest.json"
if (!(Test-Path $manifest)) { Write-Error "manifest not found"; exit 1 }
$j = Get-Content $manifest | ConvertFrom-Json
Write-Host "Manifest: $($j.cows) cows, $($j.total_frames) frames, $($j.resolution)" -ForegroundColor Green
# Whole output mp4
$whole = Join-Path $videoDir "whole_output.mp4"
if (Test-Path $whole) {
  ffprobe -v error -select_streams v:0 -show_entries stream=codec_name,width,height,duration -of default=noprint_wrappers=1 $whole
  Write-Host "whole_output.mp4 ready: $((Get-Item $whole).Length) bytes" -ForegroundColor Green
}
Write-Host "Done. Videos in $videoDir"
Get-ChildItem $videoDir -Filter *.mp4 | Measure-Object | ForEach-Object { Write-Host "Total mp4s: $($_.Count)" }
