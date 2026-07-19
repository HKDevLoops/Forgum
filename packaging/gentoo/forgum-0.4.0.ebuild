# Copyright 2024 Gentoo Authors
# Distributed under the terms of the GNU General Public License v2
# Run `cargo ebuild` / `cargo-ebuild` to populate CRATES and SRC_URI checksums.
# Submitted to ::guru via maintainer PR.
EAPI=8
CRATES=""
inherit cargo
DESCRIPTION="Cross-platform cowsay+fortune+lolcat with a Rust ANSI animation engine"
HOMEPAGE="https://github.com/HKDevLoops/Forgum"
SRC_URI="https://github.com/HKDevLoops/Forgum/archive/refs/tags/v${PV}.tar.gz -> ${P}.tar.gz"
LICENSE="MIT"
SLOT="0"
KEYWORDS="~amd64 ~arm64"
DEPEND=""
RDEPEND="${DEPEND}"
src_unpack() {
	cargo_src_unpack
}
src_compile() {
	cargo_src_compile -- -p forgum-engine
}
src_install() {
	dobin "target/release/forgum-engine" || die
}
