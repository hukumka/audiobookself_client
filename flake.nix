{
  description = "Pathfinder 2e SpellCard generator";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
      };

  outputs = {
    self, nixpkgs, flake-utils, rust-overlay
  }: flake-utils.lib.eachDefaultSystem (system: 
    let
      rustVersion = "1.74.1";
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
      # build time dependencies

      rustToolchain = 
          (pkgs.rust-bin.stable.${rustVersion}.default.override { extensions = [ "rust-src" "rust-analyzer"]; });
    in with pkgs; {
      devShells.default = mkShell {
        buildInputs = [
          rustToolchain
          pkgs.pkg-config
        ];
      };
      
    });
}
