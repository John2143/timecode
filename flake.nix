{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    utils.url = "github:numtide/flake-utils";
  };

  outputs =
    { self
    , nixpkgs
    , utils
    , rust-overlay
    , naersk
    , ...
    }:
    utils.lib.eachDefaultSystem (system:
    let
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs { inherit system overlays; };
      naersk-lib = pkgs.callPackage naersk { };
      rust = pkgs.rust-bin.stable.latest.default.override {
        # https://github.com/oxalica/rust-overlay
        extensions = [ "llvm-tools-preview" ];
      };
      nativeBuildInputs = with pkgs; [
        rust
        pkg-config
      ];
      buildInputs = with pkgs; [
        maturin
        python3
        bacon
        cargo
        cargo-edit
        rustup
        rustc
        rustfmt
        pre-commit
        # Broken, install via. cargo for now
        # cargo-llvm-cov
        sqlx-cli
        rustPackages.clippy
      ] ++ lib.optionals pkgs.stdenv.isDarwin [
        # Additional darwin specific inputs can be set here
        pkgs.libiconv
        pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
      ];
      packages = with pkgs; [
        rust-analyzer
        # cargo-llvm-cov # Disabled due to https://github.com/NixOS/nixpkgs/issues/351574
        # Needs manual install currently, should be possible with rust-overlay
      ];
    in
    {
      defaultPackage = naersk-lib.buildPackage ./.;
      devShell = with pkgs; mkShell {
        inherit nativeBuildInputs buildInputs packages;
        RUST_SRC_PATH = rustPlatform.rustLibSrc;
      };
    }
    );
}
