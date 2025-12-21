# Luau runtime standard

Packages define set of input and output resources and provide their names,
formats and hashes. Same inputs and outputs can be used by different packages, 
being shared as singletons. Names are not unique and different packages can use
the same names for different resources.

Output lua or luau files will be loaded as modules. They can access input
resources of the package.

## Inputs loading

All the resources are loaded only once and are stored in global environment 
variable in shared (singleton) state. When loaded by one module and changed the 
changes will be visible to all the other modules.

Resources can be loaded using `load` function.

```ts
type LoadedResource = {
    // Format of the loaded resource.
    format: ResourceFormat,

    // Base32 encoded hash of the resource.
    hash: string,

    // Value of the resource.
    value: any
};
```

```luau
local file = load("file-input")

print(file.format) -- "file"
print(file.hash)   -- "<base32 value>"
print(file.value)  -- "<path to the file>"
```

## Values cloning

Since tables in lua work similarly to arrays in JS (they're shared on cloning)
it's convenient to have a function to create a full copy of some value which
will not be shared with the rest of the script. This is called a deep copy.

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

## Sandbox module scope

Some APIs naturally allow modules to escape the sandbox and directly affect
the user's system. By default such APIs are not available to modules.

There's currently no standard way to enable such APIs (TBD).

## Backward compatibility

There's no promise to keep backward compatibility with old runtime versions,
although some effort will definitely be made to minimize the changes. To support
both old and new runtime versions your luau modules should use `versions` table
provided by the runtime:

| Field              | Meaning                                              |
| ------------------ | ---------------------------------------------------- |
| `versions.core`    | Version of the Anime Games Launcher core library.    |
| `versions.runtime` | Version of the Anime Games Launcher runtime library. |

For each runtime change some migration guide will be provided. It's also
recommended to implement some abstract polyfill libraries which would simplify
migration process.

## Available APIs

List of all available APIs:

| Name           | Prefix       | Description                                    |
| -------------- | ------------ | ---------------------------------------------- |
| String API     | `str`        | String conversions and data serialization.     |
| Path API       | `path`       | Paths construction and resolution.             |
| Filesystem API | `fs`         | Sandboxed filesystem manipulations.            |
| Network API    | `net`        | Perform HTTP requests.                         |
| Downloader API | `downloader` | HTTP files downloader.                         |
| Archive API    | `archive`    | Archives extraction.                           |
| Hash API       | `hash`       | Hash values calculation.                       |
| SQLite API     | `sqlite`     | SQLite databases management.                   |
| Portal API     | `portal`     | Sandboxed application and system interactions. |
| Process API    | `process`    | Binaries execution.                            |
