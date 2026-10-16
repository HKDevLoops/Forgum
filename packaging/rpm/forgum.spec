Name:           forgum
Version:        0.0.8
Release:        1%{?dist}
Summary:        Cross-platform ANSI animation mascot and shell integration engine

License:        MIT
URL:            https://github.com/HKDevLoops/Forgum
Source0:        %{name}-%{version}.tar.gz

%description
Forgum is a modern terminal mascot and animation engine rendering cowsay, fortune,
and lolcat with physical ANSI effects, procedural biomes, and split-shell integration.

%install
mkdir -p %{buildroot}%{_bindir}
install -m 755 forgum %{buildroot}%{_bindir}/forgum

%files
%{_bindir}/forgum

%changelog
* Tue Oct 13 2026 harish2222 <harish2222@users.noreply.github.com> - 0.0.8-1
- Release 0.0.8 beta: Procedural tree species diversity, seasonal phenology, celestial arcs, and multi-threaded compute
* Mon Oct 12 2026 harish2222 <harish2222@users.noreply.github.com> - 0.0.7-1
- Release 0.0.7 beta: Interactive update method selector, zero-alloc render damage pipeline, security overhaul, and packaging parity
* Mon Oct 12 2026 harish2222 <harish2222@users.noreply.github.com> - 0.0.6-1
- Release 0.0.6 beta: Certified Pantone Nature palettes, mascot art detailing, and 120/144/240Hz fluid motion engine
* Mon Oct 12 2026 harish2222 <harish2222@users.noreply.github.com> - 0.0.5-1
- Release 0.0.5 beta: Universal Unicode engine, NPU compute priority, and dynamic kinematics offload
* Sun Oct 04 2026 harish2222 <harish2222@users.noreply.github.com> - 0.0.4-1
- Release 0.0.4 beta: Moonwalk reverse locomotion, universal aspect ratios, and full CLI audit
* Fri Oct 02 2026 harish2222 <harish2222@users.noreply.github.com> - 0.0.3-1
- Release 0.0.3 beta: Biological coat polymorphism, 5 shell attach modes, responsive TUI
* Sun Sep 28 2026 harish2222 <harish2222@users.noreply.github.com> - 0.0.2-1
- Release 0.0.2 beta: Unified keyword mandate, sandbox container support, nature biomes, and split shell integration
* Sat Sep 05 2026 harish2222 <harish2222@users.noreply.github.com> - 0.3.0-1
- Responsive tables, procedural scenery, universal options discovery, and dual binary support
* Mon Jun 30 2026 harish2222 <harish2222@users.noreply.github.com> - 0.1.0-1
- Initial RPM package release
