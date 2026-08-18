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
            buildLauncher = {
                pkgs,
                api ? {
                    sqlite = true;
                    protobuf = true;
                    torrent = true;
                    portal = true;
                    secrets = true;
                }
            }:
                let
                    config = pkgs.lib.importTOML ./crates/anime-games-launcher/Cargo.toml;

                    apiFeatures = pkgs.lib.concatLists [
                        (pkgs.lib.optionals api.sqlite [ "--features" "anime-games-launcher/sqlite-api" ])
                        (pkgs.lib.optionals api.protobuf [ "--features" "anime-games-launcher/protobuf-api" ])
                        (pkgs.lib.optionals api.torrent [ "--features" "anime-games-launcher/torrent-api" ])
                        (pkgs.lib.optionals api.portal [ "--features" "anime-games-launcher/portal-api" ])
                        (pkgs.lib.optionals api.secrets [ "--features" "anime-games-launcher/secrets-api" ])
                    ];
                in pkgs.rustPlatform.buildRustPackage {
                    pname = config.package.name;
                    version = config.package.version;

                    src = ./.;
                    cargoLock.lockFile = ./Cargo.lock;

                    cargoBuildFlags = [
                        "--package=anime-games-launcher"
                        "--no-default-features"
                        "--features" "anime-games-launcher/mimalloc"
                    ] ++ apiFeatures;

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

            buildAnirun = {
                pkgs,
                api ? {
                    sqlite = true;
                    protobuf = true;
                    torrent = true;
                    portal = true;
                    secrets = true;
                }
            }:
                let
                    config = pkgs.lib.importTOML ./crates/anirun/Cargo.toml;

                    apiFeatures = pkgs.lib.concatLists [
                        (pkgs.lib.optionals api.sqlite [ "--features" "anirun/sqlite-api" ])
                        (pkgs.lib.optionals api.protobuf [ "--features" "anirun/protobuf-api" ])
                        (pkgs.lib.optionals api.torrent [ "--features" "anirun/torrent-api" ])
                        (pkgs.lib.optionals api.portal [ "--features" "anirun/portal-api" ])
                        (pkgs.lib.optionals api.secrets [ "--features" "anirun/secrets-api" ])
                    ];
                in pkgs.rustPlatform.buildRustPackage {
                    pname = config.package.name;
                    version = config.package.version;

                    src = ./.;
                    cargoLock.lockFile = ./Cargo.lock;

                    cargoBuildFlags = [
                        "--package=anirun"
                        "--no-default-features"
                        "--features" "anirun/mimalloc"
                    ] ++ apiFeatures;

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
                        libnotify
                        dbus
                        makeWrapper
                    ];

                    buildInputs = pkgs.lib.optionals api.portal [ pkgs.wayland ];

                    preFixup = ''
                        wrapProgram $out/bin/anirun \
                            --prefix PATH : "${pkgs.lib.makeBinPath [ pkgs.unzip pkgs.p7zip ]}" \
                            ${pkgs.lib.optionalString api.portal ''
                                --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath [ pkgs.wayland ]}"
                            ''}
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
                        anirun = buildAnirun { inherit pkgs; };
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

                            default = buildLauncher {
                                inherit pkgs;

                                api = {
                                    sqlite = cfg.api.sqlite;
                                    protobuf = cfg.api.protobuf;
                                    torrent = cfg.api.torrent;
                                    portal = cfg.api.portal;
                                    secrets = cfg.api.secrets;
                                };
                            };

                            description = "The anime-games-launcher package to use";
                        };

                        api = {
                            sqlite = lib.mkOption {
                                type = lib.types.bool;
                                default = true;
                                description = "Build anime-games-launcher with the sqlite API";
                            };

                            protobuf = lib.mkOption {
                                type = lib.types.bool;
                                default = true;
                                description = "Build anime-games-launcher with the protobuf API";
                            };

                            torrent = lib.mkOption {
                                type = lib.types.bool;
                                default = true;
                                description = "Build anime-games-launcher with the torrent API";
                            };

                            portal = lib.mkOption {
                                type = lib.types.bool;
                                default = true;
                                description = "Build anime-games-launcher with the portal API";
                            };

                            secrets = lib.mkOption {
                                type = lib.types.bool;
                                default = true;
                                description = "Build anime-games-launcher with the secrets API";
                            };
                        };

                        anirun = {
                            enable = lib.mkEnableOption "Enable anime games launcher CLI tool for lua runtime evals";

                            package = lib.mkOption {
                                type = lib.types.package;

                                default = buildAnirun {
                                    inherit pkgs;

                                    api = {
                                        sqlite = cfg.anirun.api.sqlite;
                                        protobuf = cfg.anirun.api.protobuf;
                                        torrent = cfg.anirun.api.torrent;
                                        portal = cfg.anirun.api.portal;
                                        secrets = cfg.anirun.api.secrets;
                                    };
                                };

                                description = "The anirun package to use";
                            };

                            api = {
                                sqlite = lib.mkOption {
                                    type = lib.types.bool;
                                    default = true;
                                    description = "Build anirun with the sqlite API";
                                };

                                protobuf = lib.mkOption {
                                    type = lib.types.bool;
                                    default = true;
                                    description = "Build anirun with the protobuf API";
                                };

                                torrent = lib.mkOption {
                                    type = lib.types.bool;
                                    default = true;
                                    description = "Build anirun with the torrent API";
                                };

                                portal = lib.mkOption {
                                    type = lib.types.bool;
                                    default = true;
                                    description = "Build anirun with the portal API";
                                };

                                secrets = lib.mkOption {
                                    type = lib.types.bool;
                                    default = true;
                                    description = "Build anirun with the secrets API";
                                };
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
