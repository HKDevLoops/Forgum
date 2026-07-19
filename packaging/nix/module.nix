# forbum NixOS/home-manager module (best-effort, untested in CI)
#
# Nix CI in this repo runs on Linux only, so this module is provided as a
# convenience and has not been exercised in automated tests. It exposes a
# `programs.forgum` option that, when enabled, appends the generated shell
# init hook to the interactive rc files for bash, zsh and fish.
#
# Note: it assumes the engine binary provides an `init <shell>` subcommand
# that prints shell init code (see the shell-integration work in the repo).
{ config, lib, pkgs, ... }:

let
  cfg = config.programs.forgum;
in
{
  options.programs.forgum = {
    enable = lib.mkEnableOption "forgum shell hooks";

    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.forgum or (import ./package.nix pkgs);
      defaultText = "pkgs.forgum";
    };
  };

  config = lib.mkIf cfg.enable {
    # Append the generated hook to interactive shell rc files.
    programs.bash.interactiveShellInit = lib.optionalString cfg.enable "${cfg.package}/bin/forgum-engine init bash";
    programs.zsh.interactiveShellInit = lib.optionalString cfg.enable "${cfg.package}/bin/forgum-engine init zsh";
    programs.fish.interactiveShellInit = lib.optionalString cfg.enable "${cfg.package}/bin/forgum-engine init fish";
  };
}
