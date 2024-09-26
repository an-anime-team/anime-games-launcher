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

## Values cloning

Since tables in lua work similarly to arrays in JS (they're shared on cloning)
it's convenient to have a function to create a full copy of some value which
will not be shared with the rest of the script.

```lua
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

## Sandboxed IO API

All the IO operations are sandboxed by both [luau](https://luau.org) engine
and rust-lua bridge API. From rust side we provide the following functions:

| Function         | Description                              |
| ---------------- | ---------------------------------------- |
| `fs.exists`      | Check if given path exists.              |
| `fs.metadata`    | Get metadata of given fs path.           |
| `fs.copy`        | Copy file or folder to a new location.   |
| `fs.move`        | Move a file or a folder.                 |
| `fs.remove`      | Remove a file or a folder.               |
| `fs.open`        | Try to open a file handle.               |
| `fs.seek`        | Set pointer in a file handle.            |
| `fs.read`        | Read bytes from a file handle.           |
| `fs.write`       | Write bytes to the file handle.          |
| `fs.flush`       | Flush file handle buffer.                |
| `fs.close`       | Close file handle.                       |
| `fs.create_file` | Create new file in a given path.         |
| `fs.read_file`   | Read content from the given file's path. |
| `fs.write_file`  | Write content to the given file's path.  |
| `fs.remove_file` | Remove file on a given path.             |
| `fs.create_dir`  | Create directory on a given path.        |
| `fs.read_dir`    | Read directory on a given path.          |
| `fs.remove_dir`  | Remove directory on a given path.        |

### `fs.exists(path. string) -> bool`

Check if given filesystem path exists and accessible.

```lua
if fs.exists("/tmp") then
    print("Temp folder exists and can be accessed")
else
    print("Temp folder doesn't exist or can't be accessed")
end
```

### `fs.metadata(path. string) -> Metadata`

Read metadata of the filesystem path (file, folder or a symlink).

```ts
type EntryType = 'file' | 'folder' | 'symlink';

type Metadata = {
    // UTC timestamp of the creation time.
    created_at: number,

    // UTC timestamp of the modification time.
    modified_at: number,

    // Length in bytes of the filesystem entry.
    // For files it's equal to the file's size.
    length: number,

    // Is the given path accessible.
    // Similar to `fs.exists`.
    is_accessible: boolean,

    // Type of the filesystem entry.
    type: EntryType
};
```

```lua
local metadata = fs.metadata("my_file.txt")

print("Size: " .. metadata.length)
print("Type: " .. metadata.type)
```

### `fs.copy(source: string, target: string)`

Copy file or folder to another location. This function will
throw an error if the target location already exists or is not
accessible.

```lua
fs.copy("my_folder", "new_location/my_folder")
```

### `fs.move(source: string, target: string)`

Move a file or a folder to another location. This function will
throw an error if the target location already exists or is not
accessible.

```lua
fs.move("my_folder", "new_location/my_folder")
```

### `fs.remove(path. string)`

Remove a file, folder or a symlink. Removing a folder will remove
all its content as well.

```lua
fs.remove("my_file.txt")
fs.remove("my_folder")
fs.remove("my_symlink")
```

### `fs.open(path. string, [options: Options]) -> number`

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
local handle = fs.open("my_file.txt", {
    create    = true,
    overwrite = true,
    write     = true
})
```

### `fs.seek(handle: number, position: number)`

Seek position in the given file handle.

Position can be negative to set offset from the end of the file.
Otherwise it's always set from the beginning of the file.

```lua
local handle = fs.open("my_file.txt")

fs.seek(10)

-- read chunk of data skipping first 10 bytes
local head = fs.read()

fs.seek(-10)

-- read last chunk of data with 10 bytes offset from the end
local tail = fs.read()

fs.close(handle)
```

### `fs.read(handle: number, [position: number, [length: number]]) -> [number]`

Read chunk of binary data from the open file handle.
Size of chunk is determined by the rust API. If 0 length
chunk is returned, then there's no more data to read.

If `position` is specified, then `fs.seek` will be used before
reading the chunk. This will affect future operations as well.
Position can be negative to set offset from the end of the file.
Otherwise it's always set from the beginning of the file.

If `length` is specified, then the chunk length will not be larger
than the given number.

```lua
local handle = fs.open("large_file.txt")
local chunk  = fs.read(handle)

while #chunk > 0 do
    -- do something with chunk of data

    chunk = fs.read(handle)
end

fs.close(handle)
```

```lua
local handle = fs.open("game_file")

-- read game version from the file (3 bytes)
local game_version = fs.read(handle, 1000, 3)

fs.close(handle)
```

### `fs.write(handle: number, content: [number], [position: number])`

Write given data to the open file at its current position.

If `position` is specified, then `fs.seek` will be used before
reading the chunk. This will affect future operations as well.
Position can be negative to set offset from the end of the file.
Otherwise it's always set from the beginning of the file.

```lua
-- file    : [ ]
-- pointer :  ^
local handle = fs.open("new_file.txt", {
    create    = true,
    overwrite = true,
    write     = true
})

-- file    : [1, 2, 3, ]
-- pointer :          ^
fs.write({ 1, 2, 3 })

-- file    : [1, 2, 3, 4, 5, 6, ]
-- pointer :                   ^
fs.write({ 4, 5, 6 })

fs.close(handle)

-- file    : [1, 2, 3, 4, 5, 6, ]
-- pointer :  ^
local handle = fs.open("new_file.txt", {
    write = true
})

-- file    : [7, 8, 9, 4, 5, 6, ]
-- pointer :           ^
fs.write({ 7, 8, 9 })

fs.close(handle)
```

```lua
-- []
local handle = fs.open("new_file.txt", {
    create    = true,
    overwrite = true,
    write     = true
})

-- file    : [1, 2, 3, 4, 5, 6, ]
-- pointer :                   ^
fs.write({ 1, 2, 3, 4, 5, 6 })

-- file    : [1, 2, 5, 4, 3, 6, ]
-- pointer :                 ^
fs.write({ 5, 4, 3 }, 2)

-- file    : [1, 2, 5, 4, 3, 7, 8, 9, ]
-- pointer :                         ^
fs.write({ 7, 8, 9 })

fs.close(handle)
```

### `fs.flush(handle: number)`

Flush file content on disk.

Write operations are performed on a small buffer in the RAM
and are flushed on disk only when the buffer is full. This
greatly improves performance of operations, but changes will
not be available for other file readers until the buffer
is flushed. This function forcely flushes the buffer on disk.

```lua
local reader = fs.open("file.txt", {
    create = true,
    read   = true
})

local writer_1 = fs.open("file.txt", {
    write = true
})

local writer_2 = fs.open("file.txt", {
    write = true
})

local writer_3 = fs.open("file.txt", {
    write = true
})

fs.write(writer_1, { 1, 2, 3 })
fs.write(writer_2, { 4, 5, 6 })
fs.write(writer_3, { 7, 8, 9 })

-- []
fs.read(reader)

fs.flush(writer_1)

-- [1, 2, 3]
fs.read(reader)

fs.flush(writer_2)

-- [4, 5, 6]
fs.read(reader)

fs.close(writer_1)
fs.close(writer_2)
fs.close(writer_3) -- writer_3 is flushed on close

-- [7, 8, 9]
fs.read(reader)

fs.close(reader)
```

### `fs.close(handle: number)`

Close the file handle. This will flush the inner buffer
of the file and prevent future use of this handle.

```lua
local handle = fs.write("my_file.txt", { write = true })

fs.write({ 1, 2, 3 })
fs.close(handle)
```

### `fs.read_file(path. string) -> [number]`

Read the whole content of a file in a given path.

> Note: do not try to read large files using this function.

```lua
local content = fs.read_file("my_file.txt")

print("Read " .. #content .. " bytes")
```

### `fs.write_file(path. string, content: [number])`

Overwrite existing file with given content, or create
a new one.

```lua
fs.write_file("my_file.txt", { 1, 2, 3 })
```

### `fs.remove_file(path. string)`

Remove file in a given path.

```lua
fs.remove_file("my_file.txt")
```

### `fs.create_dir(path. string)`

Create directory if it doesn't exist.

```lua
-- this will create all the parent directories too
fs.create_dir("a/b/c/d")
```

### `fs.read_dir(path. string) -> [Entry]`

Read the given directory, returning list of its entries.

```ts
type Entry = {
    name: string,
    path. string,
    type: EntryType
};
```

```lua
function print_dir(path, prefix)
    for _, entry in pairs(fs.read_dir(path)) do
        print(prefix .. entry.name)

        if entry.type == "folder" do
            print_dir(entry.path, prefix .. "  ")
        end
    end
end

print_dir("my_dir", "")
```

### `fs.remove_dir(path. string)`

Remove given folder and all its content.

```lua
fs.create_dir("my_dir")

print(fs.exists("my_dir")) -- true

fs.remove_dir("my_dir")

print(fs.exists("my_dir")) -- false
```

## Paths API

Filesystem paths are sandboxed by design. Each module can access
special sandboxed folders to store its state there. This module
provides functions to obtain these paths, as well as some utility
functions to work with them.

| Function           | Description                           |
| ------------------ | ------------------------------------- |
| `path.temp_dir`    | Get path to the temp directory.       |
| `path.module_dir`  | Get path to the module directory.     |
| `path.persist_dir` | Get path to the persistent directory. |
| `path.normalize`   | Remove special path components.       |
| `path.join`        | Create path from entries names.       |
| `path.parts`       | Split path to the entries names.      |
| `path.parent`      | Get path to the parent directory.     |
| `path.file_name`   | Get last path entry name.             |
| `path.exists`      | Check if given path exists.           |
| `path.accessible`  | Check if given path is accessible.    |

### `path.temp_dir() -> string`

Temp directory is configured by the user in the launcher app and
its content will eventually be automatically cleared. You can use
temp directory to store temporary data, e.g. downloaded archives.

```lua
local temp = path.temp_dir()

if fs.exists(temp .. "/.first-run") do
    -- ...
end
```

### `path.module_dir() -> string`

Each module has its own directory which cannot be accessed by any
other module. It should be used to store all its data. If module
is updated to a newer version (it hash was changed) - it will have
a new folder and it couldn't access the previous one.

Module directory can be deleted by the packages garbage collector
when the module is not used.

```lua
local store = path.module_dir()

fs.write_file(store .. "/secret_file", { 1, 2, 3 })
```

### `path.persist_dir(key: string) -> string`

Modules can get paths to the persistent data storage using special
keyword. Every module using the same keyword will get the same path.
This can be used to transfer state files from one module to another.

```lua
-- first module
local test_dir = path.persist_dir("test")

fs.create_file(test_dir .. "/example")
```

```lua
-- second module
local test_dir = path.persist_dir("test")

print(fs.exists(test_dir .. "/example")) -- true
```

### `path.normalize(path: string) -> string | nil`

Path normalization will remove all the special path components.
If path is meaningless, then nil is returned.

```lua
print(path.normalize("./test"))   -- "test"
print(path.normalize("a/b/../c")) -- "a/c"
print(path.normalize("a/b/./c"))  -- "a/b/c"
print(path.normalize("a\\b\\c"))  -- "a/b/c"

-- We don't support relative paths:
print(path.normalize("."))  -- nil
print(path.normalize("..")) -- nil
```

### `path.join(parts: ...string) -> string | nil`

Create new path by combining given entries names. This function
will normalize the result path as well. If no parts were given
or they're meaningless - nil is returned.

```lua
local dir = path.join(path.module_dir(), "download")

print(path.join())     -- nil
print(path.join("."))  -- nil
print(path.join("..")) -- nil
```

### `path.parts(path: string) -> [string] | nil`

Split given filesystem entry path to the components (entries names).
This function will normalize the path before splitting it. If input
string is empty or meaningless - nil is returned.

```lua
-- ["a", "c"]
print(path.parts("a/b\\../c/./"))

print(path.parts(""))   -- nil
print(path.parts("."))  -- nil
print(path.parts("..")) -- nil
```

### `path.parent(path: string) -> string | nil`

Return parent folder path or nil if it doesn't exist. Return path
will be normalized.

```lua
print(path.parent("a/./b")) -- "a"
print(path.parent("a"))     -- nil
```

### `path.file_name(path: string) -> string | nil`

Return the last entry name of the given path. Return nil if the input
string is meaningless.

```lua
print(path.file_name("a/b/c"))          -- "c"
print(path.file_name("a/./b/../../c/")) -- "c"

print(path.file_name("."))  -- nil
print(path.file_name("..")) -- nil
```

### `path.exists(path: string) -> bool`

Check if given path exists on the disk. This function,
unlike `fs.exists`, doesn't check if the given path is
accessible for the current module, so you can use it
to verify if some system libraries or binaries are
presented on the user's system.

```lua
print(path.exists(path.module_dir())) -- true
print(path.exists("/home"))           -- true
```

### `path.accessible(path: string) -> bool`

Check if given path is accessible for the current module.

```lua
print(path.accessible(path.module_dir())) -- true
print(path.accessible("/home"))           -- false
```
