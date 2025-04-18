# Network API

Launcher provides set of functions to perform HTTP request and download files.

| Function    | Description                         |
| ----------- | ----------------------------------- |
| `net.fetch` | Perform HTTP request.               |
| `net.open`  | Open HTTP request to read the body. |
| `net.read`  | Read the open HTTP request.         |
| `net.close` | Close the open HTTP request.        |

## `net.fetch(url: string, [options: Options]) -> Response`

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

```luau
local response = net.fetch("https://example.com")

if response.is_ok then
    print(response.body)
end
```

## `net.open(url: string, [options: Options]) -> LazyResponse`

Open new HTTP request in background and return a handle to lazily read the body,
similar to the IO API.

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

```luau
local head = net.open("https://example.com/large_file.zip")

if head.is_ok then
    -- ...
end

net.close(head.handle)
```

## `net.read(handle: number) -> [number] | nil`

Read chunk of response body, or return nil if there's nothing else to read.

```luau
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

## `net.close(handle: number)`

Close the open HTTP request.

```luau
local head = net.open("https://example.com/large_file.zip")

-- fetch head only and do not download the body.
print(head.headers["Content-Length"])

net.close(head.handle)
```
