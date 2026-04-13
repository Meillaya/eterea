{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  name = "eterea-dev";

  buildInputs = with pkgs; [
    # Rust toolchain
    rustc
    cargo
    rustfmt
    clippy

    # C compiler and linker (required for native deps)
    gcc
    pkg-config

    # SQLite development files
    sqlite

    # Dioxus desktop dependencies (Linux/GTK/WebKit)
    webkitgtk_4_1
    gtk3
    libsoup_3
    gdk-pixbuf
    pango
    cairo
    glib
    openssl
    librsvg

    # Additional tools
    git
  ];

  shellHook = ''
    echo "Eterea development environment"
    echo ""
    echo "Commands:"
    echo "  cargo build --workspace       # Build the desktop MVP + shared services"
    echo "  cargo run -p eterea-dioxus    # Run the Dioxus desktop app"
    echo "  cargo test --workspace        # Run regression and migration guardrail tests"
    echo ""
  '';

  # Environment variables for pkg-config
  PKG_CONFIG_PATH = "${pkgs.sqlite.dev}/lib/pkgconfig:${pkgs.openssl.dev}/lib/pkgconfig";

  # Ensure linker can find libraries
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
    pkgs.sqlite
    pkgs.openssl
    pkgs.webkitgtk_4_1
    pkgs.gtk3
  ];
}
