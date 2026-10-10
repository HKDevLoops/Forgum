# Forgum Recurring Workflows & Developer Playbook (WORKFLOW_PLAYBOOK.md)

> **Purpose:** Canonical reference guide for repeating tasks across engineering, testing, mascot authoring, terminal validation, and release management.  
> **Mandate Compliance:** Strictly enforces all architectural rules in [`AGENTS.md`](file:///Z:/Projects/Forgum/AGENTS.md).  

---

## Playbook Index

1. [Playbook 1: Full-Stack Quality Gate & Linter Execution](#playbook-1-full-stack-quality-gate--linter-execution)
2. [Playbook 2: Mascot Art & Biological DNA Authoring](#playbook-2-mascot-art--biological-dna-authoring)
3. [Playbook 3: Writing Tests & Adversarial Invariant Audits](#playbook-3-writing-tests--adversarial-invariant-audits)
4. [Playbook 4: 15-Shell Hooks & Completions Verification](#playbook-4-15-shell-hooks--completions-verification)
5. [Playbook 5: Terminal Emulator & DECSTBM Margin Testing](#playbook-5-terminal-emulator--decstbm-margin-testing)
6. [Playbook 6: Git Branch Promotion & PR Automation](#playbook-6-git-branch-promotion--pr-automation)
7. [Playbook 7: Investigating CI / GitHub Action Failures](#playbook-7-investigating-ci--github-action-failures)

---

## Playbook 1: Full-Stack Quality Gate & Linter Execution

Use this workflow before pushing any code to `dev` or opening a PR.

```powershell
# Step 1: Check formatting
cargo fmt --check

# Step 2: If formatting issues exist, auto-format
cargo fmt

# Step 3: Run Clippy across all workspace crates and all targets with zero warnings
cargo clippy --workspace --all-targets -- -D warnings

# Step 4: Run the complete unit, integration, and doc-test suite
cargo test --workspace

# Step 5: Verify resident memory budget (<100MB RAM)
cargo test -p forgum-engine --test memory_budget
```

> [!IMPORTANT]
> If any step fails, investigate and fix the root cause immediately before committing. Never commit code that causes compiler warnings or clippy warnings.

---

## Playbook 2: Mascot Art & Biological DNA Authoring

Use this workflow when creating or modifying mascots in `data/Cows/`.

### Step 1: Mascot Template Rules (`data/Cows/<name>.cow`)
1. **UTF-8 Encoding & Zero Mojibake:**
   - Mascots must strictly contain clean UTF-8 text.
   - Never use characters in the CP437 collision range (`U+2600..=U+263F`).
   - Never introduce literal `Γÿ` or unescaped Perl variables.
2. **Standard Template Escapes:**
   - `$the_cow = <<"EOC";` header and `EOC` footer.
   - Eyes placeholder: `$eyes` (default `oo`).
   - Tongue placeholder: `$tongue` (default `  `).
   - Thoughts glyph placeholder: `$thoughts` (default `\` for speech, `o` for thought).

### Step 2: Biological DNA Registration (`data/Cows/animations.json`)
Every creature profile must declare biological taxonomy and kinematics:
```json
{
  "name": "creature_name",
  "habitat": "aquatic | terrestrial | aerial | amphibious",
  "allowed_effects": ["float", "swim", "drift", "pulse"],
  "base": "Float",
  "speed": 1.0,
  "stride": 1.0,
  "particles": {
    "type": "Bubbles",
    "rate": 3,
    "palette": "aqua"
  },
  "landmarks": {
    "eyes": [4, 8],
    "mouth": [5, 9],
    "tail": [5, 20]
  }
}
```

### Step 3: Verify 132 Mascot Inventory Parity
```powershell
cargo test -p forgum-engine --test adversarial_invariants -- mascot_universe_132_parity_across_all_systems
```

---

## Playbook 3: Writing Tests & Adversarial Invariant Audits

Use this workflow when testing new features or preventing regressions.

### Step 1: Locate Target Test Suite
- CLI argument parsing: [`crates/engine/tests/cli_args.rs`](file:///Z:/Projects/Forgum/crates/engine/tests/cli_args.rs)
- Core adversarial invariants: [`crates/engine/tests/adversarial_invariants.rs`](file:///Z:/Projects/Forgum/crates/engine/tests/adversarial_invariants.rs)
- 2D cartoon kinematics & deformation: [`crates/engine/tests/cartoon_kinematics.rs`](file:///Z:/Projects/Forgum/crates/engine/tests/cartoon_kinematics.rs)
- Terminal capabilities: [`crates/platform/tests/terminal.rs`](file:///Z:/Projects/Forgum/crates/platform/src/terminal.rs)
- TUI interactive navigation: [`crates/tui/src/app.rs`](file:///Z:/Projects/Forgum/crates/tui/src/app.rs)

### Step 2: Run Targeted Test Suite
```powershell
cargo test -p forgum-engine --test cartoon_kinematics
cargo test -p forgum-engine --test cli_args -- speech_flag
```

---

## Playbook 4: 15-Shell Hooks & Completions Verification

Use this workflow when modifying shell hooks, completions, or uninstallation markers.

### Step 1: Supported Shells Matrix
Bash, Zsh, Fish, PowerShell, Pwsh, Nushell, Elvish, Xonsh, Tcsh, Ksh, Ion, Oil, Yash, Cmd, Carapace.

### Step 2: Test All 15 Shell Completions
```powershell
cargo test -p forgum-engine --test completions
```

### Step 3: Test Marker Isolation & Clean Uninstallation
```powershell
cargo test -p forgum-engine --test init_shell
cargo test -p forgum-platform --test uninstaller
```

> [!NOTE]
> All shell modifications must stay strictly between `# >>> forgum >>>` and `# <<< forgum <<<`.

---

## Playbook 5: Terminal Emulator & DECSTBM Margin Testing

Use this workflow to test split-shell top-margin reservation and terminal capabilities.

### Step 1: Test Terminal Capabilities Probe
```powershell
cargo test -p forgum-platform --test terminal
```

### Step 2: Test Live Terminal Split-Margin Execution
```powershell
# Test banner mode (zero recurring latency)
forgum --banner --duration 0

# Test split-scroll reservation in current terminal
forgum render --split-scroll --reserve-rows 10 --duration 3
```

---

## Playbook 6: Git Branch Promotion & PR Automation

Use this workflow when promoting features from `dev` to `main`.

```powershell
# 1. Ensure working tree is clean
git status

# 2. Fetch and rebase dev on remote origin/dev
git fetch origin dev
git rebase origin/dev

# 3. Fast-forward main from dev (when authorized)
git checkout main
git merge --ff-only dev
git push origin main
git checkout dev
```

---

## Playbook 7: Investigating CI / GitHub Action Failures

Use this workflow when monitoring remote GitHub Actions runs.

```powershell
# 1. View recent workflow runs
gh run list --limit 5

# 2. View failed run details
gh run view <run-id>

# 3. View job failure logs directly in terminal
gh run view <run-id> --log-failed

# 4. Rerun failed jobs
gh run rerun <run-id> --failed
```
