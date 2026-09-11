# Uninstalling Forgum

Remove every trace of Forgum from your system. Run only the sections that match how you installed it.

---

## Package Managers

### scoop (Windows)

```powershell
scoop uninstall forgum
```

This removes the binary and start menu shortcut. Then clean up leftover scoop data:

```powershell
# Remove scoop's Forgum bucket (if you added one)
scoop bucket remove forgum-test 2>$null

# Remove cached artifacts (optional — frees space)
scoop cache remove forgum
```

---

### winget (Windows)

```powershell
winget uninstall HKDevLoops.Forgum
```

Or open **Settings → Apps → Installed apps**, find `Forgum`, and uninstall from there.

---

### Chocolatey (Windows)

```powershell
choco uninstall forgum -removeall
```

This removes the package and all files. If you installed the `.nupkg` locally:

```powershell
choco uninstall forgum -removeall --skip-scripts
```

---

### Homebrew (macOS / Linux)

```sh
brew uninstall forgum
brew cleanup  # optional — removes old bottle caches
```

If you installed the tap separately:

```sh
brew untap HKDevLoops/forgum
```

---

### apt / dpkg (Ubuntu / Kali Linux / Debian)

```sh
sudo apt remove --purge forgum
# or via dpkg:
sudo dpkg -P forgum
```

---

### dnf / rpm (Fedora / RHEL)

```sh
sudo dnf remove forgum
# or via rpm:
sudo rpm -e forgum
```

---

### zypper / rpm (openSUSE)

```sh
sudo zypper remove forgum
# or via rpm:
sudo rpm -e forgum
```

---

### pacman (Arch Linux / AUR)

```sh
# If using an AUR helper (yay, paru, etc.):
yay -Rns forgum

# Or manually:
sudo pacman -Rns forgum
```

The PKGBUILD pulls from GitHub source — no separate cleanup needed.

---

### Nix

**If you used `nix run` or `nix build`:**
Forgum wasn't installed — it ran from a temporary store path. Nothing to uninstall.

**If you added it to your NixOS / home-manager config:**

```nix
# In your configuration.nix or home.nix:
programs.forgum.enable = false
# Remove the import line if you no longer need it:
# imports = [ (import /path/to/flake.nix).nixosModules.forgum ];
```

Then rebuild:

```sh
# NixOS
sudo nixos-rebuild switch

# home-manager
home-manager switch
```

**If you installed via `nix-env`:**

```sh
nix-env -e forgum
```

---

### Gentoo

```sh
# From your overlay:
sudo ebuild /var/db/repos/<overlay>/sys-apps/forgum/forgum-0.4.0.ebuild merge --unmerge

# Or using portage directly:
sudo emerge --deselect sys-apps/forgum
sudo emerge --unmerge sys-apps/forgum
```

---

## Shell Hook Removal

The shell hook modifies your shell's prompt to render Forgum on each command. Remove it to restore your original prompt.

### PowerShell 7

Open your profile and remove the Forgum block:

```powershell
code $PROFILE
```

Look for this block (typically near the `# Modules` or `# Forgum Shell Hook` section) and delete it entirely:

```powershell
# ---------- Forgum Shell Hook ----------
if (Get-Command forgum -ErrorAction Ignore) {
    Invoke-Expression (forgum init pwsh 2>$null)
}
```

### PowerShell 5.1 (Windows PowerShell)

```powershell
notepad $PROFILE
```

Remove any lines referencing `forgum`, `forgum init`, or `Forgum`.

### bash

```sh
# Edit your bashrc (~/.bashrc or ~/.bash_profile)
nano ~/.bashrc
```

Look for and remove:

```sh
# Look for lines like these:
eval "$(forgum init bash)"
```

### zsh

```sh
nano ~/.zshrc
```

Remove:

```sh
eval "$(forgum init zsh)"
```

### fish

```sh
nano ~/.config/fish/config.fish
```

Remove:

```fish
forgum init fish | source
```

### cmd (Windows Command Prompt)

The CMD integration is manual — if you added a registry AutoRun or a `.cmd` startup file, remove it:

```cmd
reg delete "HKCU\Software\Microsoft\Command Processor" /v AutoRun /f 2>nul
```

---

## Config & Data Cleanup

Forgum stores config, daemon state, and optional custom data. Remove these to wipe Forgum completely:

### Windows

```powershell
# Remove config and data
Remove-Item -Recurse -Force "$env:APPDATA\Forgum"

# Remove daemon state (if daemon was running)
Remove-Item -Recurse -Force "$env:TEMP\Forgum"

# Remove PowerShell module (if installed)
Remove-Module Forgum -ErrorAction SilentlyContinue
$modulePath = "$env:PSModulePath" -split ';' -match [regex]::escape($env:APPDATA)
Remove-Item -Recurse -Force "$env:APPDATA\PowerShell\Modules\Forgum" 2>$null
```

### macOS / Linux

```sh
# Config and data
rm -rf ~/.config/forgum

# Daemon state
rm -rf /tmp/forgum

# PowerShell module (if applicable)
rm -rf ~/.local/share/powershell/Modules/Forgum
```

---

## PATH Cleanup (Manual / Portable Install)

If you installed Forgum manually (downloaded zip, no package manager) and added it to PATH:

### Windows (PowerShell)

```powershell
# Find the PATH entry
$path = [Environment]::GetEnvironmentVariable("Path", "User")
$path -split ';' | Select-String forgum

# Remove it (replace with actual path if different)
$newPath = ($path -split ';' | Where-Object { $_ -notmatch 'forgum' }) -join ';'
[Environment]::SetEnvironmentVariable("Path", $newPath, "User")
```

Restart your terminal for changes to take effect.

### macOS / Linux

```sh
# Find forgum in your shell profile
grep -l 'forgum' ~/.bashrc ~/.bash_profile ~/.zshrc ~/.profile 2>/dev/null

# Edit each file and remove lines that add forgum to PATH, e.g.:
#   export PATH="$HOME/.local/bin:$PATH"
#   or the specific forgum entry
```

---

## Complete Wipe (Everything)

Run all of the above in one shot:

```powershell
# Package manager removal
scoop uninstall forgum 2>$null
winget uninstall HKDevLoops.Forgum 2>$null
choco uninstall forgum -removeall 2>$null

# Config & data
Remove-Item -Recurse -Force "$env:APPDATA\Forgum" 2>$null
Remove-Item -Recurse -Force "$env:TEMP\Forgum" 2>$null
Remove-Module Forgum -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force "$env:APPDATA\PowerShell\Modules\Forgum" 2>$null

# PATH cleanup
$path = [Environment]::GetEnvironmentVariable("Path", "User")
$newPath = ($path -split ';' | Where-Object { $_ -notmatch 'forgum' }) -join ';'
[Environment]::SetEnvironmentVariable("Path", $newPath, "User")

# Shell profile cleanup
$PROFILE_DIR = Split-Path $PROFILE -Parent
Get-ChildItem "$PROFILE_DIR\*" -Filter "*.ps1" | ForEach-Object {
    $content = Get-Content $_.FullName -Raw
    if ($content -match 'forgum') {
        $new = $content -replace '(?m)^.*forgum.*$', ''
        Set-Content -Path $_.FullName -Value $new
    }
}

Write-Host "Forgum fully removed. Restart your terminal." -ForegroundColor Green
```

> **Note:** The shell profile cleanup uses a regex — it removes every line containing `forgum`. Review the changes with `git diff` or your editor before saving if you have other customizations in the same file.

---

## Still Having Issues?

If anything persists after uninstalling:

```powershell
# Find any remaining forgum files
Get-ChildItem C:\ -Recurse -Filter "*forgum*" -ErrorAction SilentlyContinue 2>$null | Select-Object FullName

# Check for leftover services
Get-Service | Where-Object { $_.DisplayName -match 'forgum' }

# Check startup items
Get-CimInstance Win32_StartupCommand | Where-Object { $_.Command -match 'forgum' }
```

File an issue at [github.com/HKDevLoops/Forgum/issues](https://github.com/HKDevLoops/Forgum/issues).