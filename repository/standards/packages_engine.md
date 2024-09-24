# v1 standard of the packages engine

Packages define set of input and output resources and provide
their names and formats. Same inputs and outputs could be used
by different packages, being shared as singletons. Names are not
unique and different packages can use the same ones.

Each resource has its own "loaded format" - a lua representation
of it. Modules, special luau scripts, can obtain loaded resources
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
-- v1 module standard.
local file = load('file-input')

print(file.format) -- "file"
print(file.hash)   -- "<base32 value>"
print(file.value)  -- "<path to the file>"
```

## Sandboxed IO API

All the IO operations are sandboxed by both [luau](https://luau.org) engine
and rust-lua bridge API.

| Function         | Description                              |
| ---------------- | ---------------------------------------- |
| `fs:exists`      | Check if given path exists.              |
| `fs:metadata`    | Get metadata of given fs path.           |
| `fs:open`        | Try to open a file handle.               |
| `fs:read`        | Read bytes from a file handle.           |
| `fs:write`       | Write bytes to the file handle.          |
| `fs:seek`        | Set pointer in a file handle.            |
| `fs:close`       | Close file handle.                       |
| `fs:create_file` | Create new file in a given path.         |
| `fs:read_file`   | Read content from the given file's path. |
| `fs:write_file`  | Write content to the given file's path.  |
| `fs:remove_file` | Remove file on a given path.             |
| `fs:create_dir`  | Create directory on a given path.        |
| `fs:read_dir`    | Read directory on a given path.          |
| `fs:remove_dir`  | Remove directory on a given path.        |

### `fs:exists(path: string) -> bool`

Check if given filesystem path exists and accessible.

```lua
if fs:exists("/tmp") then
    print("Temp folder exists and can be accessed")
else
    print("Temp folder doesn't exist or can't be accessed")
end
```

### `fs:metadata(path: string) -> Metadata`

Read metadata of the filesystem path (file, folder or a symlink).

```ts
type Metadata = {
    // UTC timestamp of the creation time.
    created_at: number,

    // UTC timestamp of the modification time.
    modified_at: number,

    // Length in bytes of the filesystem entry.
    // For files it's equal to the file's size.
    length: number,

    // Is the given path accessible.
    // Similar to `fs:exists`.
    is_accessible: boolean
};
```

```lua
local metadata = fs:metadata("my_file.txt")

print("File size: " .. metadata.length)
```

### `fs:open(path: string, [options: Options]) -> u64`

Open a file handle.

Handle is a randomly generated number associated with the file reader.
Modules have limited amount of simultaniously open handles.

```ts
type Options = {
    // Allow reading content from file.
    // Default: true.
    read: boolean,

    // Allow writing content to file.
    // Default: false.
    write: boolean,

    // Create file if it doesn't exist.
    // Default: false.
    create: boolean,

    // Clear file's content or create an empty one.
    // Default: false.
    overwrite: boolean,

    // Append writes to the end of the existing file's content.
    // Default: false.
    append: boolean
};
```

```lua
-- Create a new file or clear already existing.
local handle = fs:open("my_file.txt", {
    create    = true,
    overwrite = true,
    write     = true
})
```

TBD
