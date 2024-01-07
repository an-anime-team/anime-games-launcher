# Games integration guide

## Manifest file

```json
{
	"manifest_version": "1",
	"game": {
		"name": "game-code-name",
		"title": "Formal Game Name",
		"developer": "Game Developer"
	},
	"script": {
		"path": "integration.lua",
		"version": "0.0.0",
		"standard": "1"
	}
}
```

## List of APIs

### Built-in APIs

| API | Methods | Description |
| - | - | - |
| Network | | Work with the network |
| | `v1_network_http_get(uri)` | Perform GET request to the given URI |
| JSON | | Work with JSON |
| | `v1_json_decode(json)` | Decode JSON string |

### Required APIs (should be implemented by the maintainer)

| API | Method | Output | Description |
| - | - | - | - |
| Visual | | | Visual representation of the game in the launcher |
| | `v1_visual_get_card_picture(edition)` | `string` | Get card picture URI for the game |
| | `v1_visual_get_background_picture(edition)` | `string` | Get background picture URI for the game |
| Game | | | Base game manipulations |
| | `v1_game_get_editions_list()` | `Edition[]` | Get list of game editions |
| | `v1_game_is_installed(game_path)` | `boolean` | Check if the game is installed |
| | `v1_game_get_version(game_path, edition)` | `string \| null` | Get installed game version |
| | `v1_game_get_download(edition)` | `Download` | Get full game downloading info |
| | `v1_game_get_diff(game_path, edition)` | `Diff \| null` | Get game version diff |
| | `v1_game_get_status(game_path, edition)` | `Status` | Get installed game status |
| | `v1_game_get_launch_options(game_path, edition)` | `LaunchOptions` | Get launch options for the game |
| Addons | | | Additional game content manipulations |
| | `v1_addons_get_list(edition)` | `AddonsGroup[]` | Get list of available addons |
| | `v1_addons_is_installed(group_name, addon_name, addon_path, edition)` | `boolean` | Check if addon is installed |
| | `v1_addons_get_version(group_name, addon_name, addon_path, edition)` | `string \| null` | Get installed addon version |
| | `v1_addons_get_download(group_name, addon_name, edition)` | `Download \| null` | Get full addon downloading info |
| | `v1_addons_get_diff(group_name, addon_name, addon_path, edition)` | `Diff \| null` | Get addon version diff |

### Optional APIs (can be ignored)

| Method | Description |
| - | - |
| `v1_diff_transition(path, edition)` | Process diff files before finishing transition |
| `v1_diff_post_transition(path, edition)` | Process diff files after finishing transition |

### Types

For syntax highlighting types definition is written on typescript

#### Edition

```ts
type Edition = {
	name: string,
	title: string
};
```

#### GameInfo

```ts
type GameInfo = {
	version: string,
	edition: string
};
```

#### Download

```ts
type Download = {
	version: string,
	edition: string,

	download: {
		type: DiffType,
		size: number,

		// URI if type is `archive`
		uri?: string,

		// List of segments URIs if type is `segments`
		segments?: string[],

		// List of files URIs if type is `files`
		files?: string[]
	}
};
```

#### Diff

```ts
type Diff = {
	current_version: string,
	latest_version: string,
	edition: string,
	status: DiffStatus,

	// Isn't needed if the current version is latest
	diff?: {
		type: DiffType,
		size: number,

		// URI if type is `archive`
		uri?: string,

		// List of segments URIs if type is `segments`
		segments?: string[],

		// List of files URIs if type is `files`
		files?: string[]
	}
};
```

#### DiffStatus

```ts
type DiffStatus = 'latest' | 'outdated' | 'unavailable';
```

| Value | Description |
| - | - |
| `latest` | Installed component version is latest |
| `outdated` | Component update is available |
| `unavailable` | The component is outdated, but there's no update available (e.g. too outdated version) |

#### DiffType

```ts
type DiffType = 'archive' | 'segments' | 'files';
```

| Value | Description |
| - | - |
| `archive` | Single archive with all updated files |
| `segments` | Segmented archive |
| `files` | List of files needed to be downloaded |

#### Status

```ts
type Status = {
	allow_launch: boolean,
	severity: 'critical' | 'warning' | 'none',
	reason?: string
};
```

#### LaunchOptions

```ts
type LaunchOptions = {
	// Relative path to the executable
	executable: string,

	// Table of environment variables
	environment: [variable: string]: string
};
```

#### AddonsGroup

```ts
type AddonsGroup = {
	name: string,
	title: string,
	addons: Addon[]
};
```

#### Addon

```ts
type Addon = {
	type: AddonType,
	name: string,
	title: string,
	version: string,
	required: boolean
};
```

#### AddonType

```ts
type AddonType = 'module' | 'component';
```

| Value | Description |
| - | - |
| `module` | Modules are merged into the game folder when launching the game |
| `component` | Components are installed to separate folders and are not merged to the game folder |

All the addons are downloaded to separate folders. When launching the game, however, launcher can process them differently: for example, you want to put voice packages inside the game folder - then voice packages are "modules". Launcher will create new merged folder with "base game" and "modules" together (modules can overwrite base game files). "Components", however, intended to be used outside the game folder. You can access them using integration API.
