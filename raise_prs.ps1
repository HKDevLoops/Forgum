# Forgum — raise 10 INDEPENDENT PRs (1 engine refactor + 9 per-package-manager).
# Run from repo root: D:\Projects\Forgum  (PowerShell). Needs: git, cargo, gh (authed).
# Each package-manager PR contains ONLY its own packaging/<mgr>/ directory.
# The engine/refactor PR contains everything else (shared source, tests, CI, README).
# Packaging PRs are independent of each other (no file overlap) and can merge after PR0.

$ErrorActionPreference = 'Stop'
$GITHUB_OWNER = "HKDevLoops"   # <-- change if your GitHub username differs
$REMOTE_URL    = "git@github.com:$GITHUB_OWNER/Forgum.git"

# 0) Remote
if (-not (git remote get-url origin 2>$null)) {
    Write-Host "Adding remote origin -> $REMOTE_URL"
    git remote add origin $REMOTE_URL
}

# 1) Full gate (must be green)
Write-Host "== cargo test ==";      cargo test --workspace
Write-Host "== cargo clippy ==";    cargo clippy --workspace --all-targets -- -D warnings
Write-Host "== cargo fmt ==";       cargo fmt --check
Write-Host "== feat sync ==";       cargo test --workspace --features forgum-engine/synchronized-update
Write-Host "== feat sixel ==";      cargo test -p forgum-platform --features forgum-platform/sixel
Write-Host "== Pester ==";          Invoke-Pester -Path Tests/ -Output Detailed

# 2) PR0 — Engine refactor + shared (everything EXCEPT packaging/)
git checkout -b pr0-engine-refactor
git add -A
# Unstage packaging so PR0 doesn't carry it
git reset -q HEAD -- packaging/
git commit -m "perf(engine/platform): dirty-tracking framebuffer, zero-alloc renderer, off-by-default sync, cross-platform CI matrix, daemon leak soak, Cmd/PowerShell hooks, terminal capability probe"
git push -u origin pr0-engine-refactor
gh pr create --title "perf/leak/modern-render + shell + capability probe" `
              --body-file PR1_ENGINE_REFACTOR.md --base main

# 3) 9 independent packaging PRs (each ONLY its own directory)
$pkgs = @(
  @{name="winget";  dir="packaging/winget";  title="packaging(winget): HKDevLoops.Forgum manifest";    body="Adds the winget manifest (packaging/winget/HKDevLoops.Forgum.yaml) for Forgum 0.4.0."},
  @{name="scoop";   dir="packaging/scoop";   title="packaging(scoop): forgum.json";                    body="Adds the scoop manifest (packaging/scoop/forgum.json) with x64+arm64 zips and autoupdate."},
  @{name="choco";   dir="packaging/choco";   title="packaging(choco): nuspec + install script";        body="Adds the Chocolatey package (packaging/choco/forgum.nuspec + tools/chocolateyinstall.ps1)."},
  @{name="nix";     dir="packaging/nix";     title="packaging(nix): flake + module";                   body="Adds Nix flake/module (packaging/nix/{flake,package,module}.nix)."},
  @{name="deb";     dir="packaging/deb";     title="packaging(deb): DEBIAN control + scripts";         body="Adds Debian packaging (packaging/deb/DEBIAN/{control,postinst} + build-deb.sh)."},
  @{name="rpm";     dir="packaging/rpm";     title="packaging(rpm): spec + build script";              body="Adds RPM packaging (packaging/rpm/forgum.spec + build-rpm.sh)."},
  @{name="msi";     dir="packaging/windows"; title="packaging(msi/wix): forgum.wxs";                   body="Adds the WiX/MSI source (packaging/windows/forgum.wxs)."},
  @{name="pacman";  dir="packaging/pacman";  title="packaging(pacman): PKGBUILD";                       body="Adds the Arch PKGBUILD (packaging/pacman/PKGBUILD + build-pacman.sh)."},
  @{name="gentoo";  dir="packaging/gentoo";  title="packaging(gentoo): ebuild";                        body="Adds the Gentoo ebuild (packaging/gentoo/forgum-0.4.0.ebuild + metadata.xml)."}
)

foreach ($p in $pkgs) {
    git checkout main
    git checkout -b "pr-pkg-$($p.name)"
    git add -A
    # Keep ONLY this manager's directory
    git reset -q HEAD -- packaging/
    git add "$($p.dir)/"
    git commit -m $p.title
    git push -u origin "pr-pkg-$($p.name)"
    gh pr create --title $p.title --body $p.body --base main
    Write-Host "Raised PR for $($p.name)"
}

Write-Host "DONE: 1 engine PR + 9 package-manager PRs raised."
