{
  description = "QMK HID host";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs =
    { flake-utils, nixpkgs, ... }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
        };
        rustfmt = pkgs.rustfmt.override { asNightly = true; };
        
        isDarwin = pkgs.stdenv.isDarwin;
        isLinux  = pkgs.stdenv.isLinux;

        baseDeps = with pkgs; [
          cmake
          rustc
          pkg-config
        ];

        linuxDeps = with pkgs; [
          systemd       # libudev
          pulseaudio
          dbus
          xorg.libX11
        ];

        darwinDeps = with pkgs; [ ];
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = baseDeps ++ (if isLinux then linuxDeps else darwinDeps);

          shellHook = ''
            if test -f ".env"; then
              source .env
            fi
            echo "🦀 Dev shell for qmk-hid-host on ${system}"
            echo "→ Using ${if isLinux then "Linux" else "macOS"} dependencies"
          '';
        };
      }
    );
}
