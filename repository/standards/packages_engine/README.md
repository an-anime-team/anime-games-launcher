# v1 standard of the packages engine

Packages define set of input and output resources and provide their names
and formats. Same inputs and outputs can be used by different packages, being
shared as singletons. Names are not unique and different packages can use
the same ones.

Each resource has its own "loaded format" - a lua representation of it.
Modules, special luau scripts, can obtain loaded resources using special
rust-lua bridge API.

Modules can be listed in inputs and outputs of a package. Input module cannot
obtain any loaded resource. Output modules, on the contrary, can obtain any
input resource of their parent package. Output modules can't load themselves.

| Can load?     | Input | Output |
| ------------- | ----- | ------ |
| Input module  | No    | No     |
| Output module | Yes   | No     |

## Inputs loading

As already said all the resources are loaded only once and are stored in global
environment variable, in shared (singleton) state. When loaded by one module and
changed the changes will be visible to other modules.

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

```luau
-- v1 module standard.
local file = load("file-input")

print(file.format) -- "file"
print(file.hash)   -- "<base32 value>"
print(file.value)  -- "<path to the file>"
```

## Inputs importing

Unlike loading, inputs importing doesn't fetch metadata of input resources.
You're directly loading their values instead. Importing is prefered way of
using inputs for most cases.

> From the technical aspect, `import` function uses `load` output and strips
> all the metadata from it.

```luau
local input_file   = import("file-input")
local input_module = import("module-input")

print(input_file)   -- "<path to the file>"
print(input_module) -- "<content of the module>"
```

## Values cloning

Since tables in lua work similarly to arrays in JS (they're shared on cloning)
it's convenient to have a function to create a full copy of some value which
will not be shared with the rest of the script.

```luau
local table_1 = {
    hello = "world"
}

local table_2 = table_1
local table_3 = clone(table_1)

table_1.hello = "sugoma"

print(table_1.hello) -- "sugoma"
print(table_2.hello) -- "sugoma"
print(table_3.hello) -- "world"
```

## Debug logging

To perform well-known, time-proven debug technique called "printf each line"
you can use `dbg` function. It will print all the input values into the
application's debug logger.

```luau
dbg("test", 123, { a = { hello = "world", 1 }, 2 })
```

## Extended privileges

Some APIs naturally allow modules to escape the sandbox and directly affect
the user's system. By default such APIs are not available for all the packages.
You can use authority index to specify which packages are allowed to use
such APIs. It's generally recommended to create small "safe bindings" to
extended privilege APIs with some safety checks to allow other packages to
use them.

## Available APIs

List of all available APIs:

| Name       | Prefix       | Extended privileges | Description                                    |
| ---------- | ------------ | ------------------- | ---------------------------------------------- |
| Strings    | `str`        | No                  | String conversions and data serialization.     |
| Paths      | `path`       | No                  | Paths construction and resolution.             |
| Filesystem | `fs`         | No                  | Sandboxed filesystem manipulations.            |
| Network    | `net`        | No                  | Perform HTTP requests.                         |
| Downloader | `downloader` | No                  | HTTP files downloader.                         |
| Archives   | `archive`    | No                  | Archives extraction.                           |
| Hashes     | `hash`       | No                  | Hash values calculation.                       |
| Sync       | `sync`       | No                  | Inter-packages data synchronization.           |
| SQLite     | `sqlite`     | No                  | SQLite databases management.                   |
| Portals    | `portals`    | No                  | Show notifications and ask for sandbox escape. |
| Process    | `process`    | **Yes**             | Binaries execution.                            |
