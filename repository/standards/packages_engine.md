# v1 standard of the packages engine

Packages define set of input and output resources and provide
their names and formats. Same inputs and outputs could be used
by different packages, being shared as singletons. Names are not
unique and different packages can use the same ones.

Each resource has its own "loaded format" - a lua representation
of it. Modules, special lua scripts, can obtain loaded resources
using special rust-lua bridge API.

Modules can be listed in inputs and outputs of a package. Input
module cannot obtain any loaded resource. Output modules, on the
contrary, can obtain any input resource of their parent package.
Output modules can't load themselves.

| Can load?     | Input | Output |
| ------------- | ----- | ------ |
| Input module  | No    | No     |
| Output module | Yes   | No     |

## Inputs loading

As already said all the resources are loaded only once and are stored
in global environment variable, in shared (singleton) state.
When loaded by one module and changed the changes will be visible
by other modules too.

```ts
type LoadedResource = {
    // Format of the loaded resource defined
    // in the packages manifests standard.
    format: ResourceFormat,

    // Base32 encoded hash of the resource.
    hash: string,

    // Value of the resource.
    value: any
};
```

```lua
local file = v1_packages_load('file-input')

print(file.format) -- "file"
print(file.hash)   -- "<base32 value>"
print(file.value)  -- "<path to the file>"
```
