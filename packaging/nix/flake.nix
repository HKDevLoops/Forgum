{
  description = "forgum - cross-platform cowsay+fortune+lolcat";

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
        };
      in
      {
        packages.default = forgum;

        # Optional NixOS/home-manager module enabling the shell hook
        # for bash/zsh/fish. Import with:
        #   nixosConfigurations.host = nixosSystem {
        #     modules = [ self.nixosModules.forgum ./configuration.nix ];
        #   };
        nixosModules.forgum = import ./module.nix;
      });
}
