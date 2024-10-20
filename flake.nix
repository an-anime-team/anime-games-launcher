{
    description = "Anime Games Launcher";

    inputs = {
        nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

        rust-overlay = {
            url = "github:oxalica/rust-overlay";
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

    outputs = { self, nixpkgs, rust-overlay }:
        let
            system = "x86_64-linux";

            pkgs = import nixpkgs {
                inherit system;

                overlays = [
                    rust-overlay.overlays.default
                ];
            };

            config = pkgs.lib.importTOML ./Cargo.toml;

        in {
            packages.${system}.default = pkgs.rustPlatform.buildRustPackage {
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
                ];

                buildInputs = with pkgs; [
                    libadwaita
                    gtk4
                    gdk-pixbuf
                    gobject-introspection

                    openssl
                    luau
                ];
            };

            devShells.${system}.default = pkgs.mkShell {
                nativeBuildInputs = with pkgs; [
                    (rust-bin.stable.latest.default.override {
                        extensions = [ "rust-src" ];
                    })

                    gcc
                    cmake
                    pkg-config

                    git
                    unzip
                    p7zip
                    libwebp

                    # adwaita-1-demo
                    libadwaita.devdoc
                ];

                buildInputs = with pkgs; [
                    libadwaita
                    gtk4
                    gdk-pixbuf
                    gobject-introspection

                    openssl
                    luau
                ];

                # CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = "${pkgs.llvmPackages.clangUseLLVM}/bin/clang";
                # CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER = "${pkgs.llvmPackages.clangUseLLVM}/bin/clang";

                # CARGO_ENCODED_RUSTFLAGS = "-Clink-arg=--ld-path=${pkgs.mold}/bin/mold";
            };
        };
}
