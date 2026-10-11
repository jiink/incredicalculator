{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    { self, nixpkgs, rust-overlay, ... }@inputs:
    let
      system = "x86_64-linux";
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };

      rustToolchain = pkgs.rust-bin.stable.latest.default.override {
        extensions = [ "rust-src" "rustfmt" "clippy" ];
        targets = [ "wasm32-unknown-unknown" ];
      };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs = [
          rustToolchain
          pkgs.wasm-pack
          pkgs.openssl
          pkgs.pkg-config
          pkgs.cargo-deny
          pkgs.cargo-edit
          pkgs.cargo-watch
          pkgs.rust-analyzer
          pkgs.cmake
          pkgs.alsa-lib
          pkgs.llvmPackages.libclang
          pkgs.xorg.libX11
          pkgs.xorg.xorgproto
          pkgs.mesa
          pkgs.xorg.libxcb
          pkgs.xorg.libXrandr
          pkgs.xorg.libXinerama
          pkgs.xorg.libXcursor
          pkgs.xorg.libXi
          pkgs.libGL
          pkgs.clang
        ];

        env = {
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (
            with pkgs;
            [
              alsa-lib
              libGL
              libxkbcommon
              xorg.libX11
              xorg.libXau
              xorg.libXcursor
              xorg.libXext
              xorg.libXfixes
              xorg.libXi
              xorg.libXinerama
              xorg.libXrandr
              xorg.libXrender
              xorg.libXtst
              xorg.libxcb
            ]
          );
        };
      };
    };
}