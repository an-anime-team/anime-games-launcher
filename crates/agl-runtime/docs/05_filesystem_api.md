# Filesystem API

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
| `fs.seek_rel`    | Set relative pointer in a file handle.   |
| `fs.truncate`    | Truncate file to specified length.       |
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

All the relative paths are resolved in the module folder.

## `fs.exists(path: string) -> boolean`

Check if given filesystem path exists and can be read.

```luau
if fs.exists("/tmp") then
    print("Temp folder exists and can be read")
else
    print("Temp folder doesn't exist or can't be read")
end
```

## `fs.metadata(path: string) -> Metadata`

Read metadata of the filesystem path (file, directory or a symlink).

```ts
type EntryType = 'file' | 'directory' | 'symlink';

type Metadata = {
    // UTC timestamp of the creation time.
    created_at: number;

    // UTC timestamp of the modification time.
    modified_at: number;

    // Length in bytes of the filesystem entry. For files it's equal to the 
    // file's size. Currently symlink and directory lengths are undefined.
    length: number;

    // Filesystem entry permissions.
    permissions: {
        // Whether the path can be read.
        read: boolean;

        // Whether the path can be written to.
        write: boolean;
    };

    // Type of the filesystem entry.
    type: EntryType;
};
```

```luau
local metadata = fs.metadata("my_file.txt")

print("Size: " .. metadata.length)
print("Type: " .. metadata.type)
```

## `fs.copy(source: string, target: string)`

Copy file or folder to another location. This function will throw an error if
the target location already exists or is not accessible.

```luau
fs.copy("my_folder", "new_location/my_folder")
```

## `fs.move(source: string, target: string)`

Move a file or a folder to another location. This function will throw an error
if the target location already exists or is not accessible.

```luau
fs.move("my_folder", "new_location/my_folder")
```

## `fs.remove(path: string)`

Remove a file, folder or a symlink. Removing a folder will remove all its
content as well.

```luau
fs.remove("my_file.txt")
fs.remove("my_folder")
fs.remove("my_symlink")
```

## `fs.open(path: string, [options: Options]) -> number`

Open a file handle.

Handle is a randomly generated number associated with the file reader.
Modules have limited amount of simultaniously open handles.

```ts
type Options = {
    // Allow reading content from file.
    // Default: true.
    read: boolean;

    // Allow writing content to file.
    // Default: false.
    write: boolean;

    // Create file if it doesn't exist.
    // Default: false.
    create: boolean;

    // Clear file's content or create an empty one.
    // Default: false.
    overwrite: boolean;

    // Append writes to the end of the existing file's content.
    // Default: false.
    append: boolean;
};
```

```luau
-- Create a new file or clear already existing.
local handle = fs.open("my_file.txt", {
    create    = true,
    overwrite = true,
    write     = true
})
```

## `fs.seek(handle: number, position: number)`

Seek position in the given file handle.

Position can be negative to set offset from the end of the file.
Otherwise it's always set from the beginning of the file.

```luau
local handle = fs.open("my_file.txt")

fs.seek(10)

-- read chunk of data skipping first 10 bytes
local head = fs.read()

fs.seek(-10)

-- read last chunk of data with 10 bytes offset from the end
local tail = fs.read()

fs.close(handle)
```

## `fs.seek_rel(handle: number, offset: number)`

Seek position relative to the current: `new_pos = curr_pos + offset`.
For negative numbers you seek backwards, positive - forward the current position.

```luau
local handle = fs.open("my_file.txt")

fs.seek(3)      -- seek to position 3
fs.seek_rel(-2) -- seek 2 bytes before the current position of 3 (position 1)

fs.write(handle, { 123 }) -- write byte 123 to this position

local byte = fs.read(handle, 1, 1)[1] -- read byte from position 1

print(byte) -- verify that it's equal to 123

fs.close(handle)
```

## `fs.truncate(handle: number, length: number)`

If specified length is greater than the length of the file, then it will be
extended with zeros. Otherwise excess bytes will be deleted from the end of the
file.

```luau
local handle = fs.open("my_file.txt")

-- Write 11 bytes to the file
fs.write(handle, str.to_bytes("Hello World"))

-- Truncate file to contain only first 5 bytes
fs.truncate(handle, 5)

-- Read file's content from its beginning
local content = str.from_bytes(fs.read(handle, 0))

print(content) -- "Hello"
```

## `fs.read(handle: number, [position: number, [length: number]]) -> Bytes | nil`

Read chunk of binary data from the open file handle. Size of chunk is determined
by the rust API. If `nil` is returned then there's no more data to read.

If `position` is specified, then `fs.seek` will be used before reading the
chunk. This will affect future operations as well. Position can be negative to
set offset from the end of the file. Otherwise it's always set from the
beginning of the file.

If `length` is specified, then the exact amount of bytes will attempted to be
read. If file doesn't have enough bytes - an error will be returned.

```luau
local handle = fs.open("large_file.txt")
local chunk  = fs.read(handle)

while chunk do
    -- do something with chunk of data

    chunk = fs.read(handle)
end

fs.close(handle)
```

```luau
local handle = fs.open("game_file")

-- read game version from the file (3 bytes)
local game_version = fs.read(handle, 1000, 3)

fs.close(handle)
```

## `fs.write(handle: number, content: Bytes, [position: number])`

Write given data to the open file at its current position.

If `position` is specified, then `fs.seek` will be used before reading the
chunk. This will affect future operations as well. Position can be negative to
set offset from the end of the file. Otherwise it's always set from the
beginning of the file.

```luau
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

```luau
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

## `fs.flush(handle: number)`

Flush file content on disk.

Write operations are performed on a small buffer in the RAM and are flushed on
disk only when the buffer is full. This greatly improves performance of
operations, but changes will not be available for other file readers until the
buffer is flushed. This function forcely flushes the buffer on disk.

```luau
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

## `fs.close(handle: number)`

Close the file handle. This will flush the inner buffer of the file and prevent
future use of this handle.

```luau
local handle = fs.write("my_file.txt", { write = true })

fs.write({ 1, 2, 3 })
fs.close(handle)
```

## `fs.create_file(path: string)`

Create an empty file.

```luau
-- these two lines will do the same
fs.create_file("file_1")
fs.write_file("file_2", {})
```

## `fs.read_file(path: string) -> Bytes`

Read the whole content of a file in a given path. It's not recommended to read
whole content of large files at once.

```luau
local content = fs.read_file("my_file.txt")

print(`Read {#content} bytes`)
```

## `fs.write_file(path: string, content: Bytes)`

Overwrite existing file with given content, or create a new one.

```luau
fs.write_file("my_file.txt", { 1, 2, 3 }) -- bytes 1, 2 and 3
fs.write_file("my_file.txt", "123") -- ASCII characters for 1, 2 and 3
```

## `fs.remove_file(path: string)`

Remove file in a given path.

```luau
fs.remove_file("my_file.txt")
```

## `fs.create_dir(path: string)`

Create directory if it doesn't exist.

```luau
-- this will create all the parent directories too
fs.create_dir("a/b/c/d")
```

## `fs.read_dir(path: string) -> Entry[]`

Read the given directory, returning list of its entries.

```ts
type Entry = {
    name: string;
    path: string;
    type: EntryType;
};
```

```luau
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

## `fs.remove_dir(path: string)`

Remove given folder and all its content.

```luau
fs.create_dir("my_dir")

print(fs.exists("my_dir")) -- true

fs.remove_dir("my_dir")

print(fs.exists("my_dir")) -- false
```
