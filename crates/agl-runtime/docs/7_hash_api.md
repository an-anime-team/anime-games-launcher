# Hash API

In many cases you would like to calculate hashes of values. This API provides
some most used ones for you.

| Function             | Description                               |
| -------------------- | ----------------------------------------- |
| `hash.digitize`      | Calculate hash for a given bytes slice.   |
| `hash.digitize_file` | Calculate hash of a given file.           |
| `hash.builder`       | Create new hash builder.                  |
| `hash.write`         | Write a chunk of data to the open hasher. |
| `hash.finalize`      | Finalize hash value.                      |

### Supported hash algorithms

Following table contains list of `HashAlgorithm` enum values. Supported hashing
algorithms are provided by the `agl-core` library.

| Algorithm         | Bits | Cryptographic | URL                                  |
| ----------------- | ---- | ------------- | ------------------------------------ |
| `seahash`         | 64   | No            | https://crates.io/crates/seahash     |
| `crc32`           | 32   | No            | https://crates.io/crates/crc32fast   |
| `crc32c`          | 32   | No            | https://crates.io/crates/crc32c      |
| `siphash-1-3-64`  | 64   | No            | https://crates.io/crates/siphasher   |
| `siphash-1-3-128` | 128  | No            | https://crates.io/crates/siphasher   |
| `siphash-2-4-64`  | 64   | No            | https://crates.io/crates/siphasher   |
| `siphash-2-4-128` | 128  | No            | https://crates.io/crates/siphasher   |
| `xxh32`           | 32   | No            | https://crates.io/crates/xxhash-rust |
| `xxh64`           | 64   | No            | https://crates.io/crates/xxhash-rust |
| `xxh3-64`         | 64   | No            | https://crates.io/crates/xxhash-rust |
| `xxh3-128`        | 128  | No            | https://crates.io/crates/xxhash-rust |
| `blake2s`         | 256  | No            | https://crates.io/crates/blake2      |
| `blake2b`         | 512  | No            | https://crates.io/crates/blake2      |
| `blake3`          | 256  | No            | https://crates.io/crates/blake3      |
| `md5`             | 128  | Yes           | https://crates.io/crates/md5         |
| `sha1`            | 160  | Yes           | https://crates.io/crates/sha1        |
| `sha2-224`        | 224  | Yes           | https://crates.io/crates/sha2        |
| `sha2-256`        | 256  | Yes           | https://crates.io/crates/sha2        |
| `sha2-384`        | 384  | Yes           | https://crates.io/crates/sha2        |
| `sha2-512`        | 256  | Yes           | https://crates.io/crates/sha2        |
| `sha3-224`        | 224  | Yes           | https://crates.io/crates/sha3        |
| `sha3-256`        | 256  | Yes           | https://crates.io/crates/sha3        |
| `sha3-384`        | 384  | Yes           | https://crates.io/crates/sha3        |
| `sha3-512`        | 256  | Yes           | https://crates.io/crates/sha3        |

## `hash.digitize(algorithm: HashAlgorithm, value: any) -> number[]`

Calculate hash for a given bytes slice using specified algorithm. By default
`seahash` is used as a launcher's internal algorithm.

```luau
-- [236, 74, 195, 208]
dbg(hash.digitize("crc32", "Hello, World!"))
```

## `hash.digitize_file(algorithm: HashAlgorithm, path: string) -> number[]`

Calculate hash for a given file path using specified algorithm. By default
`seahash` is used as a launcher's internal algorithm. Only accessible files can
be hashed.

```luau
fs.write_file("test.txt", "Hello, World!")

-- [236, 74, 195, 208]
dbg(hash.digitize_file("crc32", "test.txt"))
```

## `hash.hasher(algorithm: HashAlgorithm) -> number`

Create new incremental data hasher. This should be used to hash large amounts of
data. Unlike `hash.digitize` method where you had to hold the whole data slice
in RAM before making a hash, the hasher struct allows you to write small chunks
of data iteratively, not keeping all of them in RAM at once.

```luau
local hasher = hash.hasher("md5")

-- do some actions
```

## `hash.write(handle: number, value: any)`

Write a chunk of data to the open hasher.

```luau
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

## `hash.finalize(handle: number) -> [number]`

Finalize hash calculation in the open hasher struct. This will close the hasher
and prevent future writes.

```luau
local hasher = hash.hasher("sha1")

hash.write(hasher, "Hello")
hash.write(hasher, "World")

-- printed the same value
print(str.encode(hash.finalize(hasher), "hex"))
print(str.encode(hash.digitize("HelloWorld"), "hex"))
```
