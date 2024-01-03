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

### Required APIs (should be implemented by the maintainer)

| API | Method | Output | Description |
| - | - | - | - |
| Visual | | | Visual representation of the game in the launcher |
| | `v1_visual_get_card_picture` | `string` | Get card picture URI for the game |
| | `v1_visual_get_background_picture` | `string` | Get background picture URI for the game |
| Game | | | Base game manipulations |
| | `v1_game_get_editions_list()` | `Edition[]` | Get list of game editions |
| | `v1_game_get_info(path)` | `GameInfo \| null` | Get installed game info |
| | `v1_game_get_diff(path)` | `Diff \| null` | Get game version diff |
| | `v1_game_post_process_diff` | | Post-process game after unpacking (installing) the diff |
| DLC | | | Manipulate with additional game content (e.g. voice packages) |
| ? | `v1_dlc_get_info(path, dlc)` | | Get installed DLC info |
| ? | `v1_dlc_get_latest_info(edition)` | | Get list of available DLCs |
| Tasks | | | Manipulate with the tasks (game downloading, etc.) |
| ? | `v1_tasks_create_from_game_diff(diff)` | | Create new task from the game diff |
| ? | `v1_tasks_create_from_dlc_diff(dlc, diff)` | | Create new task from the DLC diff |
| ? | `v1_tasks_get_status(id)` | | Get tasks's status and completion progress |

### Optional APIs (can be ignored)

| API | Method | Output | Description |
| - | - | - | - |
| Game | | | Base game manipulations |
| | `v1_game_post_process_diff` | | Post-process game after unpacking (installing) the diff |

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
