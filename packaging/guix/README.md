# ╔══════════════════════════════════════════════════════════════════════════╗
║                🐮  G N U   G U I X   P A C K A G I N G  🐮                 ║
║                                                                        ║
║   Build, test, and containerize Forgum with GNU Guix.                  ║
╚══════════════════════════════════════════════════════════════════════════╝

This directory provides a declarative GNU Guix package specification for `forgum`.

---

## ⚡ Zero-Host-Modification Instant Sandbox Shell

Run Forgum in a 100% isolated, unprivileged container sandbox without installing it to your host or modifying your configuration:

```sh
# Launch an ephemeral Guix container running Forgum
guix shell --container --network -f packaging/guix/forgum.scm -- forgum say "Running in Guix container!" --cow tux

# Run interactive showcase
guix shell --container --network -f packaging/guix/forgum.scm -- forgum showcase

# Run doctor diagnostic
guix shell --container --network -f packaging/guix/forgum.scm -- forgum doctor
```

---

## 📦 Building and Installing

```sh
# Build package locally
guix build -f packaging/guix/forgum.scm

# Install to default profile
guix package -f packaging/guix/forgum.scm
```

---

## 🔄 Upgrading with Guix

When a new version is released:
```sh
# Pull latest Guix channels
guix pull

# Upgrade package
guix package -u forgum
```

---

<div align="center">

*Guix functional package management guarantees reproducible, isolated builds.*

</div>
