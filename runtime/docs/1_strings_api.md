# Strings API

Rust-lua bridge API is designed to return raw bytes instead of strings. While
many methods can accept strings and other data types as their inputs,
outputs always return tables of bytes. Strings API provides functions to perform
bytes-string conversions, support for data encoding and serialization.

| Function         | Description                      |
| ---------------- | -------------------------------- |
| `str.to_bytes`   | Convert value to a bytes slice.  |
| `str.from_bytes` | Convert bytes slice to a string. |
| `str.encode`     | Encode value to a string.        |
| `str.decode`     | Decode value from a string.      |

## Supported encodings

Following table contains list of `StringEncoding` enum values.

| Name                   | Description                                          |
| ---------------------- | ---------------------------------------------------- |
| `base16` or `hex`      | Convert bytes array to base16 (hex) string.          |
| `base32`               | Convert bytes array to base32 string (`base32/pad`). |
| `base32/pad`           | RFC 4648 lower with padding.                         |
| `base32/nopad`         | RFC 4648 lower without padding.                      |
| `base32/hex-pad`       | RFC 4648 hex lower with padding.                     |
| `base32/hex-nopad`     | RFC 4648 hex lower without padding.                  |
| `base64`               | Convert bytes array to base64 string (`base64/pad`). |
| `base64/pad`           | Standard lower with padding.                         |
| `base64/nopad`         | Standard lower without padding.                      |
| `base64/urlsafe-pad`   | URL-safe with padding.                               |
| `base64/urlsafe-nopad` | URL-safe without padding.                            |
| `json`                 | Convert given value to a JSON string.                |
| `toml`                 | Convert given value to a TOML string.                |
| `yaml`                 | Convert given value to a YAML string.                |

## `str.to_bytes(value: any, [charset: string]) -> [number]`

Convert string (or some other values) to a bytes vector.

If charset is specified, then the given value's byte representation will be
interpreted as UTF-8 encoded string, and this method will try to convert it into
a given charset.

> Note: this method internally uses the same algorithm as many other methods in
> the standard which accept many types.
>
> E.g. when hashing a value it firstly will be converted into bytes using this
> method.

```luau
print(str.to_bytes("abc")) -- [97, 98, 99]
print(str.to_bytes(0.5)) -- [63, 224, 0, 0, 0, 0, 0, 0]
print(str.to_bytes({ 1, 2, 3 })) -- [1, 2, 3]

local a = str.from_bytes({ 208, 176, 208, 177, 208, 190, 208, 177, 208, 176 })
local b = str.to_bytes(a, "cp1251")

-- Cyrillic is encoded using 1 byte in cp1251:
-- [224, 225, 238, 225, 224]
print(b)
```

## `str.from_bytes(bytes: [number], [charset: string]) -> string`

Convert bytes slice into a lua string. If charset is specified, then
this method will try to decode bytes from this charset into UTF-8.

```luau
local a = str.from_bytes({ 224, 225, 238, 225, 224 }, "cp1251")
local b = str.to_bytes(a)

-- Cyrillic is encoded using 2 bytes in UTF-8:
-- [208, 176, 208, 177, 208, 190, 208, 177, 208, 176]
print(b)
```

## `str.encode(value: any, encoding: StringEncoding) -> string`

Encode given value to a string.

```luau
print(str.encode(123, "base16"))               -- 7b
print(str.encode("Hello, World!", "base64"))   -- "SGVsbG8sIFdvcmxkIQ=="
print(str.encode({ hello = "world" }, "json")) -- "{\"hello\":\"world\"}"
```

## `str.decode(value: string, encoding: StringEncoding) -> any`

Decode given string to a bytes slice.

```luau
print(str.decode("7b", "base16"))                                   -- [0, 0, 0, 123]
print(str.from_bytes(str.decode("SGVsbG8sIFdvcmxkIQ==", "base64"))) -- "Hello, World!"
print(str.decode("{\"hello\":\"world\"}", "json"))                  -- { hello = "world" }
```
