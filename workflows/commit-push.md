# Workflow: commit-push

> Source of truth for the "commit, verify tests, fix-if-needed, push" loop.
> Driven by NOTES.md (repo = HKDevLoops/Forgum @ main; gate = fast-gate + deep-gate).

## Purpose
A recurring loop: turn local work into a pushed, test-green commit on `origin/main`.
If the test gate fails, the workflow investigates, researches, and fixes the issue
(removing latent bugs too) — and only pushes when green. It never force-pushes and
never silently pushes broken code.

## Trigger
Two triggers fire this workflow (Q1 = A + B):
1. **Event (primary):** a commit/push attempt.
   - Mechanism (Q2 = A + B): a local wrapper command `forgum-wf commit-push` AND a
     git `pre-push` hook. Both run the gate locally *before* the remote is touched.
   - If invoked as a plain `git commit`/`git push` without the wrapper, the pre-push
     hook still enforces the gate; the wrapper adds nicer briefing/reporting.
2. **Schedule (secondary):** nightly at a quiet hour (default 03:00 local) — see Schedule.

## Inputs
- A set of staged (and for the wrapper, optionally unstaged) changes.
- Optional flags:
  - `--deep` — also run the deep-gate (feature-flag variants).
  - `--with-specs` — (always on by default; see Specs) include `workflows/` + `NOTES.md`.
  - `--no-fix` — checkpoint on first failure instead of attempting fixes (Q4-B behavior).

## Steps (each run)

### 0. Pre-flight
- Resolve `repo`/`push-target` from NOTES.md: `origin/main` (HKDevLoops/Forgum).
  Add remote `https://github.com/HKDevLoops/Forgum.git` only if missing.
- **`.gitignore` check (Q9 = A):** verify `.gitignore` covers known artifact patterns:
  `/target/`, `test-renders/`, `graphify-out/`, `*.psd1`, `*.psm1`, `Private/`,
  `Public/`, `.kilo/`. For any generated/staged file NOT covered, append the missing
  pattern(s) to `.gitignore`. Bundle the `.gitignore` change into the same commit and
  note it in the brief. (Transient generated markdown e.g. `*.report.tmp.md` is ignored.)
- Confirm no large/unwanted dir (`target/`, `graphify-out/`, `.kilo/`, `packaging/`,
  `Private/`, `Public/`) is staged; if so, `git reset -q -- <path>` and report.

### 1. Stage (wrapper only)
- Stage the intended paths. For the nightly schedule, stage NOTHING uncommitted —
  it only pushes work that is already committed but unpushed (Q8 = B: never auto-commit WIP).
- Specs (Q10 = A): ensure `workflows/*.md` and `NOTES.md` are included in the commit
  when they changed.

### 2. Run the gate (Q3 = all)
- **fast-gate** (always):
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --check`
  - cfg-grep: assert `crates/engine/src/` has no platform-targeting `#[cfg]`
    (expect `OK: no platform cfg in engine source.`)
  - `Invoke-Pester Tests/`
- **deep-gate** (if `--deep` or configured): also
  - `cargo test -p forgum-engine --features forgum-engine/synchronized-update`
  - `cargo test -p forgum-platform --features forgum-platform/sixel`
- **minimal subset** (documented for quick iterations): `cargo test --workspace` + `cargo clippy`.

### 3. On failure — fix ladder (Q4 = all, Q5 = A + B)
Escalate through attempts; hard cap = **5 attempts OR 10 minutes**, whichever first.
Stop and checkpoint (Step 4) when the cap is hit or a fix is unsafe to auto-apply.
- Attempts 1–2 (mechanical, autonomous): `cargo fmt`, `cargo clippy --fix`,
  `cargo audit fix`, regenerate any derived assets.
- Attempts 3–5 (investigate + research + robust fix): read the failure; research the
  issue (docs/web if needed); apply a fix that **resolves the issue AND removes
  potential latent bugs** (not a band-aid). Re-run the gate.
- The fix ladder never edits `main` history on the remote; all fixes are local commits.

### 4. Checkpoint (Q6 = A, Q7 = A)
When the cap is reached without green (or `--no-fix` on first failure):
- Write a **brief** to `workflows/reports/commit-push-<timestamp>.md` containing:
  - **What:** the local commit(s)/diffstat produced so far.
  - **Why:** which gate step failed + the hypothesis for the root cause.
  - **Link:** the report file path + the current local commit/branch ref.
  - The name of the **driving spec** (`workflows/commit-push.md`).
- Print the brief to terminal (secondary).
- Do NOT push. Wait for the user to approve or take over.

### 5. Commit + push (only when gate is green)
- Commit with a Conventional Commit message; bundle any `.gitignore` change and spec
  changes noted in Step 0/1.
- `git push -u origin main`. **Never `--force` / `--force-with-lease`.**
- If remote rejects (divergence), STOP — do not rebase/force. Brief the user.
- On success, append the result (commit hash + push URL) to the run's report file.

## Schedule (Q8 = B)
- Nightly cron (default 03:00 local): run Steps 0–2 + fix ladder (3) on the
  **already-committed-but-unpushed** state only.
  - If green → push to `origin/main`.
  - If not green after the fix ladder → write a brief (Step 4), do NOT push, wait.
- Never auto-commits uncommitted WIP.

## Specs handling (Q10 = A)
- `workflows/*.md` and `NOTES.md` are first-class: always part of the commit when changed.
- Transient generated markdown (e.g. `graphify-out/`, `*.report.tmp.md`) is gitignored.
- Every brief names the exact spec file that drove the run.

## Definition of done
An implementer agent can build this without asking a question:
- trigger = event (wrapper + pre-push hook) + nightly schedule ✅
- gate = fast-gate (mirrors CI) + optional deep-gate + documented minimal subset ✅
- on-fail = mechanical → investigate/research → robust fix removing latent bugs → checkpoint ✅
- cap = 5 attempts OR 10 min, then brief ✅
- brief = what/why/link, local report file + terminal ✅
- .gitignore pre-flight auto-append, bundled + surfaced ✅
- specs first-class, transient markdown ignored ✅
- push-target = origin/main, no force-push ✅
