# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[unreleased]: https://github.com/an-anime-team/anime-games-launcher/compare/2.0.0-alpha1...next
[2.0.0-alpha1]: https://github.com/an-anime-team/anime-games-launcher/compare/1.0.2...2.0.0-alpha1
[1.0.2]: https://github.com/an-anime-team/anime-games-launcher/compare/1.0.1...1.0.2
[1.0.1]: https://github.com/an-anime-team/anime-games-launcher/compare/1.0.0...1.0.1
[1.0.0]: https://github.com/an-anime-team/anime-games-launcher/releases/tag/1.0.0
