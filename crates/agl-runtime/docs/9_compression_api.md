# Compression API

Sometimes you would want to manually compress or decompress data, e.g. fetched
from the internet or packed into some proprietary file encoding format. This API
implements some of the most common compression algorithms.

| Function                   | Description                                              |
| -------------------------- | -------------------------------------------------------- |
| `compression.compress`     | Compress given bytes slice.                              |
| `compression.decompress`   | Decompress given bytes slice.                            |
| `compression.compressor`   | Open data compressor.                                    |
| `compression.decompressor` | Open data decompressor.                                  |
| `compression.write`        | Write data to the compressor / decompressor.             |
| `compression.flush`        | Flush written data and process it, returning the result. |
| `compression.close`        | Close open compressor / decompressor.                    |

## Supported compression algorithms

Following table contains list of `CompressionAlgorithm` enum values. Supported 
compression algorithms are provided by the `agl-core` library.

| Algorithm  | URL                                |
| ---------- | ---------------------------------- |
| `lz4`      | https://crates.io/crates/lz4_flex  |
| `bzip`     | https://crates.io/crates/bzip2     |
| `deflate`  | https://crates.io/crates/flate2    |
| `gzip`     | https://crates.io/crates/flate2    |
| `zlib`     | https://crates.io/crates/flate2    |
| `zstd`     | https://crates.io/crates/zstd      |
| `lzma`     | https://crates.io/crates/lzma-rust |
| `lzma2`    | https://crates.io/crates/lzma-rust |

## Compression levels

Different compression algorithms have different compression level ranges. The
following table defines 5 values which can be used instead of a numeric value,
which will be automatically converted to the appropriate compression level for
each algorithm. These values can change between application versions and are
presented for simplicity only.

| Algorithm   | Min | Max | `quick` | `fast` | `balanced` | `good` | `best` | `default` |
| ----------- | --- | --- | ------- | ------ | ---------- | ------ | ------ | --------- |
| `lz4` (1)   | -   | -   | -       | -      | -          | -      | -      | -         |
| `bzip`      | 1   | 9   | 1       | 3      | 5          | 7      | 9      | 4         |
| `deflate`   | 1   | 9   | 1       | 3      | 5          | 7      | 9      | 6         |
| `gzip`      | 1   | 9   | 1       | 3      | 5          | 7      | 9      | 6         |
| `zlib`      | 1   | 9   | 1       | 3      | 5          | 7      | 9      | 6         |
| `zstd`      | 1   | 22  | 3       | 9      | 13         | 17     | 22     | 10        |
| `lzma`  (2) | 0   | 9   | 1       | 3      | 5          | 7      | 9      | 4         |
| `lzma2` (2) | 0   | 9   | 1       | 3      | 5          | 7      | 9      | 4         |

> 1. lz4 doesn't have compression levels.
> 2. lzma doesn't have compression levels, but it has compression options.
>    Due to this you cannot directly rely on the standard lzma object provided
>    by this standard to process external data. You also have to specify the
>    compression level in lzma decompressor builder, unlike other algorithms.

## `compression.compress(algorithm: string, value: any) -> Promise<number[]>`

Compress given value using provided algorithm. The algorithm string must be a
`CompressionAlgorithm` value with optional compression level specified after the
column. If value is not a bytes slice - then it will be converted to bytes
representation first. Since this function can take some time to finish the
returned value is a background promise.

```luau
-- zstd with default compression level, equal to zstd:default
dbg(compression.compress("zstd", "Hello, World!"):await())

-- zstd with compression level 7
dbg(compression.compress("zstd:7", "Hello, World!"):await())
```

## `compression.decompress(algorithm: string, value: number[]) -> Promise<number[]>`

Decompress given bytes slice using the specified algorithm. Since this function
can take some time to finish the returned value is a background promise.

```luau
local compressed = compression.compress("zstd", "Hello, World!"):await()
local decompressed = compression.decompress("zstd", compressed):await()

-- "Hello, World!"
dbg(str.from_bytes(decompressed))
```

## `compression.compressor(algorithm: string) -> number`

Create data compression object, returning handle to it. Just like in the
`compress` method the algorithm is a `CompressionAlgorithm` value with optional
compression level specified after the column.

```luau
local lzma2_default = compression.compressor("lzma2")
local zstd_level_9 = compression.compressor("zstd:9")
```

## `compression.decompressor(algorithm: string) -> number`

Create data decompression object, returning handle to it.

```luau
local lzma2 = compression.decompressor("lzma2")
```

## `compression.write(handle: number, value: [number]) -> number`

Write bytes to the compressor / decompressor object, returning amount of bytes
of processed data added to the flush buffer. You can use this value to read
chunks of processed data of some size, e.g. don't flush the data if it's under
1024 bytes, which is quite helpful for advanced data processing logic.

```luau
local compressor = compression.compressor("lz4")

compression.write(compressor, str.to_bytes("Hello, "))
compression.write(compressor, str.to_bytes("World!"))
```

## `compression.flush(handle: number) -> [number]`

Flush the written data, returning the processing result.

```luau
local compressor = compression.compressor("zstd:best")

compression.write(compressor, str.to_bytes("Hello, "))
compression.write(compressor, str.to_bytes("World!"))

dbg(compression.flush(compressor))

compression.close(compressor)
```

## `compression.close(handle: number)`

Close compressor / decompressor object and prevent its future use.

```luau
local compressor = compression.compressor("zstd")

-- process the data

compression.close(compressor)
```
