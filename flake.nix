{
    description = "Anime Games Launcher";

    inputs = {
        nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
        flake-utils.url = "github:numtide/flake-utils";

        rust-overlay = {
            url = "github:oxalica/rust-overlay";
            inputs.nixpkgs.follows = "nixpkgs";
        };
    };

    outputs = { self, nixpkgs, flake-utils, rust-overlay }:
        let
            buildLauncher = pkgs:
                let
                    config = pkgs.lib.importTOML ./crates/anime-games-launcher/Cargo.toml;
                in pkgs.rustPlatform.buildRustPackage {
                    pname = config.package.name;
                    version = config.package.version;

                    src = ./.;
                    cargoLock.lockFile = ./Cargo.lock;
                    cargoBuildFlags = [ "--package=anime-games-launcher" ];

                    doCheck = false;

                    meta = with pkgs.lib; {
                        description = config.package.description;
                        homepage = config.package.homepage;
                        license = licenses.gpl3Plus;

                        maintainers = [
                            {
                                name = "Nikita Podvirnyi";
                                email = "krypt0nn@dawn.wine";
                                matrix = "@krypt0nn:mozilla.org";
                                github = "krypt0nn";
                                githubId = 29639507;
                            }
                        ];
                    };

                    nativeBuildInputs = with pkgs; [
                        gcc
                        cmake
                        glib
                        pkg-config

                        gtk4
                        gobject-introspection
                        wrapGAppsHook4
                        libnotify
                        dbus
                        makeWrapper
                    ];

                    buildInputs = with pkgs; [
                        libadwaita
                        gdk-pixbuf
                    ];

                    postInstall = ''
                        install -Dm644 crates/anime-games-launcher/assets/anime-games-launcher.desktop \
                            $out/share/applications/anime-games-launcher.desktop

                        install -Dm644 crates/anime-games-launcher/assets/images/icon.png \
                            $out/share/icons/hicolor/scalable/apps/moe.launcher.anime-games-launcher.png
                    '';

                    preFixup = ''
                        gappsWrapperArgs+=(
                            --prefix PATH : "${pkgs.lib.makeBinPath [ pkgs.unzip pkgs.p7zip ]}"
                        )
                    '';
                };

            buildAnirun = pkgs:
                let
                    config = pkgs.lib.importTOML ./crates/anirun/Cargo.toml;
                in pkgs.rustPlatform.buildRustPackage {
                    pname = config.package.name;
                    version = config.package.version;

                    src = ./.;
                    cargoLock.lockFile = ./Cargo.lock;
                    cargoBuildFlags = [ "--package=anirun" ];

                    doCheck = false;

                    meta = with pkgs.lib; {
                        description = config.package.description;
                        homepage = config.package.homepage;
                        license = licenses.gpl3Plus;

                        maintainers = [
                            {
                                name = "Nikita Podvirnyi";
                                email = "krypt0nn@dawn.wine";
                                matrix = "@krypt0nn:mozilla.org";
                                github = "krypt0nn";
                                githubId = 29639507;
                            }
                        ];
                    };

                    nativeBuildInputs = with pkgs; [
                        gcc
                        cmake
                        glib
                        pkg-config
                        makeWrapper
                    ];

                    preFixup = ''
                        wrapProgram $out/bin/anirun \
                            --prefix PATH : "${pkgs.lib.makeBinPath [ pkgs.unzip pkgs.p7zip ]}"
                    '';
                };
        in
            (flake-utils.lib.eachDefaultSystem (system:
                let
                    pkgs = import nixpkgs {
                        inherit system;

                        overlays = [ rust-overlay.overlays.default ];
                    };
                in {
                    packages = rec {
                        default = anime-games-launcher;

                        anime-games-launcher = buildLauncher pkgs;
                        anirun = buildAnirun pkgs;
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
                            libnotify
                            dbus

                            unzip
                            p7zip

                            # adwaita-1-demo
                            libadwaita.devdoc
                            icon-library

                            python3
                        ];

                        buildInputs = with pkgs; [
                            libadwaita
                            gdk-pixbuf
                        ];
                    };
                }
            )) // {
                nixosModules.anime-games-launcher = { config, lib, pkgs, ... }: let
                    cfg = config.programs.anime-games-launcher;
                in {
                    options.programs.anime-games-launcher = {
                        enable = lib.mkEnableOption "Enable Anime Games Launcher";

                        package = lib.mkOption {
                            type = lib.types.package;
                            default = buildLauncher pkgs;
                            description = "The anime-games-launcher package to use";
                        };

                        anirun = {
                            enable = lib.mkEnableOption "Enable anime games launcher CLI tool for lua runtime evals";

                            package = lib.mkOption {
                                type = lib.types.package;
                                default = buildAnirun pkgs;
                                description = "The anirun package to use";
                            };
                        };
                    };

                    config = lib.mkMerge [
                        (lib.mkIf cfg.enable {
                            environment.systemPackages = [ cfg.package ];
                        })

                        (lib.mkIf cfg.anirun.enable {
                            environment.systemPackages = [ cfg.anirun.package ];
                        })
                    ];
                };
            };
}
