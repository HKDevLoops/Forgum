param(
  [string]$VideoDir="test-renders/video",
  [string]$OutputZip="test-renders/model/forgum-ffmpeg-dataset.zip",
  [string]$HfRepo="",
  [string]$HfToken=""
)
$ErrorActionPreference='Stop'
$root = (Resolve-Path "$PSScriptRoot/..").Path
$dir = Join-Path $root $VideoDir
$outZip = Join-Path $root $OutputZip
$outDir = Split-Path $outZip -Parent
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
Write-Host "=== Upload ffmpeg captures to model ===" -ForegroundColor Cyan
Write-Host "Source: $dir"
if (!(Test-Path $dir)) { Write-Error "no $dir"; exit 1 }
$mp4s = Get-ChildItem $dir -Filter *.mp4
Write-Host "Found $($mp4s.Count) mp4s"
$manifest = Join-Path $dir "manifest.json"
if (!(Test-Path $manifest)) {
  Write-Host "manifest.json missing, generating minimal manifest..." -ForegroundColor Yellow
  $entries = $mp4s | ForEach-Object {
    $probe = ffprobe -v error -select_streams v:0 -show_entries stream=width,height,codec_name,duration -of json $_.FullName | ConvertFrom-Json
    @{ file=$_.Name; size_bytes=$_.Length; codec=$probe.streams[0].codec_name; duration=$probe.streams[0].duration }
  }
  @{ generated_at=(Get-Date -Format o); cows=$mp4s.Count; entries=$entries } | ConvertTo-Json -Depth 5 | Out-File -Encoding utf8 $manifest
}
# Create dataset card
$card = @"
---
license: mit
tags: [forgum, terminal, animation, ffmpeg, cowsay]
task: image-to-video
---

# Forgum ffmpeg captures

Captured via ffmpeg rawvideo -> libx264 (ultrafast, yuv420p) at 640x384 @30fps.

- Cows: $($mp4s.Count)
- Frames per cow: from manifest
- Resolution: 640x384 (80x24 cells, 8x16 px)
- Codec: h264
- Generated: $(Get-Date -Format o)

See manifest.json for per-cow metadata (blake3, duration, fps).

Usage as model input: decode with ``ffmpeg -i <cow>.mp4 -f rawvideo -pix_fmt rgba pipe:1`` or ``ffprobe``.

"@
$card | Out-File -Encoding utf8 (Join-Path $outDir "README.md")
Copy-Item $manifest (Join-Path $outDir "manifest.json") -Force
# Zip videos + manifest + card for model upload
if (Test-Path $outZip) { Remove-Item $outZip -Force }
Write-Host "Zipping to $outZip ..."
Compress-Archive -Path "$dir/*.mp4", $manifest, (Join-Path $outDir "README.md") -DestinationPath $outZip -Force
Write-Host "Zip: $((Get-Item $outZip).Length) bytes, $( (Get-Item $outZip).Length /1MB) MB" -ForegroundColor Green

# Prepare model directory (for local model registry)
$modelDir = Join-Path $root "test-renders/model"
Write-Host "Model dir: $modelDir"
Get-ChildItem $modelDir | Format-Table Name, Length

# Hugging Face upload if repo provided
if ($HfRepo -and $HfToken) {
  Write-Host "Uploading to Hugging Face $HfRepo ..." -ForegroundColor Cyan
  pip install -q huggingface_hub 2>&1 | Out-Null
  $env:HF_TOKEN = $HfToken
  python -c @"
from huggingface_hub import HfApi
api=HfApi()
api.create_repo(repo_id='$HfRepo', repo_type='dataset', exist_ok=True)
api.upload_folder(repo_id='$HfRepo', repo_type='dataset', folder_path='$outDir', commit_message='forgum ffmpeg whole-output')
print('uploaded to $HfRepo')
"@
} elseif ($HfRepo) {
  Write-Host "HF repo set but no token: set HF_TOKEN env or pass -HfToken" -ForegroundColor Yellow
  Write-Host "To upload manually: huggingface-cli upload $HfRepo $outDir --repo-type dataset"
} else {
  Write-Host "No HfRepo set. Local model zip ready for upload." -ForegroundColor Yellow
  Write-Host "Set: pwsh scripts/upload_to_model.ps1 -HfRepo USER/forgum-ffmpeg -HfToken hf_xxx"
}
Write-Host "Done." -ForegroundColor Green
Write-Host "Use videos as model input: ffmpeg -i test-renders/video/default.mp4 -f rawvideo -pix_fmt rgba pipe:1 | <model>"
