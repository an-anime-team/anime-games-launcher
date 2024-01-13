# Table of content

- Required functions
  - v1_game_get_editions_list()
  - v1_visual_get_card_picture(edition)
  - v1_visual_get_background_picture(edition)
  - v1_game_is_installed(game_path)
  - v1_game_get_version(game_path, edition)
  - v1_game_get_download(edition)
  - v1_game_get_diff(game_path, edition)
  - v1_game_get_status(game_path, edition)
  - v1_game_get_launch_options(game_path, addons_path, edition)
  - v1_game_is_running(game_path, edition)
  - v1_game_kill(game_path, edition)
  - TODO: addons
- Optional functions
  - v1_visual_get_details_background_css(edition)

> Please note that this guide is not actively maintained and some functions may be outdated from the latest specification. If there's some question - please refer the [specification](V1_SPECIFICATION.md) instead.

# Required functions

## v1_game_get_editions_list()

```ts
function v1_game_get_editions_list(): Edition[];

type Edition = {
	name: string,
	title: string
};
```

Each game supposed to have different editions for different countries or regions. You're supposed to list each of available editions (or one if there's no different editions) here. Their `name`-s will be used as arguments to all your other functions.

### Example implementation:

```lua
-- Get list of game editions
function v1_game_get_editions_list()
  return {
    {
      ["name"]  = "global",
      ["title"] = "Global"
    },
    {
      ["name"]  = "china",
      ["title"] = "China"
    }
  }
end
```

## v1_visual_get_card_picture(edition)

```ts
function v1_visual_get_card_picture(edition: string): string;
```

This function should return a URI to the picture for the game card widget in the launcher. For better view it's recommended to use pictures with appropriate resolution. For better performance it's also recommended to use lower quality pictures.

> As a note, launcher right now doesn't accept URLs to the pictures. So you have to implement image downloading to some local folder and return a path to it.

### Example implementation:

```lua
-- Get card picture URI
function v1_visual_get_card_picture(edition)
  local uri = "https://cdn.steamgriddb.com/grid/393b37dd7097776b1b56b10897e1a054.png"
  local path = "/tmp/.genshin-" .. edition .. "-card"

  -- Return cache path if the file is already downloaded
  if io.open(path, "rb") ~= nil then
    return path
  end

  -- Download the file if it's not downloaded yet
  local file = io.open(path, "w+")

  file:write(v1_network_http_get(uri))
  file:close()

  return path
end
```

## v1_visual_get_background_picture(edition)

```ts
function v1_visual_get_background_picture(edition: string): string;
```

This function should return a URI to the background picture for the game. Launcher doesn't use this picture anywhere right now but it's planned to have some kind of "classic view" in the future.

> As a note, launcher right now doesn't accept URLs to the pictures. So you have to implement image downloading to some local folder and return a path to it.

### Example implementation:

```lua
-- Get background picture URI
function v1_visual_get_background_picture(edition)
  -- Get background picture URI using custom social_api function
  local uri = social_api(edition)["data"]["adv"]["background"]

  local path = "/tmp/.genshin-" .. edition .. "-background"

  -- Return cache path if the file is already downloaded
  if io.open(path, "rb") ~= nil then
    return path
  end

  -- Download the file if it's not downloaded yet
  local file = io.open(path, "w+")

  file:write(v1_network_http_get(uri))
  file:close()

  return path
end
```

## v1_game_is_installed(game_path)

```ts
function v1_game_is_installed(game_path: string): boolean;
```

This function should check if the game is installed. You don't need to implement a complicated method of identifying what "installed" is - will be enough to check existance of some important game file. Launcher calls this method often to identify the game's status.

### Example implementation:

```lua
-- Check if the game is installed
function v1_game_is_installed(game_path)
  return io.open(game_path .. "/UnityPlayer.dll", "rb") ~= nil
end
```

## v1_game_get_version(game_path, edition)

```ts
function v1_game_get_version(game_path: string, edition: string): ?string;
```

Each game has some kind of version system. This function should parse installed game version (if the game is installed) and return it to the launcher.

### Example implementation:

```lua
-- Get installed game version
function v1_game_get_version(game_path, edition)
  -- Path to the game file which contains its version
  -- get_edition_data_folder is a custom function
  local file = io.open(game_path .. "/" .. get_edition_data_folder(edition) .. "/globalgamemanagers", "rb")

  -- Return nil if this file doesn't exist (game is not installed)
  if not file then
    return nil
  end

  -- Parse the version number
  file:seek("set", 4000)

  return file:read(10000):gmatch("[1-9]+[.][0-9]+[.][0-9]+")()
end
```

## v1_game_get_download(edition)

```ts
function v1_game_get_download(edition: string): Download;

type Download = {
	version: string,
	edition: string,
	download: DiffInfo
};

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

type DiffType = 'archive' | 'segments' | 'files';
```

This function should return a table with information for downloading the plain game. Launcher supports different formats. You can download games as single archives, as segmented archives (splitted in multiple files), or file by file.

### DiffType values

| Value | Description |
| - | - |
| `archive` | Single archive with all updated files |
| `segments` | Segmented archive |
| `files` | List of files needed to be downloaded |

### Example implementation:

```lua
-- Get full game downloading info
function v1_game_get_download(edition)
  local latest_info = game_api(edition)["data"]["game"]["latest"]
  local segments = {}
  local size = 0

  for _, segment in pairs(latest_info["segments"]) do
    table.insert(segments, segment["path"])

    size = size + segment["package_size"]
  end

  return {
    ["version"] = latest_info["version"],
    ["edition"] = edition,
  
    ["download"] = {
      ["type"]     = "segments",
      ["size"]     = size,
      ["segments"] = segments
    }
  }
end
```

## v1_game_get_diff(game_path, edition)

```ts
function v1_game_get_diff(game_path: string, edition: string): ?Diff;

type Diff = {
	current_version: string,
	latest_version: string,
	edition: string,
	status: DiffStatus,

	// Isn't needed if the current version is latest
	diff?: DiffInfo
};

type DiffStatus = 'latest' | 'outdated' | 'unavailable';
```

> DiffInfo type is described above.

This function should compare installed game version (if it is installed) with latest available and if they're not the same (installed game is outdated) - return the difference between version. This difference should contain `DiffInfo` object to install the update, or notify the launcher that the game is too outdated and cannot be updated.

### DiffStatus values

| Value | Description |
| - | - |
| `latest` | Installed component version is latest |
| `outdated` | Component update is available |
| `unavailable` | The component is outdated, but there's no update available (e.g. too outdated version) |

### Example implementation:

```lua
-- Get game version diff
function v1_game_get_diff(game_path, edition)
  local installed_version = v1_game_get_version(game_path, edition)

  if not installed_version then
    return nil
  end

  local game_data = game_api(edition)["data"]["game"]

  local latest_info = game_data["latest"]
  local diffs = game_data["diffs"]

  -- It should be impossible to have higher installed version
  -- but just in case I have to cover this case as well
  if compare_versions(installed_version, latest_info["version"]) ~= -1 then
    return {
      ["current_version"] = installed_version,
      ["latest_version"]  = latest_info["version"],

      ["edition"] = edition,
      ["status"]  = "latest"
    }
  else
    for _, diff in pairs(diffs) do
      if diff["version"] == installed_version then
        return {
          ["current_version"] = installed_version,
          ["latest_version"]  = latest_info["version"],

          ["edition"] = edition,
          ["status"]  = "outdated",

          ["diff"] = {
            ["type"] = "archive",
            ["size"] = diff["package_size"],
            ["uri"]  = diff["path"]
          }
        }
      end
    end

    return {
      ["current_version"] = installed_version,
      ["latest_version"]  = latest_info["version"],

      ["edition"] = edition,
      ["status"]  = "unavailable"
    }
  end
end
```

## v1_game_get_status(game_path, edition)

```ts
function v1_game_get_status(game_path: string, edition: string): ?Status;

type Status = {
	allow_launch: boolean,
	severity: 'critical' | 'warning' | 'none',
	reason?: string
};
```

This function should say launcher how it should treat installed game. You can use it to forbid user to launch the game, e.g. before an addon is not installed or some additional actions from the user side are required.

### Example implementation:

```lua
-- Get installed game status before launching it
function v1_game_get_status(game_path, edition)
  return {
    ["allow_launch"] = true,
    ["severity"] = "none"
  }
end
```

## v1_game_get_launch_options(game_path, addons_path, edition)

```ts
function v1_game_get_launch_options(game_path: string, addons_path: string, edition: string): LaunchOptions;

type LaunchOptions = {
	// Path to the executable
	executable: string,

	// Launch options
	options: string[],

	// Table of environment variables
	environment: [variable: string]: string
};
```

This function should say the launcher how to launch the game. If you launch the game using some addon (standalone executable) - you can access it using `addons_path` variable.

### Example implementation:

```lua
-- Get game launching options
function v1_game_get_launch_options(game_path, addons_path, edition)
  local executable = {
    ["global"] = "GenshinImpact.exe",
    ["china"]  = "YuanShen.exe"
  }

  return {
    ["executable"]  = executable[edition],
    ["options"]     = {},
    ["environment"] = {}
  }
end
```

## v1_game_is_running(game_path, edition)

```ts
function v1_game_is_running(game_path: string, edition: string): boolean;
```

This function should check if installed game is running.

### Example implementation:

```lua
-- Check if the game is running
function v1_game_is_running(game_path, edition)
  local process_name = {
    ["global"] = "GenshinImpact.e",
    ["china"]  = "YuanShen.exe"
  }

  local handle = io.popen("ps -A", "r")
  local result = handle:read("*a")

  handle:close()

  return result:find(process_name[edition])
end
```

## v1_game_kill(game_path, edition)

```ts
function v1_game_kill(game_path: string, edition: string);
```

This function should kill the game process if it is running.

### Example implementation:

```lua
-- Kill running game process
function v1_game_kill(game_path, edition)
  local process_name = {
    ["global"] = "GenshinImpact.e",
    ["china"]  = "YuanShen.exe"
  }

  os.execute("pkill -f " .. process_name[edition])
end
```

TODO: addons documentation

# Optional functions

## v1_visual_get_details_background_css(edition)

```rs
fn v1_visual_get_details_background_css(edition: &str) -> String;
```

This function should return a URI to the background picture for the game. Launcher doesn't use this picture anywhere right now but it's planned to have some kind of "classic view" in the future.

### Example implementation:

```lua
-- Get CSS styles for game details background
function v1_visual_get_details_background_css(edition)
  return "background: radial-gradient(circle, rgba(168,144,111,1) 30%, rgba(88,88,154,1) 100%);"
end
```
