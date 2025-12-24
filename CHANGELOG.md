# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.0.0-beta3] - 24.12.2025

### Added

- Added special handling for empty game editions list
- Added runtime torrent API
- Added `sleep` runtime function

### Changed

- Improve actions pipeline graph drawing

## [2.0.0-beta2] - 22.12.2025

### Added

- Added separate read and write permissions to sandboxed filesystem paths in
  modules runtime
- Added modules allow lists. Modules runtime tries to read module's scope from
  it and falls back to default values
- Add module scope to the game package lock. This scope will be applied to all
  the modules used by the game integration (game-specific sandbox permissions)
- Added portal API
- Added logging for runtime modules loading

### Fixed

- Fixed layout of the games store details page
- Provide most of default lua functions for runtime modules
- Input resources of a package are now allowed to be read by output modules of
  this package
- Fixed panic message on application close
- Fixed game launch info hint being `nil` when unset

### Changed

- Changed logging filters for stdout and `debug.log` file
- Game integration pipeline actions now don't need to return any (boolean)
  output from `perform` functions
- Changed pipeline actions graph update rate to 0.5 seconds
- In many manifests `format` is expected instead of `version`. For now `version`
  is accepted as fallback field

## [2.0.0-beta1] - 20.12.2025

🚀 Complete rework of the app

## [2.0.0-alpha2] - 23.04.2025

### Added

- Added game description wrapping on the store page
- Added `fs.seek_rel` and `fs.truncate` methods to the v1 packages standard
- Added SQLite and Portals APIs for v1 packages standard
- Automatically resolve relative paths in v1 standard's filesystem API

### Fixed

- Fixed null values decoding from `json` and other formats in the `str.decode`
  v1 packages standard
- Fixed total download size detection in v1 standard's Downloader API

### Changed

- Updated `mlua` to version ^0.10
- Slightly optimized lua tables creation in many places
- Disabled low-level network-related logging
- Updated v1 standard's Downloader API to allow parallel download tasks
- Changed path returned by the `path.persist_dir` method in the v1 packages standard
- Updated packages loading order in the engine code
- Made game editions optional in the v1 games integrations standard
- Enum rows returned from the v1 games integrations standard are now forcely sorted
- Now v1 packages engine's `clone` function preserves metatables

### Removed

- Removed `update-unavailable` game status from the v1 games integrations standard

## [2.0.0-alpha1] - 14.04.2025

🚀 Complete rewrite, first public alpha release.

## [1.0.2] - 21.01.2024

### Changed

- Fixed German
- Replaced `v1_network_http_get` with more powerful `v1_network_fetch`

## [1.0.1] - 20.01.2024

### Added

- Added Chinese
- Added Portuguese
- Added German
- Added outdated games category
- Added virtual desktop preference
- Added xxhash support
- Added `pre_transition` optional API

### Changed

- Updated `v1_network_http_get` standard

## [1.0.0] - 13.01.2024

🚀 Initial release

<br>

[unreleased]: https://github.com/an-anime-team/anime-games-launcher/compare/v2.0.0-beta3...next
[2.0.0-beta3]: https://github.com/an-anime-team/anime-games-launcher/compare/v2.0.0-beta2...v2.0.0-beta3
[2.0.0-beta2]: https://github.com/an-anime-team/anime-games-launcher/compare/v2.0.0-beta1...v2.0.0-beta2
[2.0.0-beta1]: https://github.com/an-anime-team/anime-games-launcher/compare/v2.0.0-alpha2...v2.0.0-beta1
[2.0.0-alpha2]: https://github.com/an-anime-team/anime-games-launcher/compare/v2.0.0-alpha1...v2.0.0-alpha2
[2.0.0-alpha1]: https://github.com/an-anime-team/anime-games-launcher/compare/v1.0.2...v2.0.0-alpha1
[1.0.2]: https://github.com/an-anime-team/anime-games-launcher/compare/v1.0.1...v1.0.2
[1.0.1]: https://github.com/an-anime-team/anime-games-launcher/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/an-anime-team/anime-games-launcher/releases/tag/v1.0.0
