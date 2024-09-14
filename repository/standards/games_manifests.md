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
            background: string
        }
    },

    package: {
        // URL to the game integration package.
        url: string
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
    // Game has a scenes of gambling or has game mechanics
    // related to gambling (wishes, banners, etc.)
    | 'gambling'

    // Game can accept real money for in-game content.
    | 'payments'

    // Game contains scenes of violence.
    | 'violence'

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
    | 'compatibility-layer';

// If just a string, then it will be used despite selected
// launcher locale.
//
// If an object, then the value under the selected locale
// will be used, or, if not set, fallback to en-us.
type Localizable = string | [locale: string]: string;
```
