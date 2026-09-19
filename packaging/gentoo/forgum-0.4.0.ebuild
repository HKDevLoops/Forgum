# Copyright 2024-2026 Gentoo Authors
# Distributed under the terms of the MIT License
# Ebuild for Forgum - Modern ANSI animation mascot and shell integration engine

EAPI=8

CRATES=""

inherit cargo

DESCRIPTION="Cross-platform ANSI animation mascot and shell integration engine"
HOMEPAGE="https://github.com/HKDevLoops/Forgum"
SRC_URI="https://github.com/HKDevLoops/Forgum/archive/refs/tags/v${PV}.tar.gz -> ${P}.tar.gz"

LICENSE="MIT"
SLOT="0"
KEYWORDS="~amd64 ~arm64 ~x86"

DEPEND=""
RDEPEND="${DEPEND}"

src_unpack() {
	cargo_src_unpack
}

src_compile() {
	cargo_src_compile -- -p forgum-engine --bin forgum
}

src_install() {
	# Install primary forgum executable
	dobin "target/release/forgum" || die
	# Backward compatibility symlink
	dosym forgum /usr/bin/forgum-engine
}
