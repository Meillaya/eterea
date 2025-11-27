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

    # Node/Bun for frontend
    bun
    nodejs_20

    # Tauri dependencies (Linux/GTK)
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
    echo "  cargo build -p eterea-core    # Build Rust backend"
    echo "  cd src/frontend && bun install # Install frontend deps"
    echo "  cargo tauri dev               # Run full app"
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
