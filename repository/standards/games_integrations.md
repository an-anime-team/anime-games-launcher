# v1 standard of the games integrations

Game integration is a specially encoded lua table containing
set of pre-defined functions which could be used by the launcher
to display information about the game and perform actions with it.

Game integrations are provided by the top-level packages (modules).
If a package depends on another game integration - it will not be
used in the launcher.

## Game integration format

Top-level modules are expected to return the game integration object:

```ts
type GameIntegration = {
    standard: 1,

    // List of available game editions.
    editions: (): Edition[],

    // List of game components.
    components: (): Component[],

    game: {
        // Get status of the game installation.
        get_status: (edition: string): InstallationStatus,

        // Get installation diff. If no diff
        // available (not required) - return nil.
        get_diff: (edition: string): InstallationDiff | null,

        // Get params used to launch the game.
        get_launch_info: (edition: string): GameLaunchInfo
    }
};
```

## Game editions

Many games have different regional variants: a global version,
Chinese version, version for South-East Asia and so on. Editions
system allow you to define different logic for different game
variants instead of making several packages for the same game.

Game editions are displayed in the library on the game's details
page. Users can freely select any available one. When only one
game edition is available - it will be used by default.

> `Localizable` type is defined in the games manifests standard.

```ts
type Edition = {
    // Unique name of the edition.
    name: string,

    // Title used in UI.
    title: Localizable
};
```

## Game launching

Game starts in a special sandboxed environment with parameters
defined in the launcher (used profile). Integration scripts
can't directly modify the used environment or command used to
start the game. Instead they're only allowed to return the information
that could be used by the launcher to start the game.

Sometimes you want to warn users about the game's status,
e.g. if there's a chance that the user's account can be damaged
due to some reasons. For this purpose you could use optional
`status` and `hint` fields.

| Status      | Color  | Description                |
| ----------- | ------ | -------------------------- |
| `normal`    | Blue   | Used by default.           |
| `warning`   | Yellow | User's attention requried. |
| `dangerous` | Red    | User's approval required.  |
| `disabled`  | Grey   | Game can't be started.     |

```ts
type GameLaunchInfo = {
    // Optional status of the game launch button.
    status?: 'normal' | 'warning' | 'dangerous' | 'disabled',

    // Optional hint displayed nearby the launch button.
    hint?: Localizable,

    // Path to the binary, absolute or relative
    // to the module's folder.
    binary: string,

    // Arguments for the binary.
    args?: string[],

    // Environment variables applied for the binary.
    env?: [key: string]: string
};
```

## Game components

Components are separate parts used by the game or the
integration script. This can be a special binary without which
the game cannot be launched, or an optional part of
the game - e.g. a language pack.

```ts
type Component = {
    // Unique name of the component.
    name: string,

    // Title used in UI.
    title: Localizable,

    // Optional description of the component.
    description?: Localizable,

    // Optional field. If set, then component will be
    // forcely installed by the launcher.
    required?: boolean | ((): boolean),

    // Optional field. When specified, components with
    // greater value are installed (updated) first.
    priority?: number | ((): number),

    // Get status of the component installation.
    get_status: (): InstallationStatus,

    // Get installation diff of the component.
    // If no diff available (not required) - return nil.
    get_diff: (): InstallationDiff | null
};
```

## Installation diffs

Integration script must return information about the game's
installation and all the available components. Launcher
calls script-defined functions to obtain this information
and perform needed actions.

```ts
type InstallationStatus =
    // Latest component version is installed.
    | 'installed'

    // Component is installed but there's an
    // optional update available.
    | 'update-available'

    // Component is installed but there's an update
    // available that must be installed.
    | 'update-required'

    // Component is installed but there's an update
    // which cannot be installed automatically.
    | 'update-unavailable'

    // Component is not installed.
    | 'not-installed';

type InstallationDiff = {
    // Title of the diff.
    title: Localizable,

    // Optional description of the diff.
    description?: Localizable,

    // List of actions which will be executed to apply the diff.
    pipeline: PipelineAction[]
};
```

## Actions pipeline

Actions pipelines are used as a general way of applying
updates or installing the games or components. Pipeline
is a list of actions, where actions are lua functions.
Launcher execute these functions in provided order.

Example pipeline: `download -> extract -> apply_patches -> verify`.

Pipeline actions can return their execution progress back
to the launcher using provided status updating callback.

```ts
type PipelineAction = {
    // Title of the action.
    // 
    // Example: "Download"
    title: Localizable,

    // Optional description of the action.
    // 
    // Example: "Download base game files"
    description?: Localizable,

    // Optional hook used before launching the action.
    // 
    // If `true` is returned, then the action should be started.
    // If `false`, then the action should be skipped.
    before?: (progress: ProgressReporter): boolean,

    // Perform the action.
    perform: (progress: ProgressReporter): void,

    // Optional hook used after the action.
    // 
    // If `true` is returned, then the pipeline should continue execution.
    // If `false`, then all the following actions should be skipped.
    after?: (progress: ProgressReporter): boolean
};

type ProgressReporter = (status: ProgressReport): void;

type ProgressReport = {
    // Optional title of the current action.
    // 
    // Example: "Downloading <something>"
    title?: Localizable,

    // Optional description of the current action.
    // 
    // Example: "Downloading base game files"
    description?: Localizable,

    // Progress status.
    progress: {
        // Current progress.
        current: number,

        // Total progress.
        total: number,

        // Optional progress formatter. When set,
        // it will be used to generate output for
        // the launcher's UI.
        format?: (): Localizable
    }
};
```
