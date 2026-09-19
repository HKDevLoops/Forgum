# Forgum NixOS/home-manager module
#
# Exposes a `programs.forgum` option that, when enabled, provides the `forgum`
# package and automatically appends the generated shell init hook to interactive
# rc files across Bash, Zsh, Fish, and Nushell.
{ config, lib, pkgs, ... }:

let
  cfg = config.programs.forgum;
in
{
  options.programs.forgum = {
    enable = lib.mkEnableOption "Forgum ANSI animation mascot and shell integration";

    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.forgum or (import ./package.nix pkgs);
      defaultText = "pkgs.forgum";
      description = "The Forgum package to install.";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    # Append the generated hook to interactive shell rc files
    programs.bash.interactiveShellInit = lib.mkAfter ''
      eval "$(${cfg.package}/bin/forgum init bash)"
    '';

    programs.zsh.interactiveShellInit = lib.mkAfter ''
      eval "$(${cfg.package}/bin/forgum init zsh)"
    '';

    programs.fish.interactiveShellInit = lib.mkAfter ''
      ${cfg.package}/bin/forgum init fish | source
    '';

    programs.nushell.extraConfig = lib.mkAfter ''
      # Forgum Nushell prompt hook
      use ${cfg.package}/bin/forgum
    '';
  };
}
