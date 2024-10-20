# v1 standard of the games manifests

Game manifest is used by the launcher to display desciption of the game. Manifests
are served by the "collections" and are fetched and constantly updated during the start
of the launcher. These manifests contain title of the game, links to the images,
list of tags, genres and all the other information about the game, and a link to the
package implementing the game's integration to the launcher.

## Manifest format

```ts
type Manifest = {
    standard: 1,

    game: {
        title: Localizable,
        description: Localizable,
        developer: Localizable,
        publisher: Localizable,

        images: {
            // URL to the game's icon in square format (e.g. 64x64 px).
            icon: string,

            // URL to the game's poster in portrait orientation (e.g. 200x300 px).
            poster: string,

            // URL to the game's background in album orientation (e.g. 1920x1080 px).
            background: string,

            // List of URLs to the game's slides displayed in the details page.
            // Slides should have album orientation (e.g. 1920x1080 px).
            slides: string[]
        }
    },

    package: {
        // URL to the game integration package.
        url: string,

        // Name of the output lua module containing the game's integration code.
        output: string,

        // Information about the profile (runtime) which should be used
        // to execute the integration script.
        runtime: {
            // Platform native for the integration script.
            // In most cases it's x86_64-windows-native
            //
            // Game will be executed without additional compatibility layers
            // if the current platform is native for it.
            platform: TargetPlatform,

            // Required features of the native platform.
            features?: PlatformFeature[],

            // Target platforms supported by the integration script.
            // In most cases it's x86_64-linux-wine64 with no required features.
            //
            // If the current platform is not native for the game, but it's supported
            // then the game will be run using special compatibility tools.
            supported?: {
                platform: TargetPlatform,
                features?: PlatformFeature[]
            }[]
        }
    },

    // Information displayed on the game's details page.
    info?: {
        hardware_requirements?: {
            minimal: HardwareRequirements,
            optimal?: HardwareRequirements
        },

        tags?: GameTag[]
    }
};

type TargetPlatform =
    // Native x86_64 windows game.
    | 'x86_64-windows-native'

    // Native x86_64 linux game.
    | 'x86_64-linux-native'

    // x86_64 windows game which can be run on linux via 32 bit wine.
    | 'x86_64-linux-wine32'

    // x86_64 windows game which can be run on linux via 64 bit wine.
    | 'x86_64-linux-wine64';

type PlatformFeature =
    // Any installed DXVK version.
    | 'wine-dxvk';

type HardwareRequirements = {
    cpu?: {
        // CPU device model name.
        model: Localizable,

        // Expected required amount of CPU cores.
        cores?: number,

        // Expected required CPU frequency, in hertz.
        frequency?: number
    },

    gpu?: {
        // GPU device model name.
        model: Localizable,

        // Expected required VRAM size, in bytes.
        vram?: number
    },

    ram?: {
        // Expected required RAM size, in bytes.
        size: number,

        // Expected required RAM frequency, in hertz.
        frequency?: number
    },

    disk?: {
        // Expected required disk size, in bytes.
        size: number,

        // Expected required disk type.
        type?: 'hdd' | 'ssd' | 'nvme'
    }
};

type GameTag =
    // Game has scenes of gambling or has game mechanics
    // related to gambling (wishes, banners, etc.)
    | 'gambling'

    // Game can accept real money for in-game content.
    | 'payments'

    // Graphic violence generally consists of any clear and uncensored
    // depiction of various violent acts. Commonly included depictions
    // include murder, assault with a deadly weapon, accidents which
    // result in death or severe injury, suicide, and torture. In all
    // cases, it is the explicitness of the violence and the injury
    // inflicted which results in it being labeled "graphic". In fictional
    // depictions, appropriately realistic plot elements are usually
    // included to heighten the sense of realism
    // (i.e. blood effects, prop weapons, CGI).
    //
    // Source: https://en.wikipedia.org/wiki/Graphic_violence
    | 'graphic-violence'

    // Game is known to have a bad performance, either
    // on any platform or on linux specifically
    // (perhaps due to bad DXVK/wine/gstreamer implementation).
    | 'performance-issues'

    // Game has an anti-cheat, either server- or client-side.
    // This tag doesn't necessary mean that this anti-cheat
    // doesn't support linux platform.
    | 'anti-cheat'

    // Game is not officially supported on linux.
    | 'unsupported-platform'

    // Game is not runnable on linux, but the integration package
    // provides set of special utilities or game files modifications
    // which make the game to function. Note that this may violate its
    // terms of service and result in taking actions on your account.
    | 'workarounds';

// If just a string, then it will be used despite selected
// launcher locale.
//
// If an object, then the value under the selected locale
// will be used, or, if not set, fallback to en-us.
type Localizable = string | [locale: string]: string;
```

## Target platforms

Most of games are made for 64 bit windows machines which makes it difficult
to run on other platforms. Several different compatibility tools exist to
play windows-native games on linux and macOS. Launcher allows integration
scripts developers to specify their platforms so launcher could automatically
decide which tools it should use to run the game.

| Platform                | Supported platforms   | Description                               |
| ----------------------- | --------------------- | ----------------------------------------- |
| `x86_64-windows-native` | `x86_64-linux-wine64` | Most of games can run on linux using wine |

## Games registry

Games registry is a standard way of storing collections of the games'
manifests. Launcher fetches all the manifests listed in the registry
and displays them on the store page.

```ts
type Manifest = {
    format: 1,

    // Title of the registry.
    title: Localizable,

    // List of games.
    games: Game[]
};

type Game = {
    // URL to the game's manifest.
    url: string,

    // When true, this game will be proposed to be displayed
    // on the top of the store page.
    featured?: boolean
};
```
