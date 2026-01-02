# Bytes type

`Bytes` userdata is an immutable type that stores binary data on the rust side
and provides lua methods to read them. Most of the standard library functions
will return this type for memory efficiency.

You can't build this type from lua side. Instead, all the API functions accept
lua strings or tables with bytes stored as sequence values. In API methods'
definitions all the lua types which can be converted in `Bytes` are called
`Bytes`.

## `Bytes.as_table() -> number[]`

Convert bytes type into a lua table.

```luau
local bytes: Bytes = ...

for _, byte: number in ipairs(bytes:as_table()) do
    dbg(byte)
end
```

## `Bytes.as_string() -> string`

Convert bytes type into a lua string. It is possible since lua strings don't
require any specific encoding.

```luau
local bytes: Bytes = ...

dbg(bytes:as_string())
```

## `Bytes.len -> number`

Get length of the buffer. Since it's immutable the value will never change.

## `Bytes.pos -> number`

Get current cursor position in the buffer.

## `Bytes.read() -> number[] | nil`

Read content of the bytes buffer. Return lua table of read bytes or `nil` if end
of buffer reached.

```luau
local bytes: Bytes = ...
local len = 0

while true do
    local result = bytes:read()

    if not result then
        break
    end

    len += #result
end

print(`buffer length: {len} bytes`)
```

## `Bytes.read_exact(len: number) -> number[]`

Try to read exact number of bytes from the buffer. Return lua table with read
bytes or an error if end of buffer reached or if it doesn't have enough bytes.

```luau
local bytes: Bytes = ...

if bytes:read_exact(3) == str.to_bytes("png") then
    -- do something
end
```

## `Bytes.seek(pos: number) -> number`

Set buffer cursor to the specified absolute position if it's a positive value,
and to `Bytes.len - |pos|` if its value is negative. Out of bounds seeking is
not allowed and will return an error.

Return new cursor position of the buffer.

```luau
local bytes: Bytes = ...

dbg(bytes:seek(-1) == bytes.len - 1)
```

## `Bytes.seek_rel(offset: number)`

Set buffer cursor to the specified position relative to the current buffer
position. If provided offset has positive value - then position is changed to
`Bytes.pos + offset`. If value is negative - `Bytes.pos - |offset|`. Out of
bounds seeking is not allowed and will return an error.

Return new cursor position of the buffer.

```luau
local bytes: Bytes = ...

local buf_1 = bytes.read()

bytes.seek_rel(-#buf_1)

local buf_2 = bytes.read()

dbg(buf_1 == buf_2) -- true
```
