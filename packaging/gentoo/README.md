# ╔══════════════════════════════════════════════════════════════════════════╗
║                🐮  G E N T O O   P A C K A G I N G  🐮                 ║
║                                                                        ║
║   Gentoo ebuild and portage overlay instructions for Forgum.           ║
╚══════════════════════════════════════════════════════════════════════════╝

This directory provides Gentoo Portage metadata and ebuild files for `app-misc/forgum`.

---

## 📁 Files

- `forgum-0.4.0.ebuild`: Current release ebuild targeting `target/release/forgum`
- `metadata.xml`: Portage maintainer and package metadata

---

## 🛠️ Testing Locally in a Portage Overlay

To test the ebuild in your local overlay:

```sh
# Create local overlay path
mkdir -p /var/db/repos/localrepo/app-misc/forgum

# Copy ebuild and metadata
cp packaging/gentoo/forgum-0.4.0.ebuild /var/db/repos/localrepo/app-misc/forgum/
cp packaging/gentoo/metadata.xml /var/db/repos/localrepo/app-misc/forgum/

# Generate Manifest
cd /var/db/repos/localrepo/app-misc/forgum
ebuild forgum-0.4.0.ebuild manifest

# Merge package
emerge --ask app-misc/forgum
```

---

<div align="center">

*Compiled with your native CFLAGS and Rust optimizations.*

</div>
