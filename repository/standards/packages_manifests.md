# v1 standard of the packages manifests

Packages are needed to allow games integration scripts implementers
to share their parts of code between different games which depend
on the same or similar functionality, and to require external applications
(e.g. hdiff to apply diff changes in a certain game).

Packages are designed in similar to nix packages format, where each
has a set of inputs and outputs with constant hashes to pin the versions.

Inputs and outputs are files, archives or another packages. Each will
be hashed, and hashes will be stored in the lock file to proof consistency.
Hashes are also used to locate packages in one single storage folder.

Hashes of packages are calculated as inverse hashes of the package's
manifest file. Bit inversion is needed to differ `package.json` as a
package source file and `package.json` as a "file" format resource.

## Manifest format

```ts
type PackageManifest = {
    standard: 1,

    package?: {
        // List of package maintainers.
        // Recommended formats:
        // - "John Doe"
        // - "John Doe <john.doe@gmail.com>"
        maintainers?: string[]
    },

    // Table of inputs for the package.
    // Names are used by this package to get
    // the dependencies.
    inputs?: [name: string]: Resource,

    // Table of outputs of the package.
    // Names are used by other packages which depend
    // on the current one.
    outputs: [name: string]: Resource
};

// URI of the resource or detailed description of it.
type Resource = string | {
    // URI of the resource.
    // It can either be a relative path to a file
    // or a URL to file to download.
    uri: string,

    // Format of the resource.
    format: ResourceFormat,

    // Base32 hash value.
    hash?: string
};

type ResourceFormat =
    // Reference to another package.
    | 'package'

    // Lua script that will be loaded into the engine
    // and could be used by other modules. A singleton.
    | 'module'

    // Raw file.
    | 'file'

    // Archives.
    | 'archive'
    | 'archive/tar'
    | 'archive/zip'
    | 'archive/7z';
```

## Lock file format

Lock files are used to snapshot a state of the global packages storage.
Lock files should be loaded at the start of the launcher and
used to validate the status of all the packages for all the games
by comparing their hashes (or sizes for higher speed). If some package
from the lock file is missing in the storage - it could either be
re-downloaded, or launcher could fallback to the previous lock file.
This is called a "generations" system.

```ts
type LockFileManifest = {
    standard: 1,

    metadata: {
        // UTC timestamp of lock file generation time.
        generated_at: number
    },

    // List of base32 hashes of the root packages
    // which were used to produce the dependency graph.
    root: string[],

    // List of all the packages and resources.
    resources: ResourceLock[]
};

// Information about the resource's lock.
type ResourceLock = {
    // Link used to download the resource.
    url: string,

    // Format of the resource.
    // If not specified originally, then
    // automatically assigned.
    format: ResourceFormat,

    // Lock information about the resource.
    lock: {
        // Base32 hash of it.
        hash: string,

        // Size in bytes of downloaded resource.
        size: number
    },

    // Table of inputs names and hashes of
    // imported resources of the current package.
    inputs?: [name: string]: string,

    // Table of outputs names and hashes of
    // exported resources of the current package.
    outputs?: [name: string]: string
};
```
