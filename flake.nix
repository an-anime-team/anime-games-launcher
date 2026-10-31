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
            launcher = {
                lib,
                rustPlatform,
                gcc,
                cmake,
                glib,
                pkg-config,
                gtk4,
                gobject-introspection,
                wrapGAppsHook4,
                libnotify,
                dbus,
                makeWrapper,
                libadwaita,
                gdk-pixbuf,

                unzip,
                p7zip,

                sqliteSupport ? true,
                protobufSupport ? true,
                torrentSupport ? true,
                portalSupport ? true,
                secretsSupport ? true,
            }:
                let
                    config = lib.importTOML ./crates/anime-games-launcher/Cargo.toml;

                    apiFeatures = lib.concatLists [
                        (lib.optionals sqliteSupport [ "--features" "anime-games-launcher/sqlite-api" ])
                        (lib.optionals protobufSupport [ "--features" "anime-games-launcher/protobuf-api" ])
                        (lib.optionals torrentSupport [ "--features" "anime-games-launcher/torrent-api" ])
                        (lib.optionals portalSupport [ "--features" "anime-games-launcher/portal-api" ])
                        (lib.optionals secretsSupport [ "--features" "anime-games-launcher/secrets-api" ])
                    ];
                in rustPlatform.buildRustPackage {
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

                    meta = {
                        description = config.package.description;
                        homepage = config.package.homepage;
                        license = lib.licenses.gpl3Plus;

                        maintainers = [
                            {
                                name = "Nikita Podvirnyi";
                                email = "krypt0nn@dawn.wine";
                                github = "krypt0nn";
                                githubId = 29639507;
                            }
                        ];
                    };

                    nativeBuildInputs = [
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

                    buildInputs = [
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
                            --prefix PATH : "${lib.makeBinPath [ unzip p7zip ]}"
                        )
                    '';
                };

            anirun = {
                lib,
                rustPlatform,
                gcc,
                cmake,
                glib,
                pkg-config,
                libnotify,
                dbus,
                makeWrapper,
                wayland,
                unzip,
                p7zip,

                sqliteSupport ? true,
                protobufSupport ? true,
                torrentSupport ? true,
                portalSupport ? true,
                secretsSupport ? true,
            }:
                let
                    config = lib.importTOML ./crates/anirun/Cargo.toml;

                    apiFeatures = lib.concatLists [
                        (lib.optionals sqliteSupport [ "--features" "anirun/sqlite-api" ])
                        (lib.optionals protobufSupport [ "--features" "anirun/protobuf-api" ])
                        (lib.optionals torrentSupport [ "--features" "anirun/torrent-api" ])
                        (lib.optionals portalSupport [ "--features" "anirun/portal-api" ])
                        (lib.optionals secretsSupport [ "--features" "anirun/secrets-api" ])
                    ];
                in rustPlatform.buildRustPackage {
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

                    meta = {
                        description = config.package.description;
                        homepage = config.package.homepage;
                        license = lib.licenses.gpl3Plus;

                        maintainers = [
                            {
                                name = "Nikita Podvirnyi";
                                email = "krypt0nn@dawn.wine";
                                github = "krypt0nn";
                                githubId = 29639507;
                            }
                        ];
                    };

                    nativeBuildInputs = [
                        gcc
                        cmake
                        glib
                        pkg-config
                        libnotify
                        dbus
                        makeWrapper
                    ];

                    buildInputs = lib.optionals portalSupport [ wayland ];

                    preFixup = ''
                        wrapProgram $out/bin/anirun \
                            --prefix PATH : "${lib.makeBinPath [ unzip p7zip ]}" \
                            ${lib.optionalString portalSupport ''
                                --prefix LD_LIBRARY_PATH : "${lib.makeLibraryPath [ wayland ]}"
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
                    packages = {
                        default = self.packages.${system}.anime-games-launcher;

                        anime-games-launcher = pkgs.callPackage launcher { };
                        anirun = pkgs.callPackage anirun { };
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

                            default = self.packages.${pkgs.stdenv.hostPlatform}.anime-games-launcher.override {
                                sqliteSupport = cfg.api.sqlite;
                                protobufSupport = cfg.api.protobuf;
                                torrentSupport = cfg.api.torrent;
                                portalSupport = cfg.api.portal;
                                secretsSupport = cfg.api.secrets;
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

                                default = self.packages.${pkgs.stdenv.hostPlatform}.anirun.override {
                                    sqliteSuport = cfg.anirun.api.sqlite;
                                    protobufSuport = cfg.anirun.api.protobuf;
                                    torrentSuport = cfg.anirun.api.torrent;
                                    portalSuport = cfg.anirun.api.portal;
                                    secretsSuport = cfg.anirun.api.secrets;
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
