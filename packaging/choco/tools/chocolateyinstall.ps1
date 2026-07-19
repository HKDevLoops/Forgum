# NOTE: PLACEHOLDER_HASH is replaced by the maintainer at release time with the
# real sha256 of the downloaded zip. `choco pack` runs in CI on windows.

$ErrorActionPreference = 'Stop'
$packageName = 'forgum'
$version     = '0.4.0'
$url64       = "https://github.com/HKDevLoops/Forgum/releases/download/v$version/forgum-$version-windows-x64.zip"
$urlArm64    = "https://github.com/HKDevLoops/Forgum/releases/download/v$version/forgum-$version-windows-arm64.zip"
$toolsDir    = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$arch = if ([Environment]::Is64BitOperatingSystem) {
          if ($env:PROCESSOR_ARCHITECTURE -eq 'ARM64') { 'arm64' } else { 'x64' }
        } else { 'x64' }
$url = if ($arch -eq 'arm64') { $urlArm64 } else { $url64 }
$packageArgs = @{
  packageName    = $packageName
  unzipLocation  = $toolsDir
  url            = $url
  checksum       = 'PLACEHOLDER_HASH'
  checksumType   = 'sha256'
}
Install-ChocolateyZipPackage @packageArgs
