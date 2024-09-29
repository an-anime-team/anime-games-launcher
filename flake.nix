{
    description = "Anime Games Launcher";

    inputs.nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

    outputs = { self, nixpkgs }:
        let
            system = "x86_64-linux";

            # pkgs = (import nixpkgs { inherit system; }).pkgsStatic;
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
                ];

                buildInputs = with pkgs; [
                    gtk4
                    gdk-pixbuf
                    gobject-introspection

                    libadwaita
                    openssl
                    luau
                ];

                # CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = "${pkgs.llvmPackages.clangUseLLVM}/bin/clang";
                # CARGO_ENCODED_RUSTFLAGS = "-Clink-arg=-fuse-ld=${pkgs.mold}/bin/mold";
            };
        };
}
