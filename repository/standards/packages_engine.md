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
| `fs:seek`        | Set pointer in a file handle.            |
| `fs:read`        | Read bytes from a file handle.           |
| `fs:write`       | Write bytes to the file handle.          |
| `fs:flush`       | Flush file handle buffer.                |
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

### `fs:open(path: string, [options: Options]) -> number`

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

### `fs:seek(handle: number, position: number)`

Seek position in the given file handle.

Position can be negative to set offset from the end of the file.
Otherwise it's always set from the beginning of the file.

```lua
local handle = fs:open("my_file.txt")

fs:seek(10)

-- read chunk of data skipping first 10 bytes
local head = fs:read()

fs:seek(-10)

-- read last chunk of data with 10 bytes offset from the end
local tail = fs:read()

fs:close(handle)
```

### `fs:read(handle: number, [position: number, [length: number]]) -> [number]`

Read chunk of binary data from the open file handle.
Size of chunk is determined by the rust API. If 0 length
chunk is returned, then there's no more data to read.

If `position` is specified, then `fs:seek` will be used before
reading the chunk. This will affect future operations as well.
Position can be negative to set offset from the end of the file.
Otherwise it's always set from the beginning of the file.

If `length` is specified, then the chunk length will not be larger
than the given number.

```lua
local handle = fs:open("large_file.txt")
local chunk  = fs:read(handle)

while #chunk > 0 do
    -- do something with chunk of data

    chunk = fs:read(handle)
end

fs:close(handle)
```

```lua
local handle = fs:open("game_file")

-- read game version from the file (3 bytes)
local game_version = fs:read(handle, 1000, 3)

fs:close(handle)
```

### `fs:write(handle: number, content: [number], [position: number])`

Write given data to the open file at its current position.

If `position` is specified, then `fs:seek` will be used before
reading the chunk. This will affect future operations as well.
Position can be negative to set offset from the end of the file.
Otherwise it's always set from the beginning of the file.

```lua
-- file    : [ ]
-- pointer :  ^
local handle = fs:open("new_file.txt", {
    create    = true,
    overwrite = true,
    write     = true
})

-- file    : [1, 2, 3, ]
-- pointer :          ^
fs:write({ 1, 2, 3 })

-- file    : [1, 2, 3, 4, 5, 6, ]
-- pointer :                   ^
fs:write({ 4, 5, 6 })

fs:close(handle)

-- file    : [1, 2, 3, 4, 5, 6, ]
-- pointer :  ^
local handle = fs:open("new_file.txt", {
    write = true
})

-- file    : [7, 8, 9, 4, 5, 6, ]
-- pointer :           ^
fs:write({ 7, 8, 9 })

fs:close(handle)
```

```lua
-- []
local handle = fs:open("new_file.txt", {
    create    = true,
    overwrite = true,
    write     = true
})

-- file    : [1, 2, 3, 4, 5, 6, ]
-- pointer :                   ^
fs:write({ 1, 2, 3, 4, 5, 6 })

-- file    : [1, 2, 5, 4, 3, 6, ]
-- pointer :                 ^
fs:write({ 5, 4, 3 }, 2)

-- file    : [1, 2, 5, 4, 3, 7, 8, 9, ]
-- pointer :                         ^
fs:write({ 7, 8, 9 })

fs:close(handle)
```

### `fs:flush(handle: number)`

Flush file content on disk.

Write operations are performed on a small buffer in the RAM
and are flushed on disk only when the buffer is full. This
greatly improves performance of operations, but changes will
not be available for other file readers until the buffer
is flushed. This function forcely flushes the buffer on disk.

```lua
local reader = fs:open("file.txt", {
    create = true,
    read   = true
})

local writer_1 = fs:open("file.txt", {
    write = true
})

local writer_2 = fs:open("file.txt", {
    write = true
})

local writer_3 = fs:open("file.txt", {
    write = true
})

fs:write(writer_1, { 1, 2, 3 })
fs:write(writer_2, { 4, 5, 6 })
fs:write(writer_3, { 7, 8, 9 })

-- []
fs:read(reader)

fs:flush(writer_1)

-- [1, 2, 3]
fs:read(reader)

fs:flush(writer_2)

-- [4, 5, 6]
fs:read(reader)

fs:close(writer_1)
fs:close(writer_2)
fs:close(writer_3) -- writer_3 is flushed on close

-- [7, 8, 9]
fs:read(reader)

fs:close(reader)
```

### `fs:close(handle: number)`

Close the file handle. This will flush the inner buffer
of the file and prevent future use of this handle.

```lua
local handle = fs:write("my_file.txt", { write = true })

fs:write({ 1, 2, 3 })
fs:close(handle)
```

### `fs:read_file(path: string) -> [number]`

Read the whole content of a file in a given path.

> Note: do not try to read large files using this function.

```lua
local content = fs:read_file("my_file.txt")

print("Read " .. #content .. " bytes")
```

### `fs:write_file(path: string, content: [number])`

Overwrite existing file with given content, or create
a new one.

```lua
fs:write_file("my_file.txt", { 1, 2, 3 })
```

### `fs:remove_file(path: string)`

Remove file in a given path.

```lua
fs:remove_file("my_file.txt")
```

### `fs:create_dir(path: string)`

### `fs:read_dir(path: string) -> [Entry]`

### `fs:remove_dir(path: string)`

TBD
