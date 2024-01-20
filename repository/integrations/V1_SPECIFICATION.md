# V1 integration specification

## Built-in APIs

| API | Methods | Description |
| - | - | - |
| Network | | Work with the network |
| | `v1_network_http_get(uri)` | Perform GET request to the given URI |
| JSON | | Work with JSON |
| | `v1_json_decode(json)` | Decode JSON string |

## Required APIs (should be implemented by the maintainer)

| API | Method | Output | Description |
| - | - | - | - |
| Visual | | | Visual representation of the game in the launcher |
| | `v1_visual_get_card_picture(edition)` | `string` | Get card picture URI for the game |
| | `v1_visual_get_background_picture(edition)` | `string` | Get background picture URI for the game |
| Game | | | Base game manipulations |
| | `v1_game_get_editions_list()` | `Edition[]` | Get list of game editions |
| | `v1_game_is_installed(game_path, edition)` | `boolean` | Check if the game is installed |
| | `v1_game_get_version(game_path, edition)` | `string \| null` | Get installed game version |
| | `v1_game_get_download(edition)` | `Download` | Get full game downloading info |
| | `v1_game_get_diff(game_path, edition)` | `Diff \| null` | Get game version diff |
| | `v1_game_get_status(game_path, edition)` | `Status \| null` | Get installed game status |
| | `v1_game_get_launch_options(game_path, addons_path, edition)` | `LaunchOptions` | Get launch options for the game |
| | `v1_game_is_running(game_path, edition)` | `bool` | Check if the game is running |
| | `v1_game_kill(game_path, edition)` | | Kill running game |
| | `v1_game_get_integrity_info(game_path, edition)` | `IntegrityInfo[]` | Get game integrity info |
| Addons | | | Additional game content manipulations |
| | `v1_addons_get_list(edition)` | `AddonsGroup[]` | Get list of available addons |
| | `v1_addons_is_installed(group_name, addon_name, addon_path, edition)` | `boolean` | Check if addon is installed |
| | `v1_addons_get_version(group_name, addon_name, addon_path, edition)` | `string \| null` | Get installed addon version |
| | `v1_addons_get_download(group_name, addon_name, edition)` | `Download \| null` | Get full addon downloading info |
| | `v1_addons_get_diff(group_name, addon_name, addon_path, edition)` | `Diff \| null` | Get addon version diff |
| | `v1_addons_get_paths(group_name, addon_name, addon_path, edition)` | `string[]` | Get installed addon files and folders paths |
| | `v1_addons_get_integrity_info(group_name, addon_name, addon_path, edition)` | `IntegrityInfo[]` | Get addon integrity info |

## Optional APIs (can be ignored)

| API | Method | Output | Description |
| - | - | - | - |
| Visual | | | |
| | `v1_visual_get_details_background_css(edition)` | `string` | Get CSS styles for game details page background |
| Hooks | | | |
| | `v1_game_diff_pre_transition(game_path, edition)` | | Process game files before creating transition |
| | `v1_game_diff_transition(transition_path, edition)` | | Process game diff files before finishing transition |
| | `v1_game_diff_post_transition(game_path, edition)` | | Process game diff files after finishing transition |
| | `v1_addons_diff_pre_transition(group_name, addon_name, addon_path, edition)` | | Process addons files before creating transition |
| | `v1_addons_diff_transition(group_name, addon_name, transition_path, edition)` | | Process addons diff files before finishing transition |
| | `v1_addons_diff_post_transition(group_name, addon_name, addon_path, edition)` | | Process addons diff files after finishing transition |
| Integrity | | | |
| | `v1_integrity_hash(algorithm, data)` | `string` | Hash input data |

## Types

For syntax highlighting types definition is written on typescript

### Edition

```ts
type Edition = {
	name: string,
	title: string
};
```

### GameInfo

```ts
type GameInfo = {
	version: string,
	edition: string
};
```

### Download

```ts
type Download = {
	version: string,
	edition: string,
	download: DiffInfo
};
```

### Diff

```ts
type Diff = {
	current_version: string,
	latest_version: string,
	edition: string,
	status: DiffStatus,

	// Isn't needed if the current version is latest
	diff?: DiffInfo
};
```

### DiffStatus

```ts
type DiffStatus = 'latest' | 'outdated' | 'unavailable';
```

| Value | Description |
| - | - |
| `latest` | Installed component version is latest |
| `outdated` | Component update is available |
| `unavailable` | The component is outdated, but there's no update available (e.g. too outdated version) |

### DiffInfo

```ts
type DiffInfo = {
	type: DiffType,
	size: number,

	// URI if type is `archive`
	uri?: string,

	// List of segments URIs if type is `segments`
	segments?: string[],

	// List of files if type is `files`
	files?: FileDownload[]
};
```

### DiffType

```ts
type DiffType = 'archive' | 'segments' | 'files';
```

| Value | Description |
| - | - |
| `archive` | Single archive with all updated files |
| `segments` | Segmented archive |
| `files` | List of files needed to be downloaded |

### FileDownload

```ts
type FileDownload = {
	path: string,
	uri: string,
	size: number
};
```

### Status

```ts
type Status = {
	allow_launch: boolean,
	severity: 'critical' | 'warning' | 'none',
	reason?: string
};
```

### LaunchOptions

```ts
type LaunchOptions = {
	// Path to the executable
	executable: string,

	// Launch options
	options: string[],

	// Table of environment variables
	environment: [variable: string]: string
};
```

### IntegrityInfo

```ts
type IntegrityInfo = {
	hash: HashType,
	value: string,
	file: FileDownload
};
```

### HashType

```ts
type HashType = 'md5' | 'sha1' | 'crc32' | 'xxhash32' | 'xxhash64' | 'xxhash3/64' | 'xxhash3/128';
```

Launcher will try to use `v1_integrity_hash` if given hash doesn't belong to the `HashType` type

### AddonsGroup

```ts
type AddonsGroup = {
	name: string,
	title: string,
	addons: Addon[]
};
```

### Addon

```ts
type Addon = {
	type: AddonType,
	name: string,
	title: string,
	version: string,
	required: boolean
};
```

### AddonType

```ts
type AddonType = 'module' | 'layer' | 'component';
```

| Value | Description |
| - | - |
| `module` | Modules are downloaded into the game folder |
| `layer` | Layers are merged with the game folder before launching the game using symlinks |
| `component` | Components are downloaded to separate folders |
