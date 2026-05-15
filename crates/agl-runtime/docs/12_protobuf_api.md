# Protobuf API

[Protobuf (Protocol Buffers)](https://protobuf.dev) is a standard way of
representing complex data structures in a binary format, efficiently compressing
and aligning it. This API allows you to read protobuf schemas, encode and decode
messages.

| Function          | Description                                             |
| ----------------- | ------------------------------------------------------- |
| `protobuf.create` | Create protobuf object from provided `.proto` schema.   |
| `protobuf.encode` | Encode lua value into `Bytes` using protobuf schema.    |
| `protobuf.decode` | Decode `Bytes` into a lua object using protobuf schema. |

## `protobuf.create(schema: string) -> Protobuf`

Create `Protobuf` object from the given protobuf schema. The object stores all
the messages structures defined in the schema and can be used to encode and
decode binary data.

```luau
-- Define a simple protobuf schema for one single message called "Person"
local schema = protobuf.create([["
    edition = "2024";

    message Person {
        string name = 1;
        int32 id = 2;
        string email = 3;
    }
"]])
```

## `protobuf.encode(schema: Protobuf, message: string, value: table) -> Bytes`

Encode given lua value into the given message format using given protobuf
schema.

```luau
-- Encode a "Person" message using provided values
local encoded_value = protobuf.encode(schema, "Person", {
    name = "KRypt0n_",
    id = 1,
    email = "krypt0nn@vk.com"
})

dbg(encoded_value)
```

## `protobuf.decode(schema: Protobuf, message: string, value: Bytes) -> table`

Decode given binary data using given message format and protobuf schema.

```luau
-- Decode a "Person" message
local decoded_value = protobuf.decode(schema, "Person", encoded_value)

print(` Name: {decoded_value.name}`)  -- "KRypt0n_"
print(`   Id: {decoded_value.id}`)    -- "1"
print(`Email: {decoded_value.email}`) -- "krypt0nn@vk.com"
```
