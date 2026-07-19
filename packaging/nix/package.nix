# Call with: nix-build -E 'with import <nixpkgs> {}; import ./packaging/nix/package.nix {}'

pkgs: pkgs.rustPlatform.buildRustPackage {
  pname = "forgum";
  version = "0.4.0";

  src = ../..;

  cargoLock.lockFile = ../../Cargo.lock;

  # Build only the engine binary; the workspace has multiple crates
  # (some with platform-specific deps), so scope build/test to the engine.
  cargoBuildFlags = [ "-p" "forgum-engine" ];
  cargoTestFlags = [ "-p" "forgum-engine" ];

  nativeBuildInputs = [ pkgs.pkg-config ];
  buildInputs = [ ];

  meta = {
    description = "forgum - cross-platform cowsay+fortune+lolcat";
    homepage = "https://github.com/HKDevLoops/Forgum";
    license = pkgs.lib.licenses.mit;
    maintainers = [ ];
  };
}
