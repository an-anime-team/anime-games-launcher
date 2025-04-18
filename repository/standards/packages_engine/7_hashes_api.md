# Hashes API

In many cases you would like to calculate hashes of values. This API provides
some most used ones for you.

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

## `hash.calc(value: any, [algorithm: HashAlgorithm]) -> [number]`

Calculate hash for a given bytes slice using specified algorithm.
By default `seahash` is used as a launcher's internal algorithm.

```luau
-- [236, 74, 195, 208]
dbg(hash.calc("Hello, World!", "crc32"))
```

## `hash.builder([algorithm: HashAlgorithm]) -> number`

Create new incremental data hasher. This should be used to hash large amounts of
data. Unlike `hash.calc` method where you had to hold the whole data slice in
RAM before making a hash, the hasher struct allows you to write small chunks of
data iteratively, not keeping all of them in RAM at once.

```luau
local hasher = hash.builder("md5")

-- do some actions
```

## `hash.write(handle: number, value: any)`

Write a chunk of data to the open hasher.

```luau
local hasher = hash.builder("xxh3-128")
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

## `hash.finalize(handle: number) -> [number]`

Finalize hash calculation in the open hasher struct. This will close the hasher
and prevent future writes.

```luau
local hasher = hash.builder("sha1")

hash.write(hasher, "Hello")
hash.write(hasher, "World")

-- printed the same value
print(str.encode(hash.finalize(hasher), "hex"))
print(str.encode(hash.calc("HelloWorld"), "hex"))
```
