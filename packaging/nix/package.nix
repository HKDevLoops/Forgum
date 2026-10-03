# Call with: nix-build -E 'with import <nixpkgs> {}; import ./packaging/nix/package.nix {}'

pkgs: pkgs.rustPlatform.buildRustPackage {
  pname = "forgum";
  version = "0.0.3-beta";

  src = ../..;

  cargoLock.lockFile = ../../Cargo.lock;

  # Build the forgum binary
  cargoBuildFlags = [ "-p" "forgum-engine" "--bin" "forgum" ];
  cargoTestFlags = [ "-p" "forgum-engine" "--bin" "forgum" ];

  nativeBuildInputs = [ pkgs.pkg-config ];
  buildInputs = [ ];

  meta = {
    description = "Forgum - cross-platform ANSI animation mascot and shell integration engine";
    homepage = "https://github.com/HKDevLoops/Forgum";
    license = pkgs.lib.licenses.mit;
    mainProgram = "forgum";
    maintainers = [ ];
  };
}
