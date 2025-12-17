# Archives API

Most of resources in the internet are transfered in form of archives.
This module allows you to extract their content.

| Function          | Description                                  |
| ----------------- | -------------------------------------------- |
| `archive.open`    | Open an archive.                             |
| `archive.entries` | List all the archive entries.                |
| `archive.extract` | Extract all the entries of the open archive. |
| `archive.close`   | Close an open archive.                       |

## `archive.open(path: string, [format: ArchiveFormat]) -> number`

Try to open an archive. This method will fail if the path is not accessible
or it doesn't point to an archive (or the archive format is not supported).

If format is not specified, then it's automatically assumed from the extension.

```ts
type ArchiveFormat = 'tar' | 'zip' | '7z';
```

```luau
local handle = archive.open("large_archive.zip")
```

## `archive.entries(handle: number) -> [Entry]`

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

```luau
local handle = archive.open("archive", "tar")

for _, entry in ipairs(archive.entries(handle)) do
    print(entry.size .. "   " .. entry.path)
end

archive.close(handle)
```

## `archive.extract(handle: number, target: string, [progress: (current: number, total: number, diff: number) -> ()]) -> boolean`

Extract an open archive to the terget directory. You can specify a callback
which will be used to update the progress of the archive extraction. Progress is
measured in bytes.

Returns extraction status. If failed, `false` is returned.

This is a blocking method.

```luau
local handle = archive.open("small_archive.zip")

-- don't request progress updates for small archive
archive.extract(handle, "my_folder")
archive.close(handle)
```

```luau
local handle = archive.open("large_archive.zip")

-- display extraction progress for a large archive
archive.extract(handle, "my_folder", function(current, total, diff)
    print("progress: " .. (current / total * 100) .. "%")
end)

archive.close(handle)
```

## `archive.close(handle: number)`

Close an open archive.

```luau
local handle = archive.open("archive.zip")

-- do some operations

archive.close(handle)
```
