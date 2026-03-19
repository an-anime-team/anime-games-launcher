# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.0.0-rc1] - 19.03.2026

### Added

- Added `anirun` binary to test luau runtime packages and modules
- Added BSON format support in `string.encode`, `string.decode` runtime APIs
- Added support for functions without output (without `return` statements) in
  `Promise.poll` runtime API
- Added custom user agent in all the HTTP requests, including downloader runtime
  API: `User-Agent: anime-games-launcher/<version>`. The HTTP runtime API can
  overwrite this value, but downloader API cannot. All the launcher's own
  requests will use this custom user agent too
- Added standard luau `number` table to support advanced arithmetic functions
  in the luau runtime. Global environment table building was slightly optimized
  too
- Added `stringify` runtime API to convert lua objects to strings and return
  them back to the lua runtime side
- Added "No games available" status in store page
- Added `str.lowercase`, `str.uppercase` and `str.trim` runtime APIs to support
  unicode characters (standard lua functions work only with ASCII)
- Added `selector` and `number` game settings entry formats
- Added search bar to `enum` game settings entry if there's 10 or more values
  available
- Added `tools.get_buttons` game integration function. Now integrations can add
  their own buttons to the library details page for different needs, e.g.
  "Delete game files", or "Open wine config". Clicking on these buttons will
  reload game info after the button's callback is executed
- Added launcher-side luau runtime garbage collection task, and related
  `runtime.collect_garbage_interval` config. While luau engine itself should
  perform garbage collection automatically, launcher will perform full
  collection cycles from time to time. This can be disabled by settings this
  config to `0`

### Fixed

- Fixed `hash.digitize_file` runtime API stack overflow
- Fixed `POST` method name in HTTP runtime API
- Fixed game scope overwriting with default values in added games' manifests
- Fixed default image rendering in horizontal lazy loadable pictures. Previously
  only vertical pictures (cards) had proper default image scaling and
  positioning
- Fixed tests and CI workflows

### Changed

- Disable human panic in debug builds
- Return `nil` in `compression.read` runtime API if nothing to read
- Changed `string.encode` and `string.decode` runtime API args order. Now
  in all the similar APIs the algorithm name goes first, arguments go next
- In game details within the store page, carousel will now hide controls if
  there's only one picture available
- Featured games are now shown before non-featured games

## [2.0.0-beta4] - 03.01.2026

### Added

- `torrent.add` API got `restart` option to restart already added torrents
- Added launcher localization using in-house `agl-locale` crate;
  English, Russian, German and Portuguese languages are supported now
- Game integrations now can be returned by a function to support lazy loading
- Added task API with `Promise` userdata type. If returned from runtime - the
  actual work happens in background and doesn't block lua engine thread
- Added `await` runtime function to resolve different lua types, including
  functions, coroutines (threads), `Promises` and more
- Added `Bytes` userdata type to replace tables of numbers used to represent
  bytes on lua side. Most of runtime API methods were reworked to return and
  accept this custom type
- Added system API to query system-related information, currently local and UTC
  time, environment variables and binaries paths

### Fixed

- Actions pipeline execution graph now resets on window close
- Fixed vertical distance between store page game cards
- Fixed `http.fetch` options parsing
- Process API now doesn't resolve the binary path and doesn't check for relative
  path

### Changed

- Force torrent API to add global torrents list to each added torrent
- Display progress bar in actions pipeline window even if current progress is 0
- Updated lua engine version; 64 bit numbers should be supported now
- Changed required GTK4 and libadwaita versions to support older linux distros
- Add more environment variables to parse system language from
- Renamed network API to HTTP API
- Most of runtime API methods were promisified (reworked to return `Promise`)
  and perform actual work in background to not to block lua engine thread
- In-RAM memory buffers for some APIs were increased for better performance
- Sqlite API now can accept functions, coroutines (threads), `Promise` and
  `Bytes` types as query params (they will be resolved into actual values)

### Removed

- Removed unused `utils` and `i18n` launcher modules

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

[unreleased]: https://github.com/an-anime-team/anime-games-launcher/compare/v2.0.0-rc1...next
[2.0.0-rc1]: https://github.com/an-anime-team/anime-games-launcher/compare/v2.0.0-beta4...v2.0.0-rc1
[2.0.0-beta4]: https://github.com/an-anime-team/anime-games-launcher/compare/v2.0.0-beta3...v2.0.0-beta4
[2.0.0-beta3]: https://github.com/an-anime-team/anime-games-launcher/compare/v2.0.0-beta2...v2.0.0-beta3
[2.0.0-beta2]: https://github.com/an-anime-team/anime-games-launcher/compare/v2.0.0-beta1...v2.0.0-beta2
[2.0.0-beta1]: https://github.com/an-anime-team/anime-games-launcher/compare/v2.0.0-alpha2...v2.0.0-beta1
[2.0.0-alpha2]: https://github.com/an-anime-team/anime-games-launcher/compare/v2.0.0-alpha1...v2.0.0-alpha2
[2.0.0-alpha1]: https://github.com/an-anime-team/anime-games-launcher/compare/v1.0.2...v2.0.0-alpha1
[1.0.2]: https://github.com/an-anime-team/anime-games-launcher/compare/v1.0.1...v1.0.2
[1.0.1]: https://github.com/an-anime-team/anime-games-launcher/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/an-anime-team/anime-games-launcher/releases/tag/v1.0.0
