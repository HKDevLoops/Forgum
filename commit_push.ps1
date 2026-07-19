# commit_push.ps1 — Stage, commit, and push to GitHub (PowerShell).
#
# Run locally (git/gh authenticated). PowerShell only (no bash).
#
# Usage (PowerShell, from repo root D:\Projects\Forgum):
#   .\commit_push.ps1
#
# Behavior:
#   1. Adds remote https://github.com/HKDevLoops/Forgum.git (idempotent).
#   2. Ensures .gitignore covers transient/artifact dirs (workflow spec Q9).
#   3. Stages working tree, then EXCLUDES brain/ (historical planning),
#      graphify-out/, .kilo/, and test-renders/ (generated/transient).
#   4. Commits with a Conventional Commit message.
#   5. Pushes to `main`. Never force-pushes.

$ErrorActionPreference = 'Stop'
$repoRoot = $PSScriptRoot
Push-Location $repoRoot

$remoteUrl = 'https://github.com/HKDevLoops/Forgum.git'
$branch    = 'main'

# 1. Ensure remote
$existing = git remote get-url origin 2>$null
if (-not $existing) {
    git remote add origin $remoteUrl
    Write-Host "Added remote origin -> $remoteUrl"
} elseif ($existing -ne $remoteUrl) {
    Write-Warning "origin already points to: $existing"
    Write-Warning "Expected: $remoteUrl  (remote left unchanged; edit if wrong)"
}

# 2. .gitignore pre-flight (workflow spec Q9): append missing artifact
#    patterns so generated dirs never get committed accidentally.
$ignorePatterns = @('graphify-out/', '.kilo/', 'test-renders/')
$gi = Join-Path $repoRoot '.gitignore'
$giText = if (Test-Path $gi) { Get-Content $gi } else { @() }
$dirty = $false
foreach ($p in $ignorePatterns) {
    if (-not ($giText -contains $p)) {
        Add-Content $gi "`n$p"
        $dirty = $true
        Write-Host "Appended to .gitignore: $p"
    }
}
if ($dirty) {
    git add -A
    git reset -q -- brain/ graphify-out/ .kilo/ test-renders/
    git commit -m "chore: ensure .gitignore covers generated artifact dirs" | Out-Null
    if ($LASTEXITCODE -eq 0) { Write-Host "Committed .gitignore update" }
}

# 3. Fetch + stage everything, then exclude transient/historical dirs.
git fetch origin
git add -A
git reset -q -- brain/ graphify-out/ .kilo/ test-renders/
Write-Host "Staged changes (brain/, graphify-out/, .kilo/, test-renders/ excluded):"
git status --short

# 4. Commit
$msg = @'
feat: interactive config TUI, perf/packaging closure, docs revamp, workflow spec

Engine/TUI:
- Break engine<->tui cyclic dependency: move SceneConfig + Shell into
  forgum-platform; forgum-tui now depends only on forgum-platform.
- Feature-gated crates/tui ratatui config menu (forgum config --tui) wired via
  cfg! macro (no platform #[cfg] in engine/src).
- Extend SceneConfig with default_shell, auto_render_on_prompt, color_mode.

Bug fixes surfaced by the test gate:
- framebuffer swap() now copies displayed frame into back (renderer reads
  back-buffer; fixes stale-frame + back/front test semantics).
- Fix completions match (Cmd/PowerShell), config-set moved-value borrow,
  tui draw closure return, clippy metadata/lint cleanups.
- Make golden/visual regression tests hermetic per feature set.

Docs/packaging:
- README revamp, docs/TALES.md, real CONTRIBUTING.md + ADVANCED.md.
- Repo identity HKDEVS/forgum -> HKDevLoops/Forgum (install scripts, psd1,
  ci.yml winget grep, winget manifest filename).
- Sample configs under docs/samples/.

Workflow spec:
- workflows/commit-push.md + NOTES.md: gate on tests, fix failures, push
  only when green; .gitignore pre-flight; specs first-class.
'@

git commit -m $msg
if ($LASTEXITCODE -ne 0) { Write-Error "commit failed"; exit 1 }

# 5. Push to main (never force)
git push -u origin $branch
if ($LASTEXITCODE -ne 0) {
    Write-Error "push failed (diverged or auth). Do NOT force-push. Resolve manually."
    exit 1
}

Write-Host "Done: pushed to origin/$branch"
Pop-Location
