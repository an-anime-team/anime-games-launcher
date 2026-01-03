# HTTP API

Standard set of methods to perform HTTP(S) requests.

| Function     | Description                         |
| ------------ | ----------------------------------- |
| `http.fetch` | Perform HTTP request.               |
| `http.open`  | Open HTTP request to read the body. |
| `http.read`  | Read the open HTTP request.         |
| `http.close` | Close the open HTTP request.        |

## `http.fetch(url: string, [options: Options]) -> Promise<Response>`

```ts
type Options = {
    // Method of the request. GET is used by default.
    method?: 'get' | 'post' | 'head' | 'put' | 'patch' | 'delete' | 'connect';

    // Headers of the request.
    headers?: [key: string]: string;

    // Body of the request.
    body?: Bytes;
};

type Response = {
    // Status code of the response.
    status: number;

    // True if request succeeded (status 200 - 299).
    is_ok: boolean;

    // Table of response headers.
    headers: [key: string]: string;

    // Body of the response.
    body: Bytes;
};
```

```luau
local response = http.fetch("https://example.com")

if response.is_ok then
    print(response.body:as_string())
end
```

## `http.open(url: string, [options: Options]) -> LazyResponse`

Open new HTTP request in background and return a handle to lazily read the body,
similar to the IO API.

```ts
type LazyResponse = {
    // Status code of the response.
    status: number;

    // True if request succeeded (status 200 - 299).
    is_ok: boolean;

    // Table of response headers.
    headers: [key: string]: string;

    // Request handle.
    handle: number;
};
```

```luau
local head = http.open("https://example.com/large_file.zip")

if head.is_ok then
    -- ...
end

http.close(head.handle)
```

## `http.read(handle: number) -> Bytes | nil`

Read chunk of response body, or return `nil` if there's nothing else to read.

```luau
local head = http.open("https://example.com/large_file.zip")

if head.is_ok do
    local chunk = http.read(head.handle)

    while chunk do
        -- do something with a chunk of data.

        chunk = http.read(head.handle)
    end
end

http.close(head.handle)
```

## `http.close(handle: number)`

Close the open HTTP client.

```luau
local head = http.open("https://example.com/large_file.zip")

-- fetch head only and do not download the body.
print(head.headers["Content-Length"])

http.close(head.handle)
```
