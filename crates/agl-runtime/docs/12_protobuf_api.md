# Protobuf API

[Protobuf (Protocol Buffers)](https://protobuf.dev) is a standard way of
representing complex data structures in a binary format, efficiently compressing
and aligning it. This API allows you to read protobuf schemas, encode and decode
messages.

| Function            | Description                                             |
| ------------------- | ------------------------------------------------------- |
| `protobuf.create`   | Create protobuf object from provided `.proto` schema.   |
| `protobuf.messages` | List available protobuf messages names.                 |
| `protobuf.encode`   | Encode lua value into `Bytes` using protobuf schema.    |
| `protobuf.decode`   | Decode `Bytes` into a lua object using protobuf schema. |

## `protobuf.create(schema: string) -> Protobuf`

Create `Protobuf` object from the given protobuf schema. The object stores all
the messages structures defined in the schema and can be used to encode and
decode binary data.

```luau
-- Define a simple protobuf schema for one single message called "Person"
local schema = protobuf.create([[
    syntax = "proto3";

    enum Status {
        INACTIVE = 0;
        ACTIVE   = 1;
    }

    message Person {
        int32 id      = 1;
        string name   = 2;
        string email  = 3;
        Status status = 4;
    }
]])
```

## `protobuf.messages(schema: Protobuf) -> string[]`

List names of all the messages available in a protobuf schema.

```luau
dbg(protobuf.messages(schema)) -- ["Person"]
```

## `protobuf.encode(schema: Protobuf, message: string, values: table) -> Bytes`

Encode given lua value into the given message format using given protobuf
schema.

Since every protobuf message has indexed fields - it's possible to encode a
protobuf message from either a key-value table, or a sequential values table.

```luau
-- Encode a "Person" message using provided named values
local encoded_value = protobuf.encode(schema, "Person", {
    id     = 1,
    name   = "KRypt0n_",
    email  = "krypt0nn@dawn.wine",
    status = "ACTIVE" -- number "1" would work too
})

-- Encode a "Person" message using provided indexed values
local encoded_value = protobuf.encode(schema, "Person", {
    1,
    "KRypt0n_",
    "krypt0nn@dawn.wine",
    1 -- string "ACTIVE" would work too
})

dbg(encoded_value)
```

## `protobuf.decode(schema: Protobuf, message: string, value: Bytes) -> table`

Decode given binary data using given message format and protobuf schema.

```luau
-- Decode a "Person" message
local decoded_value = protobuf.decode(schema, "Person", encoded_value)

print(`  Name: {decoded_value.name}`)   -- "KRypt0n_"
print(`    Id: {decoded_value.id}`)     -- "1"
print(` Email: {decoded_value.email}`)  -- "krypt0nn@dawn.wine"
print(`Status: {decoded_value.status}`) -- "ACTIVE"
```
