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
        ],
        "agreement": {
            "en": "Example game agreement",
            "ru": "Пример пользовательского соглашения для установки игры"
        }
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

### Game integration agreement

Each game integration can provide and optional "agreement" field. It can contain
either a real game agreement message, a warning from the integration author,
some kind of pre-requirements, instructions, or any other message that will be
shown to the user prior adding the game into their library. The user will have
to agree with it.

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
    // Optional game launching status.
    status?: 'normal' | 'warning' | 'danger';

    // A text displayed on the game launching button.
    hint?: LocalizableString;

    // Path to the game binary.
    binary: string;

    // Optional args passed to the binary.
    args?: string[];

    // Optional table of environment variables.
    env?: { [key: string]: string };

    // Optional stdout handler. If provided, the spawned process's stdout will
    // be copied to the given handler function.
    stdout?: (buf: Bytes): void;

    // Optional stderr handler. If provided, the spawned process's stderr will
    // be copied to the given handler function.
    stderr?: (buf: Bytes): void;
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

type ToolButton = {
    // Title of the button.
    title: LocalizableString;

    // Optional description (hint) of the button.
    description?: LocalizableString;

    // Lua function which will be executed when user clicks the button.
    callback: (): void;
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
    | SettingsEntrySecretText
    | SettingsEntryNumber
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

type SettingsEntrySecretText = {
    format: 'secret_text';
    value: string;
};

type SettingsEntryNumber = {
    format: 'number';
    min?: number;
    max?: number;
    step?: number;
    value: number;
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

type ComponentsGroup = {
    // Optional title of the components group.
    title?: LocalizableString;

    // Optional description (subtitle) of the components group.
    description?: LocalizableString;

    // List of available components entries.
    entries: ComponentsEntry[]
};

type ComponentsEntry = {
    // Name of the components entry.
    name: string;

    // Title of the components entry.
    title: LocalizableString;

    // Optional description (subtitle) of the components entry.
    description?: LocalizableString;

    // Whether the component is locked. If set to `true`, then the software
    // implementations of this API should avoid setting enabled/disabled state
    // for this component (it should always be considered as "enabled").
    // Therefore, underlying state handling code can always assume a locked
    // component is enabled so you don't need to store this information
    // anywhere.
    // 
    // Locked components still can be "enabled" or "disabled" if you want to
    // implement this logic, and locked components still can be "installed"
    // and "uninstalled".
    // 
    // Upstream software implementations for this API can use "uninstall"
    // functions on components to uninstall the game. For example, you can
    // provide a "Base game" "component" with locked state so it's always
    // enabled, and an "uninstall" function call would delete all the game
    // files. The upstream software implementation could use this behavior to
    // implement an "Uninstall all" button to delete all the components provided
    // by a game integration, which is, if implemented properly, equivalent of
    // an "Uninstall game data" button.
    // 
    // Default value is `false`.
    locked?: boolean;

    // Optional list of values displayed under the component. Can be used to
    // display component statistics (e.g. actual / expected size on disk),
    // its version, or any other information.
    values?: ComponentEntryValue[];
};

type ComponentEntryValue = {
    // Component value title.
    title: LocalizableString;

    // Component value.
    value: LocalizableString;

    // Optional component value description.
    description?: LocalizableString;

    // Component value status. Defines the color used for the value.
    status?: 'normal' | 'warning' | 'danger' | 'success';
};

type GameIntegration = {
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

    // Game components section can be used to define optional additions to the
    // base game. For example, you could allow users to select what voiceovers
    // should be available in addition to the main game content.
    components?: {
        // Get game components layout. These can be different game voiceovers,
        // game DLCs, optional game runtime packages, or anything else.
        get_layout: (variant: GameVariant): ComponentsGroup[];

        // Check whether the given component is enabled.
        get_enabled: (variant: GameVariant, component: string): boolean;

        // Enable or disable given component by its name.
        set_enabled: (
            variant: GameVariant, 
            component: string, 
            enabled: boolean
        ): void;

        // Optional function to install given component. When provided the
        // launcher will try to use it when the user enables a component. The
        // component can become enabled without calling this function, so you
        // should not rely on it completely and always use the actions pipeline
        // to verify game components.
        install?: (
            variant: GameVariant,
            component: string,
            updater: (updater: ProgressReport): void
        ): void;

        // Optional function to uninstall given component. When provided the
        // launcher will try to use it when the user disables a component. The
        // component can become disabled without calling this function, so you
        // should not rely on it completely and always use the actions pipeline
        // to verify game components.
        uninstall?: (
            variant: GameVariant,
            component: string,
            updater: (updater: ProgressReport): void
        ): void;
    };

    tools: {
        // Get list of extra UI buttons.
        get_buttons?: (variant: GameVariant): ToolButton[];
    };

    settings?: {
        // Get dynamic settings layout.
        get_layout: (variant: GameVariant): SettingsGroup[];

        // Get property value.
        get_property: (name: string): any;

        // Set property value.
        set_property: (name: string, value: any): void;
    };
};
```

Licensed under [GPL-3.0-or-later](./LICENSE)
