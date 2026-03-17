# agl-games

Anime Games Launcher games API.

## Target platform

A target platform string consist of CPU architecture and OS family name:
`{arch}-{system}`.

List of supported platforms:

| Arch      | System    | Platform          |
| --------- | --------- | ----------------- |
| `x86_64`  | `windows` | `x86_64-windows`  |
| `aarch64` | `windows` | `aarch64-windows` |
| `x86_64`  | `linux`   | `x86_64-linux`    |
| `aarch64` | `linux`   | `aarch64-linux`   |
| `x86_64`  | `darwin`  | `x86_64-darwin`   |
| `aarch64` | `darwin`  | `aarch64-darwin`  |

Target platform string should be used to determine what systems are supported
by games.

## Game manifest

Game manifest provides metadata about the game which will be used to render the
game's store and library pages, and it also contains URL of the AGL package and
name of its output that provides the game integration module.

```json
{
    "version": 1,
    "game": {
        "title": "Example game",
        "description": {
            "en": "Example game description",
            "ru": "Пример описания игры"
        },
        "developer": "Example dev",
        "publisher": "Example publisher",
        "images": {
            "icon": "https://...",
            "poster": "https://...",
            "background": "https://...",
            "slides": [
                "https://...",
                "https://..."
            ]
        },
        "tags": [
            "gambling",
            "payments",
            "anti-cheat",
            "workarounds"
        ]
    },
    "package": {
        "url": "https://.../package.json",
        "output": "..."
    }
}
```

All the metadata strings can be written in `LocalizableString` format: as either
a plain string (without translation), or as an object of key being a language
code, and value - variant of the string for this language code. For example,
look at the `title` and `description` fields. The app will automatically use
the best translation variant using current system language.

```ts
type LocalizableString = string | { [lang: string]: string };
```

### Game tags

List of available game tags:

| Name                 | Short description                                            |
| -------------------- | ------------------------------------------------------------ |
| `gambling`           | Game has gambling elements                                   |
| `payments`           | Buying in-game items for real money                          |
| `graphic-violence`   | Game contains explicit elements including blood and injuries |
| `cooperative`        | Game has built-in multiplayer (cooperative) elements         |
| `social`             | Game has social features - online chat, VoIP, shared spaces  |
| `controller`         | Game has controllers support                                 |
| `performance-issues` | Known performance issues on some platforms                   |
| `anti-cheat`         | Client or server-side anti-cheat                             |
| `workarounds`        | Game package provides modifications to run the game          |

## Games registry

Games registry lists different game manifests. Some games can be marked as
featured (promoted) so they will be placed above others.

```json
{
    "games": [
        {
            "url": "https://.../manifest.json",
            "featured": true
        },
        {
            "url": "https://.../manifest.json",
            "featured": false
        }
    ]
}
```

## Game integration

Game integration is a lua table with the following structure:

```ts
type GameEdition = {
    // Unique name of the edition.
    name: string;

    // Title of the edition.
    title: LocalizableString;
};

type GameVariant = {
    // Current platform string (e.g. `x86_64-linux`).
    platform: string;

    // Game edition string.
    edition?: string;
};

type GameLaunchInfo = {
    // Game launching status.
    status: 'normal' | 'warning' | 'danger';

    // A text displayed on the game launching button.
    hint?: LocalizableString;

    // Path to the game binary.
    binary: string;

    // Args passed to the binary.
    args: string[];

    // Table of environment variables.
    env: { [key: string]: string };
};

type ActionsPipeline = {
    // Actions pipeline title.
    title: LocalizableString;

    // Actions pipeline description (what this pipeline is supposed to do).
    description?: LocalizableString;

    // Actions of the pipeline.
    pipeline: PipelineAction[];
};

type PipelineAction = {
    // Title of the pipeline action.
    title: LocalizableString;

    // Description of the pipeline action (what this action does).
    description?: LocalizableString;

    // Optional function executed before running the main one. If it returns
    // `true`, then the main `perform` function is called next. Otherwise, if
    // `false` is returned, then the action is skipped and the next one will be
    // executed.
    // 
    // For example, this function can check if files are already downloaded
    // before running the main function that downloads the files.
    before?: (updater: ProgressReport): boolean;

    // The main pipeline action function. Executed after the `before`.
    perform: (updater: ProgressReport);
};

type ProgressReport = {
    // Current progress.
    current: number;

    // Total progress.
    total: number;

    // Optional function to format current progress value, e.g. `13 MB/s`.
    format?: (): LocalizableString;
};

type SettingsGroup = {
    // Optional title of the settings group.
    title?: LocalizableString;

    // Optional description (subtitle) of the settings group.
    description?: LocalizableString;

    // List of available settings entries.
    entries: SettingsEntry[]
};

type SettingsEntry = {
    // Name of the settings property which will be updated when the current
    // settings entry is changed. If unset - no property will be updated.
    name?: string;

    // Title of the settings entry.
    title: LocalizableString;

    // Optional description (subtitle) of the settings entry.
    description?: LocalizableString;

    // Settings entry reactivity. By default `relaxed` is used.
    reactivity?: SettingsEntryReactivity,

    // Settings entry.
    entry: SettingsEntryVariant;
};

type SettingsEntryReactivity =
    // Do not refresh game info after changing this entry.
    | 'none'

    // Refresh game info after closing the settings window. This is the
    // default value.
    | 'relaxed'

    // Reload whole settings layout immediately after changing this settings
    // entry and refresh game info after closing it.
    | 'release';

type SettingsEntryVariant =
    | SettingsEntrySwitch
    | SettingsEntryText
    | SettingsEntryEnum
    | SettingsEntrySelector
    | SettingsEntryExpandable;

type SettingsEntrySwitch = {
    format: 'switch';
    value: boolean;
};

type SettingsEntryText = {
    format: 'text';
    value: string;
};

type SettingsEntryEnum = {
    format: 'enum';
    values: { [name: string]: LocalizableString };
    selected: string;
};

type SettingsEntrySelector = {
    format: 'selector';
    values: { [name: string]: LocalizableString };
    selected: string;
};

type SettingsEntryExpandable = {
    format: 'expandable';
    entries: SettingsEntry[];
};

type GameIntegration = {
    version: number; // current version is `1`.

    game: {
        // Get list of available game editions for the provided platform.
        get_editions?: (platform: string): GameEdition[];

        // Get game launching info if it's available. Return `null` if game
        // cannot be launched.
        get_launch_info: (variant: GameVariant): GameLaunchInfo | null;

        // Get game actions pipeline if they're available. Return `null` if game
        // doesn't have any pipeline actions.
        get_actions_pipeline: (variant: GameVariant): ActionsPipeline | null;
    };

    settings?: {
        // Get property value.
        get_property: (name: string): any;

        // Set property value.
        set_property: (name: string, value: any): void;

        // Get dynamic settings layout.
        get_layout: (variant: GameVariant): SettingsGroup[];
    };
};
```

Licensed under [GPL-3.0-or-later](./LICENSE)
