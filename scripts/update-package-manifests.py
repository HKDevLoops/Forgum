#!/usr/bin/env python3
"""
Automated Package Manager Manifest Synchronizer.

Synchronizes version numbers and exact architecture-specific SHA-256 hashes
from the master SHA256SUMS release manifest into:
  1. Scoop manifest: packaging/scoop/forgum.json
  2. WinGet manifest: packaging/winget/HKDevLoops.Forgum.yaml
  3. Homebrew formula: packaging/homebrew/forgum.rb
  4. Arch Linux PKGBUILD: packaging/pacman/PKGBUILD
"""

import argparse
import json
import os
import re
import sys
from pathlib import Path


def parse_checksums(manifest_path: Path) -> dict:
    checksums = {}
    if not manifest_path.is_file():
        print(f"Warning: Manifest file '{manifest_path}' does not exist.", file=sys.stderr)
        return checksums

    content = manifest_path.read_text(encoding="utf-8", errors="replace")
    for line in content.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split(maxsplit=1)
        if len(parts) == 2:
            sha, filename = parts[0], parts[1].lstrip("*")
            checksums[filename] = sha
    return checksums


def find_hash(checksums: dict, pattern: str) -> str:
    regex = re.compile(pattern)
    for fname, sha in checksums.items():
        if regex.search(fname):
            return sha
    return ""


def update_scoop(scoop_file: Path, version: str, checksums: dict, dry_run: bool) -> bool:
    if not scoop_file.is_file():
        print(f"Skipping Scoop (file not found: {scoop_file})")
        return False

    with open(scoop_file, "r", encoding="utf-8") as f:
        data = json.load(f)

    data["version"] = version

    # Find architecture hashes
    x64_hash = find_hash(checksums, r"windows-x64\.zip") or find_hash(checksums, r"x86_64-pc-windows-msvc.*\.zip")
    arm64_hash = find_hash(checksums, r"windows-arm64\.zip") or find_hash(checksums, r"aarch64-pc-windows-msvc.*\.zip")

    if "architecture" in data:
        if "64bit" in data["architecture"]:
            url = f"https://github.com/HKDevLoops/Forgum/releases/download/v{version}/forgum-{version}-windows-x64.zip"
            data["architecture"]["64bit"]["url"] = url
            if x64_hash:
                data["architecture"]["64bit"]["hash"] = f"sha256:{x64_hash}"
        if "arm64" in data["architecture"]:
            url = f"https://github.com/HKDevLoops/Forgum/releases/download/v{version}/forgum-{version}-windows-arm64.zip"
            data["architecture"]["arm64"]["url"] = url
            if arm64_hash:
                data["architecture"]["arm64"]["hash"] = f"sha256:{arm64_hash}"

    new_content = json.dumps(data, indent=4) + "\n"
    if dry_run:
        print(f"[Dry-run] Would update {scoop_file}")
    else:
        scoop_file.write_text(new_content, encoding="utf-8", newline="\n")
        print(f"✓ Updated Scoop manifest at {scoop_file}")
    return True


def update_winget(winget_file: Path, version: str, checksums: dict, dry_run: bool) -> bool:
    if not winget_file.is_file():
        print(f"Skipping WinGet (file not found: {winget_file})")
        return False

    content = winget_file.read_text(encoding="utf-8")
    content = re.sub(r"PackageVersion:\s+[^\r\n]+", f"PackageVersion: {version}", content)

    # Replace x64 and arm64 URLs and Hashes
    x64_hash = find_hash(checksums, r"windows-x64\.zip") or find_hash(checksums, r"x86_64-pc-windows-msvc.*\.zip")
    arm64_hash = find_hash(checksums, r"windows-arm64\.zip") or find_hash(checksums, r"aarch64-pc-windows-msvc.*\.zip")

    content = re.sub(
        r"InstallerUrl:\s+https://github\.com/HKDevLoops/Forgum/releases/download/[^/]+/forgum-[^/]+-windows-x64\.zip",
        f"InstallerUrl: https://github.com/HKDevLoops/Forgum/releases/download/v{version}/forgum-{version}-windows-x64.zip",
        content,
    )
    content = re.sub(
        r"InstallerUrl:\s+https://github\.com/HKDevLoops/Forgum/releases/download/[^/]+/forgum-[^/]+-windows-arm64\.zip",
        f"InstallerUrl: https://github.com/HKDevLoops/Forgum/releases/download/v{version}/forgum-{version}-windows-arm64.zip",
        content,
    )

    if x64_hash:
        content = re.sub(r"(Architecture:\s+x64[\s\S]*?InstallerSha256:\s+)[^\r\n]+", r"\g<1>" + x64_hash, content)
    if arm64_hash:
        content = re.sub(r"(Architecture:\s+arm64[\s\S]*?InstallerSha256:\s+)[^\r\n]+", r"\g<1>" + arm64_hash, content)

    if dry_run:
        print(f"[Dry-run] Would update {winget_file}")
    else:
        winget_file.write_text(content, encoding="utf-8", newline="\n")
        print(f"✓ Updated WinGet manifest at {winget_file}")
    return True


def update_homebrew(brew_file: Path, version: str, checksums: dict, dry_run: bool) -> bool:
    if not brew_file.is_file():
        print(f"Skipping Homebrew (file not found: {brew_file})")
        return False

    content = brew_file.read_text(encoding="utf-8")
    content = re.sub(r'version\s+"[^"]+"', f'version "{version}"', content)

    mac_intel_hash = find_hash(checksums, r"x86_64-apple-darwin\.tar\.gz")
    mac_arm_hash = find_hash(checksums, r"aarch64-apple-darwin\.tar\.gz")
    linux_intel_hash = find_hash(checksums, r"x86_64-unknown-linux-gnu\.tar\.gz")
    linux_arm_hash = find_hash(checksums, r"aarch64-unknown-linux-gnu\.tar\.gz")

    if mac_intel_hash:
        content = re.sub(
            r'(on_macos[\s\S]*?on_intel[\s\S]*?sha256\s+)"[^"]+"',
            r'\g<1>"' + mac_intel_hash + '"',
            content,
        )
    if mac_arm_hash:
        content = re.sub(
            r'(on_macos[\s\S]*?on_arm[\s\S]*?sha256\s+)"[^"]+"',
            r'\g<1>"' + mac_arm_hash + '"',
            content,
        )
    if linux_intel_hash:
        content = re.sub(
            r'(on_linux[\s\S]*?on_intel[\s\S]*?sha256\s+)"[^"]+"',
            r'\g<1>"' + linux_intel_hash + '"',
            content,
        )
    if linux_arm_hash:
        content = re.sub(
            r'(on_linux[\s\S]*?on_arm[\s\S]*?sha256\s+)"[^"]+"',
            r'\g<1>"' + linux_arm_hash + '"',
            content,
        )

    if dry_run:
        print(f"[Dry-run] Would update {brew_file}")
    else:
        brew_file.write_text(content, encoding="utf-8", newline="\n")
        print(f"✓ Updated Homebrew formula at {brew_file}")
    return True


def update_pacman(pkgbuild_file: Path, version: str, checksums: dict, dry_run: bool) -> bool:
    if not pkgbuild_file.is_file():
        print(f"Skipping PKGBUILD (file not found: {pkgbuild_file})")
        return False

    clean_ver = version.replace("-", ".")
    content = pkgbuild_file.read_text(encoding="utf-8")
    content = re.sub(r"pkgver=[^\r\n]+", f"pkgver={clean_ver}", content)

    src_hash = find_hash(checksums, r"forgum-.*\.tar\.gz")
    if src_hash:
        content = re.sub(r"sha256sums=\('[^']+'\)", f"sha256sums=('{src_hash}')", content)

    if dry_run:
        print(f"[Dry-run] Would update {pkgbuild_file}")
    else:
        pkgbuild_file.write_text(content, encoding="utf-8", newline="\n")
        print(f"✓ Updated Arch Linux PKGBUILD at {pkgbuild_file}")
    return True


def main():
    parser = argparse.ArgumentParser(description="Update package manager manifests from SHA256SUMS")
    parser.add_argument("--manifest", default="SHA256SUMS", help="Path to SHA256SUMS file")
    parser.add_argument("--version", required=True, help="Release version (e.g. 0.0.8-beta)")
    parser.add_argument("--dry-run", action="store_true", help="Preview changes without writing")
    parser.add_argument("--root", default=".", help="Root of Forgum repository")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    manifest_path = Path(args.manifest)
    if not manifest_path.is_absolute():
        manifest_path = root / manifest_path

    checksums = parse_checksums(manifest_path)
    print(f">> Loaded {len(checksums)} checksum records from {manifest_path}")

    update_scoop(root / "packaging/scoop/forgum.json", args.version, checksums, args.dry_run)
    update_winget(root / "packaging/winget/HKDevLoops.Forgum.yaml", args.version, checksums, args.dry_run)
    update_homebrew(root / "packaging/homebrew/forgum.rb", args.version, checksums, args.dry_run)
    update_pacman(root / "packaging/pacman/PKGBUILD", args.version, checksums, args.dry_run)
    print(">> All package manager manifests synchronized.")


if __name__ == "__main__":
    main()
