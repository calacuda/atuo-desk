{
  description = "A Nix-flake-based Rust development environment for PenTestDB";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-23.05";
  };

  outputs = { self, nixpkgs, ... }:
    let
      # system should match the system you are running on
      system = "x86_64-linux";
      rust_overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
      pkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };
      rustVersion = "latest";
      #rustVersion = "1.62.0";
      rust = pkgs.rust-bin.stable.${rustVersion}.default.override {
        extensions = [
          "rust-src" # for rust-analyzer
        ];
      };
    in
    {
      devShells."${system}".default =
        let
          pkgs = import nixpkgs {
            inherit system;
            # overlays = [
            #   (self: super: rec { })
            #   # cargo
            #   # rustup
            #   # gdb
            #   # openssl
            #   # pkg-config
            # ];
          };
        in
        pkgs.mkShell {
          # create an environment with nodejs-18_x, pnpm, and yarn
          packages = with pkgs; [
            gdb
            openssl
            pkg-config
            # cargo
            # rustc
            rust-analyzer
            rustfmt
            clippy
            rusty-man
            udev
            xorg.xorgproto
            xorg.libX11
            xorg.libXt
            xorg.libXft
            xorg.libXext
            xorg.libSM
            xorg.libICE
            xorg.libXi
            xorg.libXtst
            iw
            iwd
            llvmPackages_rocm.clang
            wpa_supplicant
            dbus
          ];
          shellHook = ''
            IN_NIX_DEV="yes"
          '';
        };
    };
}
