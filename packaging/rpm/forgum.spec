Name:           forgum
Version:        0.4.0
Release:        1%{?dist}
Summary:        Cross-platform ANSI animation mascot and shell integration engine

License:        MIT
URL:            https://github.com/HKDevLoops/Forgum
Source0:        %{name}-%{version}.tar.gz

BuildArch:      x86_64

%description
Forgum is a modern terminal mascot and animation engine rendering cowsay, fortune,
and lolcat with physical ANSI effects, procedural biomes, and split-shell integration.

%install
mkdir -p %{buildroot}%{_bindir}
install -m 755 forgum %{buildroot}%{_bindir}/forgum
ln -sf forgum %{buildroot}%{_bindir}/forgum-engine

%files
%{_bindir}/forgum
%{_bindir}/forgum-engine

%changelog
* Sat Sep 19 2026 HKDevLoops <hkdevloops@example.com> - 0.4.0-1
- Release 0.4.0: Unified keyword mandate, sandbox container support, nature biomes, and split shell integration
* Sat Sep 05 2026 HKDevLoops <hkdevloops@example.com> - 0.3.0-1
- Responsive tables, procedural scenery, universal options discovery, and dual binary support
* Mon Jun 30 2026 HKDEVS <hkdevs@example.com> - 0.1.0-1
- Initial RPM package release
