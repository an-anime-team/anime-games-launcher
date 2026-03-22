{ pkgs
, inputs
,
}:
let
  craneLib = inputs.crane.mkLib pkgs;
  toml = craneLib.crateNameFromCargoToml { cargoToml = ../../crates/anirun/Cargo.toml; };
  src = ../../.;

  deps = {
    buildInputs = with pkgs; [
      libadwaita
      gdk-pixbuf
    ];
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
    ];
  };

  cargoArtifacts = craneLib.buildDepsOnly ({
    inherit src;
    inherit (toml) pname version;
  }
  // deps);

  anirun = craneLib.buildPackage ({
    inherit cargoArtifacts src;
    inherit (toml) pname version;
    cargoExtraArgs = "-p anirun";
    strictDeps = true;
    doCheck = false;
  }
  // deps);
in
anirun
