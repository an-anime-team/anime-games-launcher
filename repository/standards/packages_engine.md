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

```lua
local input_file   = import("file-input")
local input_module = import("module-input")

print(input_file)   -- "<path to the file>"
print(input_module) -- "<content of the module>"
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

## Debug logging

To perform well-known, time-proven debug technique called "printf each line"
you can use `dbg` function. It will print all the input values into the
application's debug logger.

```lua
dbg("test", 123, { a = { hello = "world", 1 }, 2 })
```

## Extended privileges

Every package can be digitally signed by the launcher developer. Modules in
signed packages obtain additional privileges that allow them to escape the
sandbox and perform arbitrary code execution on the host system. This is
often necessary for advanced features like applying patches in special formats.

Signed packages are always maintained directly by the launcher developer.

## Available APIs

List of all available APIs:

| Name       | Prefix       | Extended privileges | Description                                |
| ---------- | ------------ | ------------------- | ------------------------------------------ |
| Strings    | `str`        | No                  | String conversions and data serialization. |
| Paths      | `path`       | No                  | Paths construction and resolution.         |
| Filesystem | `fs`         | No                  | Sandboxed filesystem manipulations.        |
| Network    | `net`        | No                  | HTTP requests.                             |
| Downloader | `downloader` | No                  | HTTP files downloader.                     |
| Archives   | `archive`    | No                  | Archives extraction.                       |
| Hashes     | `hash`       | No                  | Hash values calculation.                   |
| Sync       | `sync`       | No                  | Inter-packages data synchronization.       |
| Process    | `process`    | **Yes**             | Binaries execution.                        |

## Strings API

Rust-lua bridge API is designed to return raw bytes instead of strings.
While many methods can accept strings, as well as other data types, as
their inputs, outputs always return tables of bytes. String API provides
functions to perform bytes-string conversions, support for data encoding
and serialization.

| Function         | Description                      |
| ---------------- | -------------------------------- |
| `str.to_bytes`   | Convert value to a bytes slice.  |
| `str.from_bytes` | Convert bytes slice to a string. |
| `str.encode`     | Encode value to a string.        |
| `str.decode`     | Decode value from a string.      |

### Supported encodings

Following table contains list of `StringEncoding` enum values.

| Name                   | Description                                          |
| ---------------------- | ---------------------------------------------------- |
| `base16` or `hex`      | Convert bytes array to base16 (hex) string.          |
| `base32`               | Convert bytes array to base32 string (`base32/pad`). |
| `base32/pad`           | RFC 4648 lower with padding.                         |
| `base32/nopad`         | RFC 4648 lower without padding.                      |
| `base32/hex-pad`       | RFC 4648 hex lower with padding.                     |
| `base32/hex-nopad`     | RFC 4648 hex lower without padding.                  |
| `base64`               | Convert bytes array to base64 string (`base64/pad`). |
| `base64/pad`           | Standard lower with padding.                         |
| `base64/nopad`         | Standard lower without padding.                      |
| `base64/urlsafe-pad`   | URL-safe with padding.                               |
| `base64/urlsafe-nopad` | URL-safe without padding.                            |
| `json`                 | Convert given value to a JSON string.                |
| `toml`                 | Convert given value to a TOML string.                |

### `str.to_bytes(value: any, [charset: string]) -> [number]`

Convert string (or some other values) to a bytes vector.

If charset is specified, then the given value's byte
representation will be interpreted as UTF-8 encoded string,
and this method will try to convert it into a given charset.

> Note: this method internally uses the same algorithm as
> many other methods in the standard which accept many types.
>
> E.g. when hashing a value it firstly will be converted into
> bytes using this method.

```lua
print(str.to_bytes("abc")) -- [97, 98, 99]
print(str.to_bytes(0.5)) -- [63, 224, 0, 0, 0, 0, 0, 0]
print(str.to_bytes({ 1, 2, 3 })) -- [1, 2, 3]

local a = str.from_bytes({ 208, 176, 208, 177, 208, 190, 208, 177, 208, 176 })
local b = str.to_bytes(a, "cp1251")

-- Cyrillic is encoded using 1 byte in cp1251:
-- [224, 225, 238, 225, 224]
print(b)
```

### `str.from_bytes(bytes: [number], [charset: string]) -> string`

Convert bytes slice into a lua string. If charset is specified, then
this method will try to decode bytes from this charset into UTF-8.

```lua
local a = str.from_bytes({ 224, 225, 238, 225, 224 }, "cp1251")
local b = str.to_bytes(a)

-- Cyrillic is encoded using 2 bytes in UTF-8:
-- [208, 176, 208, 177, 208, 190, 208, 177, 208, 176]
print(b)
```

### `str.encode(value: any, encoding: StringEncoding) -> string`

Encode given value to a string.

```lua
print(str.encode(123, "base16"))               -- 7b
print(str.encode("Hello, World!", "base64"))   -- "SGVsbG8sIFdvcmxkIQ=="
print(str.encode({ hello = "world" }, "json")) -- "{\"hello\":\"world\"}"
```

### `str.decode(value: string, encoding: StringEncoding) -> any`

Decode given string to a bytes slice.

```lua
print(str.decode("7b", "base16"))                                   -- [0, 0, 0, 123]
print(str.from_bytes(str.decode("SGVsbG8sIFdvcmxkIQ==", "base64"))) -- "Hello, World!"
print(str.decode("{\"hello\":\"world\"}", "json"))                  -- { hello = "world" }
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

Temp directory's content is shared between all the packages.
This can be used to create custom synchronization mechanisms.

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

### `path.accessible(path: string) -> boolean`

Check if given path is accessible for the current module.

```lua
print(path.accessible(path.module_dir())) -- true
print(path.accessible("/home"))           -- false
```

## Filesystem API

All the fielsystem operations are sandboxed by both [luau](https://luau.org) engine
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

### `fs.exists(path: string) -> boolean`

Check if given filesystem path exists and accessible.

```lua
if fs.exists("/tmp") then
    print("Temp folder exists and can be accessed")
else
    print("Temp folder doesn't exist or can't be accessed")
end
```

### `fs.metadata(path: string) -> Metadata`

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

### `fs.remove(path: string)`

Remove a file, folder or a symlink. Removing a folder will remove
all its content as well.

```lua
fs.remove("my_file.txt")
fs.remove("my_folder")
fs.remove("my_symlink")
```

### `fs.open(path: string, [options: Options]) -> number`

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

### `fs.create_file(path: string)`

Create an empty file.

```lua
-- these two lines will do the same
fs.create_file("file_1")
fs.write_file("file_2", {})
```

### `fs.read_file(path: string) -> [number]`

Read the whole content of a file in a given path.

> Note: do not try to read large files using this function.

```lua
local content = fs.read_file("my_file.txt")

print("Read " .. #content .. " bytes")
```

### `fs.write_file(path: string, content: [number])`

Overwrite existing file with given content, or create
a new one.

```lua
fs.write_file("my_file.txt", { 1, 2, 3 })
```

### `fs.remove_file(path: string)`

Remove file in a given path.

```lua
fs.remove_file("my_file.txt")
```

### `fs.create_dir(path: string)`

Create directory if it doesn't exist.

```lua
-- this will create all the parent directories too
fs.create_dir("a/b/c/d")
```

### `fs.read_dir(path: string) -> [Entry]`

Read the given directory, returning list of its entries.

```ts
type Entry = {
    name: string,
    path: string,
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

## Network API

Launcher provides set of functions to perform HTTP request
and download files.

| Function    | Description                         |
| ----------- | ----------------------------------- |
| `net.fetch` | Perform HTTP request.               |
| `net.open`  | Open HTTP request to read the body. |
| `net.read`  | Read the open HTTP request.         |
| `net.close` | Close the open HTTP request.        |

### `net.fetch(url: string, [options: Options]) -> Response`

```ts
type Options = {
    // Method of the request.
    method?: 'get' | 'post' | 'head' | 'put' | 'patch' | 'delete' | 'connect',

    // Headers of the request.
    headers?: [key: string]: string,

    // Body of the request.
    body?: [number]
};

type Response = {
    // Status code of the response.
    status: number,

    // True if request succeeded (status 200 - 299).
    is_ok: boolean,

    // Table of response headers.
    headers: [key: string]: string,

    // Body of the response.
    body: [number]
};
```

```lua
local response = net.fetch("https://example.com")

if response.is_ok then
    print(response.body)
end
```

### `net.open(url: string, [options: Options]) -> LazyResponse`

Open new HTTP request in background and return a handle
to lazily read the body, similar to the IO API.

```ts
type LazyResponse = {
    // Status code of the response.
    status: number,

    // True if request succeeded (status 200 - 299).
    is_ok: boolean,

    // Table of response headers.
    headers: [key: string]: string,

    // Request handle.
    handle: number
};
```

```lua
local head = net.open("https://example.com/large_file.zip")

if head.is_ok then
    -- ...
end

net.close(head.handle)
```

### `net.read(handle: number) -> [number] | nil`

Read chunk of response body, or return nil if there's nothing
else to read.

```lua
local head = net.open("https://example.com/large_file.zip")

if head.is_ok do
    local chunk = net.read(head.handle)

    while chunk do
        -- do something with a chunk of data.

        chunk = net.read(head.handle)
    end
end

net.close(head.handle)
```

### `net.close(handle: number)`

Close the open HTTP request.

```lua
local head = net.open("https://example.com/large_file.zip")

-- fetch head only and do not download the body.
print(head.headers["Content-Length"])

net.close(head.handle)
```

## Downloader API

Launcher provides its own network files downloader. You could use this one
instead of making your own variant using the network API.

| Function              | Description                       |
| --------------------- | --------------------------------- |
| `downloader.download` | Download file from the given URL. |

### `downloader.download(url: string, [options: Options]) -> boolean`

Start downloading a file from the given URL, returning the downloading
result. This is a blocking method.

```ts
type Options = {
    // Path to the downloaded file.
    output_file?: string,

    // If true, then downloader will continue downloading
    // if the output file already exists.
    // Enabled by default.
    continue_downloading?: boolean,

    // Downloading progress handler.
    progress?: (current: number, total: number, diff: number)
};
```

```lua
-- when no output path given - downloader will automatically
-- resolve the file name (large_file.zip) and download it
-- in the module's folder (used as a relative folder for all the operations).
local result = downloader.download("https://example.com/large_file.zip", {
    continue_downloading = false,

    progress = function(curr, total, diff)
        print("progress: " .. (curr / total * 100) .. "%")
    end
})

if result then
    -- do something
end
```

## Archives API

Most of resources in the internet are transfered in form of archives.
This module allows you to extract their content.

| Function          | Description                                  |
| ----------------- | -------------------------------------------- |
| `archive.open`    | Open an archive.                             |
| `archive.entries` | List all the archive entries.                |
| `archive.extract` | Extract all the entries of the open archive. |
| `archive.close`   | Close an open archive.                       |

### `archive.open(path: string, [format: ArchiveFormat]) -> number`

Try to open an archive. This method will fail if the path is not accessible
or it doesn't point to an archive (or the archive format is not supported).

If format is not specified, then it's automatically assumed from the extension.

```ts
type ArchiveFormat = 'tar' | 'zip' | '7z';
```

```lua
local handle = archive.open("large_archive.zip")
```

### `archive.entries(handle: number) -> [Entry]`

List entries of an open archive.

This is a blocking method.

```ts
type Entry = {
    // Relative path of the archive entry.
    path: string,

    // Size of the archive entry.
    // Depending on format this could either
    // mean compressed or uncompressed size.
    size: number
};
```

```lua
local handle = archive.open("archive", "tar")

for _, entry in ipairs(archive.entries(handle)) do
    print(entry.size .. "   " .. entry.path)
end

archive.close(handle)
```

### `archive.extract(handle: number, target: string, [progress: (current: number, total: number, diff: number) -> ()]) -> boolean`

Extract an open archive to the terget directory.
You can specify a callback which will be used to update the progress
of the archive extraction. Progress is measured in bytes.

Returns extraction status. If failed, `false` is returned.

This is a blocking method.

```lua
local handle = archive.open("small_archive.zip")

-- don't request progress updates for small archive
archive.extract(handle, "my_folder")
archive.close(handle)
```

```lua
local handle = archive.open("large_archive.zip")

-- display extraction progress for a large archive
archive.extract(handle, "my_folder", function(current, total, diff)
    print("progress: " .. (current / total * 100) .. "%")
end)

archive.close(handle)
```

### `archive.close(handle: number)`

Close an open archive.

```lua
local handle = archive.open("archive.zip")

-- do some operations

archive.close(handle)
```

## Hashes API

In many cases you would like to calculate hashes of values.
This API provides some most used ones for you.

| Function       | Description                               |
| -------------- | ----------------------------------------- |
| `hash.calc`    | Calculate hash for a bytes slice.         |
| `hash.builder` | Create new hash builder.                  |
| `hash.write`   | Write a chunk of data to the open hasher. |
| `hash.finalize`| Finalize hash value.                      |

### Supported hash algorithms

Following table contains list of `HashAlgorithm` enum values.

| Algorithm  | Bits | Cryptographic | URL                                  |
| ---------- | ---- | ------------- | ------------------------------------ |
| `seahash`  | 64   | No            | https://crates.io/crates/seahash     |
| `crc32`    | 32   | No            | https://crates.io/crates/crc32fast   |
| `crc32c`   | 32   | No            | https://crates.io/crates/crc32c      |
| `xxh32`    | 32   | No            | https://crates.io/crates/xxhash-rust |
| `xxh64`    | 64   | No            | https://crates.io/crates/xxhash-rust |
| `xxh3-64`  | 64   | No            | https://crates.io/crates/xxhash-rust |
| `xxh3-128` | 128  | No            | https://crates.io/crates/xxhash-rust |
| `md5`      | 128  | Yes           | https://crates.io/crates/md5         |
| `sha1`     | 160  | Yes           | https://crates.io/crates/sha1        |
| `sha2-224` | 224  | Yes           | https://crates.io/crates/sha2        |
| `sha2-256` | 256  | Yes           | https://crates.io/crates/sha2        |
| `sha2-384` | 384  | Yes           | https://crates.io/crates/sha2        |
| `sha2-512` | 256  | Yes           | https://crates.io/crates/sha2        |

### `hash.calc(value: any, [algorithm: HashAlgorithm]) -> [number]`

Calculate hash for a given bytes slice using specified algorithm.
By default `seahash` is used as a launcher's internal algorithm.

```lua
-- [236, 74, 195, 208]
print(hash.calc("Hello, World!", "crc32"))
```

### `hash.builder([algorithm: HashAlgorithm]) -> number`

Create new incremental data hasher. This should be used to hash
large amounts of data. Unlike `hash.calc` method where you had to
hold the whole data slice in RAM before making a hash, the hasher
struct allows you to write small chunks of data iteratively, not
keeping all of them in RAM at once.

```lua
local hasher = hash.hasher("md5")

-- do some actions
```

### `hash.write(handle: number, value: any)`

Write a chunk of data to the open hasher.

```lua
local hasher = hash.hasher("xxh3-128")
local head = net.open("https://example.com/large_file.zip")

if head.is_ok do
    local chunk = net.read(head.handle)

    while chunk do
        if #chunk > 0 do
            hasher.write(chunk)
        end

        chunk = net.read(head.handle)
    end

    -- print large file's hash
    print(hash.finalize(hasher))
end

hash.close(hasher)
net.close(head.handle)
```

### `hash.finalize(handle: number) -> [number]`

Finalize hash calculation in the open hasher struct.
This will close the hasher and prevent future writes.

```lua
local hasher = hash.hasher("sha1")

hash.write(hasher, "Hello")
hash.write(hasher, "World")

-- printed the same value
print(hash.finalize(hasher))
print(hash.calc("HelloWorld"))
```

## Sync API

Some packages would like to communicate with each other, e.g.
different version of the same package. Sync API provides a set
of data synchronization primitives for this.

### Channels

| Function             | Description                                |
| -------------------- | ------------------------------------------ |
| `sync.channel.open`  | Open inter-packages communication channel. |
| `sync.channel.send`  | Send a new message to the channel.         |
| `sync.channel.recv`  | Receive a new message from the channel.    |
| `sync.channel.close` | Close an open channel.                     |

#### `sync.channel.open(key: string) -> number`

Subscribe to a channel with a given key (name). After subscription
you receive a special identifier which will be used to hold your
read messages history. You can't read messages which were sent
before you obtained this identifier.

```lua
local channel = sync.channel.open("my_package_channel")
```

#### `sync.channel.send(handle: number, message: any)`

Send some value to the open channel. Message will be sent to all the
packages which have this channel open except you.

> NOTE: currently not *any* value is supported due to technical
> difficulties. Sent values are also not shared, meaning they all
> are cloned.

```lua
local channel = sync.channel.open("my_package_channel")

sync.channel.send(channel, "Hello, World!")
sync.channel.send(channel, { 1, 2, 3 })
sync.channel.send(channel, true)

-- messages sent by you will be visible to other
-- channel users, but you will not see them yourself.
print(sync.channel.recv(channel)) -- nil
```

#### `sync.channel.recv(handle: number) -> any | nil, bool`

Try to receive a message from the open channel. This is a non-blocking
method which will return `nil` if there's no messages to read. Second
returned value means status of the returned value. Since `nil` could
be sent in the channel as a message, second value indicates its status.
For every valid message it's `true` while for channel end message
it's `false`.

```lua
local sender   = sync.channel.open("my_package_channel")
local receiver = sync.channel.open("my_package_channel")

sync.channel.send(sender, 1)
sync.channel.send(sender, 2)
sync.channel.send(sender, nil)
sync.channel.send(sender, 3)

repeat
    local message, status = sync.channel.recv(receiver)

    -- 1, 2, nil, 3
    print(message)
until status

sync.channel.close(sender)
sync.channel.close(receiver)
```

#### `sync.channel.close(handle: number)`

Close the open channel. This will clear all the remaining messages and
prevent future writes to your identifier.

```lua
local channel = sync.channel.open("my_package_channel")

-- do some operations

sync.channel.close(channel)
```

### Mutex

Mutex is the most used synchronization primitive. It allows you to block
code execution while another thread (module, package) is using the mutex.

| Function            | Description                |
| ------------------- | -------------------------- |
| `sync.mutex.open`   | Open inter-packages mutex. |
| `sync.mutex.lock`   | Lock an open mutex.        |
| `sync.mutex.unlock` | Unlock an open mutex.      |
| `sync.mutex.close`  | Close an open mutex.       |

#### `sync.mutex.open(key: string) -> number`

Get handle to the mutex with given key identifier. This handle is used to
lock and unlock the same mutex from different packages and modules.

```lua
local mutex = sync.mutex.open("my_module_mutex")
```

#### `sync.mutex.lock(handle: number)`

Block code execution until the mutex is locked by you. Once another module
unlocks the mutex you (or some another module) will be able to lock it and
continue execution. After unlocking the mutex you allow other modules to
continue execution. This can be used if your module downloads resources
from the internet and you have different versions of the same module. Using
mutex you can block other modules from downloading the resources.

```lua
-- first module
local mutex = sync.mutex.open("my_module_mutex")

sync.mutex.lock(mutex)

-- do some operations

sync.mutex.close(mutex)
```

```lua
-- second module
local mutex = sync.mutex.open("my_module_mutex")

sync.mutex.lock(mutex)

-- do some operations

sync.mutex.unlock(mutex)
```

#### `sync.mutex.unlock(handle: number)`

Unlock the mutex, allowing other modules to lock it and continue execution.

```lua
local mutex = sync.mutex.open("my_module_mutex")

sync.mutex.lock(mutex)

-- do some operations

sync.mutex.unlock(mutex)
```

#### `sync.mutex.close(handle: number)`

Close the mutex handle. Closing locked mutex will automatically unlock it.

```lua
local mutex = sync.mutex.open("my_module_mutex")

sync.mutex.lock(mutex)

-- do some operations

sync.mutex.close(mutex)
```

## Process API (extended privileges)

Some games may need external software to be installed or
updated, e.g. their updates are encoded in some special
format and special binary should be used to apply these
updates. Process API allows signed packages to execute
binaries.

| Function           | Description                             |
| ------------------ | --------------------------------------- |
| `process.exec`     | Execute a binary, returning its output. |
| `process.open`     | Open a binary.                          |
| `process.stdin`    | Write some data to the process stdin.   |
| `process.stdout`   | Read a chunk of process stdout.         |
| `process.stderr`   | Read a chunk of process stderr.         |
| `process.wait`     | Wait until the process is closed.       |
| `process.kill`     | Kill an open binary process.            |
| `process.finished` | Check if open binary process is closed. |

### `process.exec(path: string, [args: [string]], [env: [key: string]: string]) -> Output`

Execute given binary and return its output. Module dir is used
as the binary's current directory.

```ts
type Output = {
    // Exit code of the process.
    status: number | null,

    // Was the process closed normally.
    is_ok: boolean,

    // Output of the process.
    stdout: [number],
    stderr: [number]
};
```

```lua
local my_file = path.join(path.module_dir(), "my_file.txt")

fs.write_file(my_file, str.to_bytes("Hello, World!"))

local output = process.exec("cat", { "my_file.txt" })

-- "Hello, World!"
print(string.from_bytes(output.stdout))
```

### `process.open(path: string, [args: [string]], [env: [key: string]: string]) -> number`

Start a new process with given parameters. Module dir is used
as the binary's current directory.

```lua
local handle = process.open("curl", { "api.ipify.org" })
```

### `process.stdin(handle: number, data: any)`

Write a bytes slice to the process's stdin.

```lua
local handle = process.open("my_app")

process.stdin(handle, "some input")
```

### `process.stdout(handle: number) -> [number] | nil`

Read the process's stdout chunk. If process is closed,
then `nil` is returned.

```lua
local handle = process.open("cat", { "large_file.txt" })

while not process.finished(handle) do
    local output = process.stdout(handle)

    if output then
        print(output)
    end
end

process.wait(handle)
```

### `process.stderr(handle: number) -> [number] | nil`

Read the process's stderr chunk. If process is closed,
then `nil` is returned.

```lua
local handle = process.open("my_app")

while not process.finished(handle) do
    local err = process.stderr(handle)

    if err then
        print("stderr: " .. err)
    end
end

process.wait(handle)
```

### `process.wait(handle: number) -> Output`

Wait until the process is closed. Output struct will contain
all the output and error bytes that weren't read using the
`process.stdout()` and `process.stderr()` methods.

This is a blocking method. This will remove the process handle.

```lua
-- equal to print(process.exec("my_app").stdout)
local handle = process.open("my_app")
local output = process.wait()

print(output.stdout)
```

### `process.kill(handle: number)`

Kill an open process. This will remove the process handle.

```lua
local handle = process.open("my_app")

-- immediately kill the process
process.kill(handle)
```

### `process.finished(handle: number) -> boolean`

Check if running process has finished.

```lua
local handle = process.open("my_app")

while not process.finished(handle) do
    -- do some actions
end

-- process is already finished so we call this
-- method to remove the process handle
process.kill(handle)
```
