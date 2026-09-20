Describe "Forgum ffmpeg animation capture" {
    BeforeAll {
        $root = (Resolve-Path "$PSScriptRoot/..").Path
        $videoDir = Join-Path $root "test-renders/video"
    }
    It "ffmpeg and ffprobe are available" {
        (Get-Command ffmpeg -ErrorAction SilentlyContinue) | Should -Not -Be $null
        (Get-Command ffprobe -ErrorAction SilentlyContinue) | Should -Not -Be $null
    }
    It "cargo ffmpeg_capture tests pass" {
        $out = cargo test --test ffmpeg_capture -- --nocapture 2>&1 | Out-String
        $out | Should -Match "test result: ok"
        $out | Should -Match "0 failed"
    }
    It "video directory contains mp4s after capture" {
        Test-Path $videoDir | Should -Be $true
        $mp4s = Get-ChildItem $videoDir -Filter *.mp4 -ErrorAction SilentlyContinue
        $mp4s.Count | Should -BeGreaterThan 5
        foreach ($f in $mp4s) { $f.Length | Should -BeGreaterThan 512 }
    }
    It "ffprobe reports correct dimensions on sample" {
        $sample = Get-ChildItem (Join-Path $videoDir "*.mp4") | Select-Object -First 1
        if ($null -eq $sample) { Set-ItResult -Skipped -Because "no mp4" ; return }
        $json = ffprobe -v error -select_streams v:0 -show_entries stream=width,height,codec_name -of json $sample.FullName | ConvertFrom-Json
        $json.streams[0].width | Should -Be 640
        $json.streams[0].height | Should -Be 384
        $json.streams[0].codec_name | Should -Be "h264"
    }
    It "ffmpeg can decode video back to rawvideo" {
        $sample = Get-ChildItem (Join-Path $videoDir "*.mp4") | Select-Object -First 1
        if ($null -eq $sample) { Set-ItResult -Skipped -Because "no mp4"; return }
        $tmp = Join-Path $env:TEMP "forgum_ffmpeg_raw.bin"
        ffmpeg -y -i $sample.FullName -f rawvideo -pix_fmt rgba $tmp 2>&1 | Out-Null
        Test-Path $tmp | Should -Be $true
        (Get-Item $tmp).Length | Should -BeGreaterThan 10000
        Remove-Item $tmp -Force -ErrorAction SilentlyContinue
    }
}
