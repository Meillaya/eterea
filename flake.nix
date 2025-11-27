{
  description = "Eterea - Lightning-fast Twitter bookmarks manager";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        commonBuildInputs = with pkgs; [
          rustToolchain
          cargo-tauri
          pkg-config
          gcc
          sqlite
          bun
          nodejs_20
          git
        ];

        linuxBuildInputs = with pkgs; [
          webkitgtk_4_1
          gtk3
          libsoup_3
          gdk-pixbuf
          pango
          cairo
          glib
          openssl
          librsvg
          libayatana-appindicator
        ];

        darwinBuildInputs = with pkgs; [
          darwin.apple_sdk.frameworks.Security
          darwin.apple_sdk.frameworks.CoreServices
          darwin.apple_sdk.frameworks.CoreFoundation
          darwin.apple_sdk.frameworks.WebKit
        ];

        buildInputs = commonBuildInputs
          ++ pkgs.lib.optionals pkgs.stdenv.isLinux linuxBuildInputs
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin darwinBuildInputs;

      in
      {
        devShells.default = pkgs.mkShell {
          inherit buildInputs;

          shellHook = "echo Eterea dev environment ready";

          PKG_CONFIG_PATH = pkgs.lib.makeSearchPath "lib/pkgconfig" [
            pkgs.sqlite.dev
            pkgs.openssl.dev
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (
            pkgs.lib.optionals pkgs.stdenv.isLinux [
              pkgs.sqlite
              pkgs.openssl
              pkgs.webkitgtk_4_1
              pkgs.gtk3
            ]
          );

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "eterea";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = with pkgs; [ sqlite openssl ];
          meta = with pkgs.lib; {
            description = "Lightning-fast Twitter bookmarks manager";
            license = licenses.mit;
          };
        };
      }
    );
}
