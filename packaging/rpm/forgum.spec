Name:           forgum
Version:        0.0.1
Release:        0.1.alpha1%{?dist}
Summary:        Cross-platform cowsay+fortune+lolcat

License:        MIT
URL:            https://github.com/HKDevLoops/Forgum
Source0:        %{name}-%{version}.tar.gz

BuildArch:      x86_64

%description
Cross-platform cowsay+fortune+lolcat with a Rust ANSI animation engine.

%install
mkdir -p %{buildroot}%{_bindir}
install -m 755 forgum-engine %{buildroot}%{_bindir}/forgum-engine
ln -sf forgum-engine %{buildroot}%{_bindir}/forgum

%files
%{_bindir}/forgum-engine
%{_bindir}/forgum

%changelog
* Sat Sep 05 2026 HKDevLoops <hkdevloops@example.com> - 0.5.0-1
- Responsive tables, procedural scenery, universal options discovery, and dual binary support
* Mon Jun 30 2026 HKDEVS <hkdevs@example.com> - 0.4.0-1
- Initial RPM package release
