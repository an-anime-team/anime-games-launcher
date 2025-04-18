# Process API (extended privileges)

Some games may need external software to be installed or updated, e.g. their
updates are encoded in some special format and special binary should be used
to apply these updates. Process API allows trusted packages to execute binaries.

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

## `process.exec(path: string, [args: [string]], [env: [key: string]: string]) -> Output`

Execute given binary and return its output. Module dir is used as the binary's
current directory.

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

```luau
local my_file = path.join(path.module_dir(), "my_file.txt")

fs.write_file(my_file, str.to_bytes("Hello, World!"))

local output = process.exec("cat", { "my_file.txt" })

-- "Hello, World!"
print(str.from_bytes(output.stdout))
```

## `process.open(path: string, [args: [string]], [env: [key: string]: string]) -> number`

Start a new process with given parameters. Module dir is used as the binary's
current directory.

```luau
local handle = process.open("curl", { "api.ipify.org" })
```

## `process.stdin(handle: number, data: any)`

Write a bytes slice to the process's stdin.

```luau
local handle = process.open("my_app")

process.stdin(handle, "some input")
```

## `process.stdout(handle: number) -> [number] | nil`

Read the process's stdout chunk. If process is closed, then `nil` is returned.

```luau
local handle = process.open("cat", { "large_file.txt" })

while not process.finished(handle) do
    local output = process.stdout(handle)

    if output then
        print(output)
    end
end

process.wait(handle)
```

## `process.stderr(handle: number) -> [number] | nil`

Read the process's stderr chunk. If process is closed, then `nil` is returned.

```luau
local handle = process.open("my_app")

while not process.finished(handle) do
    local err = process.stderr(handle)

    if err then
        print("stderr: " .. err)
    end
end

process.wait(handle)
```

## `process.wait(handle: number) -> Output`

Wait until the process is closed. Output struct will contain all the output and
error bytes that weren't read using the `process.stdout()` and
`process.stderr()` methods.

This is a blocking method. This will remove the process handle.

```luau
-- equal to print(process.exec("my_app").stdout)
local handle = process.open("my_app")
local output = process.wait()

print(output.stdout)
```

## `process.kill(handle: number)`

Kill an open process. This will remove the process handle.

```luau
local handle = process.open("my_app")

-- immediately kill the process
process.kill(handle)
```

## `process.finished(handle: number) -> boolean`

Check if running process has finished.

```luau
local handle = process.open("my_app")

while not process.finished(handle) do
    -- do some actions
end

-- process is already finished so we call this
-- method to remove the process handle
process.kill(handle)
```
