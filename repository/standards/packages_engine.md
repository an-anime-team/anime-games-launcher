# v1 standard of the packages engine

## Inputs loading

Each package can load its input resources using their names.
Packages can't load resources that weren't specified in their
inputs.

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
