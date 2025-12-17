# Paths API

Filesystem paths are sandboxed by design. Each module can access special
sandboxed folders to store its state there. This module provides functions to
obtain these paths, as well as some utility functions to work with them.

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

## `path.temp_dir() -> string`

Temp directory is configured by the user in the launcher app and its content
will eventually be automatically cleared. You can use temp directory to store
temporary data, e.g. downloaded archives.

Temp directory's content is shared between all the packages. This can be used
to create custom synchronization mechanisms.

```luau
local temp = path.temp_dir()

if fs.exists(temp .. "/.first-run") do
    -- ...
end
```

## `path.module_dir() -> string`

Each module has its own directory which cannot be accessed by any other modules.
It should be used to store all its private data. If module is updated to a newer
version (its hash was changed) - it will have a new folder and it won't be able
to access the previous one.

Module directory can be deleted by the packages garbage collector when the
module is not used.

```luau
local store = path.module_dir()

fs.write_file(store .. "/secret_file", { 1, 2, 3 })
```

## `path.persist_dir(key: string) -> string`

Modules can get paths to the persistent data storage using special keyword.
Every module using the same keyword will get the same path. This can be used to
transfer state files from one module to another.

```luau
-- first module
local test_dir = path.persist_dir("test")

fs.create_file(test_dir .. "/example")
```

```luau
-- second module
local test_dir = path.persist_dir("test")

print(fs.exists(test_dir .. "/example")) -- true
```

## `path.normalize(path: string) -> string | nil`

Path normalization will remove all the special path components.
If path is meaningless, then nil is returned.

```luau
print(path.normalize("./test"))   -- "test"
print(path.normalize("a/b/../c")) -- "a/c"
print(path.normalize("a/b/./c"))  -- "a/b/c"
print(path.normalize("a\\b\\c"))  -- "a/b/c"

-- We don't support relative paths:
print(path.normalize("."))  -- nil
print(path.normalize("..")) -- nil
```

## `path.join(parts: ...string) -> string | nil`

Create new path by combining given entries names. This function will normalize
the result path as well. If no parts were given or they're meaningless - nil
is returned.

```luau
local dir = path.join(path.module_dir(), "download")

print(path.join())     -- nil
print(path.join("."))  -- nil
print(path.join("..")) -- nil
```

## `path.parts(path: string) -> [string] | nil`

Split given filesystem entry path to the components (entries names).
This function will normalize the path before splitting it. If input string is
empty or meaningless - nil is returned.

```luau
-- ["a", "c"]
print(path.parts("a/b\\../c/./"))

print(path.parts(""))   -- nil
print(path.parts("."))  -- nil
print(path.parts("..")) -- nil
```

## `path.parent(path: string) -> string | nil`

Return parent folder path or nil if it doesn't exist. Return path will be
normalized.

```luau
print(path.parent("a/./b")) -- "a"
print(path.parent("a"))     -- nil
```

## `path.file_name(path: string) -> string | nil`

Return the last entry name of the given path. Return nil if the input string
is meaningless.

```luau
print(path.file_name("a/b/c"))          -- "c"
print(path.file_name("a/./b/../../c/")) -- "c"

print(path.file_name("."))  -- nil
print(path.file_name("..")) -- nil
```

## `path.exists(path: string) -> bool`

Check if given path exists on the disk. This function, unlike `fs.exists`,
doesn't check if the given path is accessible for the current module, so you
can use it to verify if some system libraries or binaries are presented on the
user's system.

```luau
print(path.exists(path.module_dir())) -- true
print(path.exists("/home"))           -- true
```

## `path.accessible(path: string) -> boolean`

Check if given path is accessible for the current module.

```luau
print(path.accessible(path.module_dir())) -- true
print(path.accessible("/home"))           -- false
```
