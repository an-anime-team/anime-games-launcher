{
    description = "Anime Games Launcher";

    inputs.nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

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

    outputs = { self, nixpkgs }:
        let
            system = "x86_64-linux";

            pkgs = import nixpkgs { inherit system; };

            config = pkgs.lib.importTOML ./Cargo.toml;

        in {
            packages.${system}.default = pkgs.rustPlatform.buildRustPackage {
                pname = config.name;
                version = config.version;

                src = ./.;

                cargoLock = {
                    lockFile = ./Cargo.lock;
                };
            };

            devShells.${system}.default = pkgs.mkShell {
                nativeBuildInputs = with pkgs; [
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
