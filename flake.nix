{
    description = "Anime Games Launcher";

    inputs = {
        nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
        flake-utils.url = "github:numtide/flake-utils";

        rust-overlay = {
            url = "github:oxalica/rust-overlay";
            inputs.nixpkgs.follows = "nixpkgs";
        };

        nixos-bundlers = {
            url = "github:NixOS/bundlers";
            inputs.nixpkgs.follows = "nixpkgs";
        };
    };

    nixConfig = {
        extra-substituters = [
            "https://cache.nixos.org"
            "https://nix-community.cachix.org"
            "https://krypt0nn.cachix.org"
        ];

        extra-trusted-public-keys = [
            "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY="
            "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
            "krypt0nn.cachix.org-1:ciP8xHjGQDDEjSW1LL9PO/fn8JRzm8zb57eUcFAblR8="
        ];
    };

    outputs = { self, nixpkgs, flake-utils, rust-overlay, nixos-bundlers }:
        flake-utils.lib.eachDefaultSystem (system:
            let
                pkgs = import nixpkgs {
                    inherit system;

                    overlays = [ rust-overlay.overlays.default ];
                };

                config = pkgs.lib.importTOML ./Cargo.toml;

            in {
                packages.default = pkgs.rustPlatform.buildRustPackage {
                    pname = config.package.name;
                    version = config.package.version;

                    src = ./.;
                    cargoLock.lockFile = ./Cargo.lock;

                    doCheck = false;

                    meta = with pkgs.lib; {
                        description = config.package.description;
                        homepage = config.package.homepage;
                        license = licenses.gpl3Only;

                        maintainers = [
                            {
                                name = "Nikita Podvirnyi";
                                email = "krypt0nn@vk.com";
                                matrix = "@krypt0nn:mozilla.org";
                                github = "krypt0nn";
                                githubId = 29639507;
                            }
                        ];
                    };

                    nativeBuildInputs = with pkgs; [
                        rust-bin.stable.latest.minimal

                        gcc
                        cmake
                        glib
                        pkg-config

                        gtk4
                        gobject-introspection
                        wrapGAppsHook4
                    ];

                    buildInputs = with pkgs; [
                        libadwaita
                        gdk-pixbuf
                    ];
                };

                bundlers = with nixos-bundlers.bundlers.${system}; {
                    deb = toDEB;
                    rpm = toRPM;
                    arx = toArx;
                };

                devShells.default = pkgs.mkShell {
                    nativeBuildInputs = with pkgs; [
                        (rust-bin.stable.latest.default.override {
                            extensions = [ "rust-src" ];
                        })

                        gcc
                        cmake
                        glib
                        pkg-config

                        gtk4
                        gobject-introspection
                        wrapGAppsHook4

                        git
                        unzip
                        p7zip
                        libwebp

                        # adwaita-1-demo
                        libadwaita.devdoc
                        icon-library
                    ];

                    buildInputs = with pkgs; [
                        libadwaita
                        gdk-pixbuf
                    ];
                };
            });
}
