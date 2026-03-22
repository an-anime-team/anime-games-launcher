# Downloader API

Launcher provides its own network files downloader. You can use this one
instead of making your variant using the network API.

| Function              | Description                                 |
| --------------------- | ------------------------------------------- |
| `downloader.create`   | Create HTTP downloader client.              |
| `downloader.download` | Start file downloading from the given URL.  |
| `downloader.progress` | Get file downloading task progress.         |
| `downloader.wait`     | Block execution until the task is finished. |
| `downloader.abort`    | Stop file downloading.                      |
| `downloader.close`    | Close HTTP downloader client.               |

## `downloader.create() -> number`

Create new downloader. Each downloader has its own cookies store, DNS queries
cache and other information. It's recommended to initialize one downloader per
module to download multiple files.

```luau
local handle = downloader.create()

-- Do something

downloader.close(handle)
```

## `downloader.download(handle: number, options: Options) -> number`

Start file downloading from the given HTTP(S) URL using provided options.
This function will create a background task on the multi-thread async runtime
and return a handle which can be used to query downloading status.
You can spawn multiple download tasks at once to perform parallel file downloads.

```ts
type Options = {
    // HTTP(S) URL to the file which is needed to be downloaded.
    url: string,

    // Path to the output file. Will be created if doesn't exist.
    // Relative paths are resolved in the module directory.
    output_file: string,

    // If enabled and downloader finds the given output file - it will continue
    // downloading appending new bytes to that file instead of overwriting it.
    //
    // Currently enabled by default.
    continue_download?: boolean,

    // Callback executed every time downloader reads a chunk of data.
    on_update?: (current: number, total: number): void,

    // Callback executed when downloading is successfully finished.
    on_finish?: (total: number): void
};
```

```luau
local downloader_handle = downloader.create()

-- Relative paths are resolved into the module's folder.
local task_handle = downloader.download(downloader_handle, {
    url = "https://example.com/archive.zip",
    output_file = "archive.zip",

    -- Print downloaded file size when it's finished.
    on_finish = function(total)
        print(`Downloaded {total / 1024} KiB`)
    end
})

-- Process the task.

-- Even if the task is finished already you can use this function
-- to forcely clean RAM which it occupied. Currently launcher doesn't
-- do it automatically.
downloader.wait(task_handle)
downloader.close(downloader_handle)
```

## `downloader.progress(handle: number) -> Progress`

Query progress of a file downloading task.

```ts
type Progress = {
    // Amount of already downloaded bytes.
    current: number,

    // Total expected amount of bytes to be downloaded.
    total: number,

    // Downloading progress of `current / total`.
    //
    // If `current` is 0, then the fraction is always 0.0.
    // If `total` is 0, then the fraction is always 1.0.
    fraction: number,

    // State of the downloading task.
    finished: boolean
};
```

```luau
local downloader_handle = downloader.create()

local task_handle = downloader.download(downloader_handle, {
    url = "https://example.com/archive.zip",
    output_file = "archive.zip"
})

-- Manually await file downloading, printing progress to the console.
local progress = downloader.progress(task_handle)

while not progress.finished do
    print(`Progress: {progress.fraction * 100}%`)

    progress = downloader.progress(task_handle)
end

-- Even if the task is finished already you can use this function
-- to forcely clean RAM which it occupied. Currently launcher doesn't
-- do it automatically.
downloader.wait(task_handle)
downloader.close(downloader_handle)
```

## `downloader.wait(handle: number) -> number`

Block current thread until the file downloading task is finished, returning
total amount of bytes in the output file after it's done. This function will
consume the task's handle and prevent its future use.

```luau
local downloader_handle = downloader.create()

local task_handle = downloader.download(downloader_handle, {
    url = "https://example.com/archive.zip",
    output_file = "archive.zip"
})

local downloaded = downloader.wait(task_handle)

print(`Downloaded {downloaded / 1024} KiB`)

downloader.close(downloader_handle)
```

## `downloader.abort(handle: number)`

Abort file downloading task. This function will consume the task's handle so you
won't be able to use it afterwards, pretty much like with `downloader.wait`.

```luau
local downloader_handle = downloader.create()

local task_handle = downloader.download(downloader_handle, {
    url = "https://example.com/archive.zip",
    output_file = "archive.zip"
})

-- Immediately stop file downloading.
downloader.abort(task_handle)

downloader.close(downloader_handle)
```

## `downloader.close(handle: number)`

Close downloader, preventing its future uses.

```luau
local handle = downloader.create()

-- Do something

downloader.close(handle)
```
