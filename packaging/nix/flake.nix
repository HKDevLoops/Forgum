{
  description = "Forgum - cross-platform ANSI animation mascot and shell integration engine";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        forgum = pkgs.rustPlatform.buildRustPackage {
          pname = "forgum";
          version = "0.4.0";

          src = ../..;

          cargoLock.lockFile = ../../Cargo.lock;

          # Build only the primary forgum binary
          cargoBuildFlags = [ "-p" "forgum-engine" "--bin" "forgum" ];
          cargoTestFlags = [ "-p" "forgum-engine" "--bin" "forgum" ];

          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ ];

          postInstall = ''
            ln -sf forgum $out/bin/forgum-engine
          '';

          meta = {
            description = "Forgum - cross-platform ANSI animation mascot and shell integration engine";
            homepage = "https://github.com/HKDevLoops/Forgum";
            license = pkgs.lib.licenses.mit;
            mainProgram = "forgum";
            maintainers = [ ];
          };
        };
      in
      {
        packages.default = forgum;
        packages.forgum = forgum;

        apps.default = {
          type = "app";
          program = "${forgum}/bin/forgum";
        };

        apps.forgum = {
          type = "app";
          program = "${forgum}/bin/forgum";
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ forgum ];
          packages = with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
            bash
            zsh
            fish
            nushell
            tmux
          ];
        };

        # NixOS / home-manager module
        nixosModules.forgum = import ./module.nix;
        nixosModules.default = import ./module.nix;
      });
}
