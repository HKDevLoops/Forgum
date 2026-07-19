# Forgum MSI packaging (WiX)

This directory builds a native Windows `.msi` installer for `forgum-engine` using
the [WiX Toolset](https://wixtoolset.org/).

## Build the MSI

Generate a real GUID and drop it into `UpgradeCode` in `forgum.wxs`
(replace `PUT-GUID-HERE`). Then, from a Windows runner with WiX installed:

```powershell
candle.exe -dBinDir=<path-to-bin> forgum.wxs
light.exe -out forgum.msi forgum.wixobj
```

`<path-to-bin>` is the directory containing the built `forgum-engine.exe`.
Alternatively, `cargo wix` can drive the same source.

## Optional Burn bootstrapper (.exe)

To wrap the MSI in a self-contained `.exe` setup (recommended for end users),
author a Burn `Bundle` in a separate `.wxs` that references `forgum.msi` as a
`MsiPackage`. This produces a single `forgum-setup.exe` with a modern UI.

## Portable zip

The per-machine MSI is for installed use. The portable `forgum-0.4.0-windows-*.zip`
is still produced for Scoop and manual/portable usage.
