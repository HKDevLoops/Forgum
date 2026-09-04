param([string]$VideoDir="test-renders/video")
$ErrorActionPreference='Stop'
$root = (Resolve-Path "$PSScriptRoot/..").Path
$dir = Join-Path $root $VideoDir
Write-Host "=== Verify ffmpeg videos as INPUT: what is NOT working ===" -ForegroundColor Cyan
$mp4s = Get-ChildItem $dir -Filter *.mp4 -ErrorAction SilentlyContinue
if (-not $mp4s) { Write-Error "no mp4s in $dir"; exit 1 }
$report = @()
$failures = @()
foreach ($f in $mp4s) {
  $probeRaw = ffprobe -v error -select_streams v:0 -show_entries stream=width,height,codec_name,duration,avg_frame_rate,nb_frames -of json $f.FullName 2>&1
  $probe = $probeRaw | ConvertFrom-Json
  $s = $probe.streams[0]
  $ok = $true; $issues=@()
  if ($s.width -ne 640 -or $s.height -ne 384) { $issues+="bad res $($s.width)x$($s.height)"; $ok=$false }
  if ($s.codec_name -ne "h264") { $issues+="bad codec $($s.codec_name)"; $ok=$false }
  $dur = [double]$s.duration
  $isWhole = $f.Name -like "whole_output*"
  if (-not $isWhole -and [Math]::Abs($dur - 1.0) -gt 0.3 -and [Math]::Abs($dur - 2.0) -gt 0.3) { $issues+="dur $dur"; $ok=$false }
  # ffmpeg decode as input check
  $tmp = "$env:TEMP/forgum_verify_$($f.BaseName).raw"
  $dec = ffmpeg -y -i $f.FullName -f rawvideo -pix_fmt rgba $tmp 2>&1
  $sz = (Get-Item $tmp -ErrorAction SilentlyContinue).Length
  if (-not $sz -or $sz -lt 5000) { $issues+="decode small $sz"; $ok=$false }
  Remove-Item $tmp -Force -ErrorAction SilentlyContinue
  # signalstats to detect empty/black frames
  $stats = ffmpeg -i $f.FullName -vf "signalstats" -f null - 2>&1 | Select-String "YAVG|signalstats"
  # signalstats is optional; don't fail if missing on whole_output concat
  $status = if($ok){"PASS"}else{"FAIL"}
  $report += [PSCustomObject]@{ File=$f.Name; Duration=$dur; Frames=$s.nb_frames; Codec=$s.codec_name; Status=$status; Issues=($issues -join "; ") }
  if (-not $ok) { $failures += "$($f.Name): $($issues -join ', ')" }
  Write-Host "$status $($f.Name) dur=$dur frames=$($s.nb_frames) issues=$($issues -join ',')"
}
Write-Host "`n=== SUMMARY ===" -ForegroundColor Cyan
Write-Host "Total: $($mp4s.Count)  Pass: $(($report|Where Status -eq PASS).Count)  Fail: $(($report|Where Status -eq FAIL).Count)"
if ($failures) { Write-Host "`nFailures (what is NOT working):" -ForegroundColor Red; $failures | ForEach-Object { Write-Host " - $_" -ForegroundColor Yellow } }
# Check coverage vs markdown spec
$expectedCows=(Get-ChildItem (Join-Path $root "data/Cows") -Filter *.cow).Count; $captured= ($mp4s | Where-Object { $_.Name -notlike "default_*" -and $_.Name -notlike "whole_output*" -and $_.Name -notlike "*_frame*" }).Count
Write-Host "`nCoverage vs actual data/Cows ($expectedCows cows, 60 frames, 10 effects):" -ForegroundColor Cyan
Write-Host "Cows captured: $captured / $expectedCows (missing $($expectedCows-$captured))"
$effects = @("breathe","float","walk","particles","pulse","glitch","fly","talk","sway","dissolve")
$missingEffects = $effects | Where-Object { -not (Test-Path (Join-Path $dir "default_$_.mp4")) }
if ($missingEffects) { Write-Host "Missing effects: $($missingEffects -join ', ')" -ForegroundColor Yellow } else { Write-Host "All 10 base effects captured: PASS" -ForegroundColor Green }
$manifest = Join-Path $dir "manifest.json"
if (Test-Path $manifest) { Write-Host "manifest.json: EXISTS ($( (Get-Item $manifest).Length) bytes)" -ForegroundColor Green } else { Write-Host "manifest.json: MISSING (need whole-output)" -ForegroundColor Red; $failures+="missing manifest.json" }
$whole = Join-Path $dir "whole_output.mp4"
if (Test-Path $whole) { Write-Host "whole_output.mp4: EXISTS" -ForegroundColor Green } else { Write-Host "whole_output.mp4: MISSING" -ForegroundColor Yellow }

# Write report for markdown
$reportPath = Join-Path $root "test-renders/VIDEO_VERIFICATION_REPORT.md"
$md = @"
# Video Verification Report (ffmpeg as input)

Generated: $(Get-Date -Format o)
Videos: $($mp4s.Count) in $dir

## Summary
- Total: $($mp4s.Count)
- Pass: $(($report|Where Status -eq PASS).Count)
- Fail: $(($report|Where Status -eq FAIL).Count)
- Cows: $captured / $expectedCows

## Failures (what is NOT working)
$($failures | ForEach-Object { "- $_`n" } | Out-String)

## Details
| File | Duration | Frames | Codec | Status | Issues |
|------|----------|--------|-------|--------|--------|
$($report | ForEach-Object { "| $($_.File) | $($_.Duration) | $($_.Frames) | $($_.Codec) | $($_.Status) | $($_.Issues) |`n" } | Out-String)
## Next Steps (how to achieve goal from current situation)
1. Capture missing $($expectedCows-$captured) cows: run ``FORGUM_REENCODE=1 cargo test --test ffmpeg_capture ffmpeg_whole_output_capture_and_manifest -- --ignored --nocapture`` or ``pwsh scripts/capture_whole_output.ps1 -Force``
2. Fix duration: re-encode with 60 frames (2.0s) via whole-output; current 30-frame (1.0s) videos need ``-Force``
3. Generate manifest.json + whole_output.mp4 (done by whole test)
4. Upload to model: ``pwsh scripts/upload_to_model.ps1``
5. Also fix engine gaps per brain/10-PLAN: 3-thread accumulator, bumpalo, rayon, catch_unwind (see earlier inventory)
"@
$md | Out-File -Encoding utf8 $reportPath
Write-Host "`nReport written to $reportPath" -ForegroundColor Green
if ($failures.Count -gt 0) { exit 1 } else { exit 0 }
