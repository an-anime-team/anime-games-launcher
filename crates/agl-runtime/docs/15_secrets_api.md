# Secrets API

Some game integrations require private user information such as login and
password, access tokens, etc. To handle this task securely from other
integrations, secrets API stores all this data in its own storage and provides
guarded access to modules which were explicitly allowed to access named secrets
containers.

| Function              | Description                                   |
| --------------------- | --------------------------------------------- |
| `secrets.permissions` | See access permissions to a secret container. |
| `secrets.list`        | List available secret container entries.      |
| `secrets.read`        | Read secret container entries.                |
| `secrets.write`       | Write secret container entry.                 |
| `secrets.remove`      | Remove secret container entry.                |

## `secrets.permissions(container: string) -> Permissions`

See access permissions for the current module to the given secrets container.
Both `read` and `write` permissions will be set to `false` if such container
does not exist (no error or `nil` value returned).

```ts
type Permissions = {
    read: boolean;
    write: boolean;
};
```

```luau
if not secrets.permissions("my_container").write then
    error("module requires write access to my_container secrets container")
end
```

## `secrets.list(container: string) -> string[]`

List entries names of a secrets container. This function will return an error if
module doesn't have read access to the named secrets container.

```luau
dbg(secrets.list("my_container")) -- ["token"]
```

## `secrets.read(container: string, key: string) -> Bytes?`

Read entry of the secrets container. This function will return `nil` if the
entry with given key doesn't exist, and an error if module doesn't have read
access to the named secrets container.

```luau
local token = secrets.read("my_container", "token")

if not token then
    portal.toast({ message = "Token is required to launch the game" })
end
```

## `secrets.write(container: string, key: string, value: Bytes?)`

Write entry to the secrets container. This function will return an error if
module doesn't have write access to the named secrets container.

```luau
secrets.write("my_container", "token", settings.get_property("game.token"))
```

## `secrets.remove(container: string, [key: string])`

Remove secrets container entry if its key is specified, or clear the whole
secrets container otherwise. This function will return an error if module
doesn't have write access to the named secrets container.

```luau
-- Delete one entry.
secrets.remove("my_container", "token")

-- Delete all entries.
secrets.remove("my_container")
```
