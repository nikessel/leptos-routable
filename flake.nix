{
  description = "Minimal Leptos development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/23e89b7da85c3640bbc2173fe04f4bd114342367";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
        };

        # Minimal Rust toolchain with just what we need
        rustToolchain = with fenix.packages.${system}; combine [
          latest.cargo
          latest.rustc
          latest.rust-std
          latest.rust-src
          targets.wasm32-unknown-unknown.latest.rust-std
        ];

        # Frontend tools needed for Leptos development
        frontendTools = with pkgs; [
          trunk
          sassc  # Native Sass compiler
          wasm-bindgen-cli
          binaryen # For wasm-opt
          nodePackages.tailwindcss
        ];

      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [
            rustToolchain
          ] ++ frontendTools;

          shellHook = ''
            # For Trunk to find sassc
            export PATH="${pkgs.lib.makeBinPath frontendTools}:$PATH"
          '';
        };
      }
    );
}