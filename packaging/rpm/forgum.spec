Name:           forgum
Version:        0.4.0
Release:        1%{?dist}
Summary:        Cross-platform cowsay+fortune+lolcat

License:        MIT
URL:            https://github.com/harish2222/Forgum
Source0:        %{name}-%{version}.tar.gz

BuildArch:      x86_64

%description
Cross-platform cowsay+fortune+lolcat with a Rust ANSI animation engine.

%install
mkdir -p %{buildroot}%{_bindir}
install -m 755 forgum-engine %{buildroot}%{_bindir}/forgum-engine

%files
%{_bindir}/forgum-engine

%changelog
* Mon Jun 30 2026 HKDEVS <hkdevs@example.com> - 0.4.0-1
- Initial RPM package release
