{
  inputs = {
    flakelight.url = "github:nix-community/flakelight";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { flakelight
    , nixpkgs
    , rust-overlay
    , ...
    }:
    # flakelight automatically import everything in ./nixdir
    flakelight ./. {
      inputs.nixpkgs = nixpkgs;
      withOverlays = [
        rust-overlay.overlays.default
      ];

      devShell.packages = { pkgs, ... }:
        with pkgs; [
          (rust-bin.stable.latest.default.override
            {
              extensions = [ "rust-src" ];
            })
          gcc
          cmake
          glib
          pkg-config

          gtk4
          gobject-introspection
          wrapGAppsHook4
          libnotify
          dbus

          git
          unzip
          p7zip

          # adwaita-1-demo
          libadwaita.devdoc
          icon-library

          libadwaita
          gdk-pixbuf
        ];
    };
}
