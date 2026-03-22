{
  config,
  lib,
  pkgs,
  ...
}: let
  inherit
    (lib)
    mkIf
    mkEnableOption
    mkPackageOption
    mkOption
    types
    optionals
    ;
  cfg = config.programs.agl;
  tomlFormat = pkgs.formats.toml {};
in {
  options.programs.agl = {
    enable = mkEnableOption "anime games launcher";
    package = mkPackageOption pkgs "anime-games-launcher" {};
    anirun.enable = mkEnableOption "anirun";
    anirun.package = mkPackageOption pkgs "anirun" {};

    settings = mkOption {
      type = types.submodule {
        options = {
          general = {
            language = mkOption {
              type = types.str;
              default = "system";
              description = "Language of the launcher. If unset (`system`) - the system one is used.";
            };
            network = {
              timeout = mkOption {
                type = types.int;
                default = 5000;
                description = "Timeout for HTTP requests in milliseconds.";
              };
              proxy = {
                url = mkOption {
                  type = types.str;
                  default = "system";
                  description = "Proxy URL. If unset (`system`) - environment variable proxy is used.";
                };
                mode = mkOption {
                  type = types.enum ["http" "https" "all" "system"];
                  default = "system";
                  description = "Proxy mode.";
                };
              };
            };
          };

          cache = {
            images.duration = mkOption {
              type = types.int;
              default = 28800;
              description = "Duration of the images cache in seconds.";
            };
            game_registries.duration = mkOption {
              type = types.int;
              default = 57600;
              description = "Duration of the game registries cache in seconds.";
            };
            game_manifests.duration = mkOption {
              type = types.int;
              default = 86400;
              description = "Duration of the game manifests cache in seconds.";
            };
            game_packages.duration = mkOption {
              type = types.int;
              default = 28800;
              description = "Duration of the game packages cache in seconds.";
            };
            packages_allow_lists.duration = mkOption {
              type = types.int;
              default = 28800;
              description = "Duration of the runtime packages allow lists cache in seconds.";
            };
          };

          packages = {
            allow_lists = mkOption {
              type = types.listOf types.str;
              default = [
                "https://raw.githubusercontent.com/an-anime-team/game-integrations/refs/heads/master/packages/allow_list.json"
              ];
              description = "URLs to the modules allow lists files.";
            };
            resources.path = mkOption {
              type = types.str;
              default = "${config.home.homeDirectory}/.local/share/anime-games-launcher/packages/resources";
              description = "Path to the folder where package resources should be stored.";
            };
            modules.path = mkOption {
              type = types.str;
              default = "${config.home.homeDirectory}/.local/share/anime-games-launcher/packages/modules";
              description = "Path to the folder where modules-specific files should be stored.";
            };
            persistent.path = mkOption {
              type = types.str;
              default = "${config.home.homeDirectory}/.local/share/anime-games-launcher/packages/persistent";
              description = "Path to the folder where persistent packages files should be stored.";
            };
            temporary.path = mkOption {
              type = types.str;
              default = "${config.home.homeDirectory}/.local/share/anime-games-launcher/packages/temporary";
              description = "Path to the folder where temporary packages files should be stored.";
            };
          };

          runtime = {
            memory_limit = mkOption {
              type = types.int;
              default = 1073741824;
              description = "Maximal amount of memory in bytes allowed to be consumed by packages runtime.";
            };
            torrent = {
              enable = mkOption {
                type = types.bool;
                default = false;
                description = "Enable torrent API support.";
              };
              enable_dht = mkOption {
                type = types.bool;
                default = true;
                description = "Enable background DHT node.";
              };
              enable_upnp = mkOption {
                type = types.bool;
                default = false;
                description = "Open BitTorrent protocol port using UPnP.";
              };
              trackers = mkOption {
                type = types.listOf types.str;
                default = [];
                description = "List of torrent trackers used by the torrent API.";
              };
              blocklist_url = mkOption {
                type = types.str;
                default = "https://raw.githubusercontent.com/Naunter/BT_BlockLists/master/bt_blocklists.gz";
                description = "URL to the torrent peers blocklist.";
              };
            };
          };

          games = {
            registries = mkOption {
              type = types.listOf types.str;
              default = [
                "https://raw.githubusercontent.com/an-anime-team/game-integrations/refs/heads/master/games/registry.json"
              ];
              description = "URLs of the game registry files.";
            };
            path = mkOption {
              type = types.str;
              default = "${config.home.homeDirectory}/.local/share/anime-games-launcher/games";
              description = "Path to the folder where game locks are stored.";
            };
          };
        };
      };
      default = {};
      description = "Configuration for anime-games-launcher.";
    };
  };

  config = mkIf cfg.enable {
    home.packages = [cfg.package] ++ optionals cfg.anirun.enable [cfg.anirun.package];

    xdg.configFile."anime-games-launcher/config.toml" = mkIf (cfg.settings != {}) {
      source = tomlFormat.generate "anime-games-launcher-config" cfg.settings;
    };
  };
}
