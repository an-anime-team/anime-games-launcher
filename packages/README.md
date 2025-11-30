# agl-packages

Anime Games Launcher packages manager.

Games are distributed as luau scripts, and since many components within these
scripts can be abstracted out, and mostly consist of boilerplate code, it's
a good idea to allow scripts to fetch some "dependencies", thus requiring
launcher to handle a dependency tree, acting as a packages manager. This is 
what this crate does.

This packages manager is inspired by nixos flakes. Each package's input or
output is called a "resource", it has a hash, is stored under this hash in some
filesystem folder, and cannot be mutated (its content is verified before
accessing it).

## Packages manifest

```ts
type Manifest = {
    inputs: { [name: string]: Resource };
    outputs: { [name: string]: Resource };
};

type Resource = string | {
    uri: string;
    format?: 'package' | 'file' | 'archive';
    hash?: string;
};
```

## Example package

```json
{
    "inputs": {
        "example_file": {
            "uri": "module_deps.zip",
            "format": "archive"
        }
    },
    "outputs": {
        "main": "my_module.luau"
    }
}
```

Licensed under [GPL-3.0-or-later](./LICENSE)
