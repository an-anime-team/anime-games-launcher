# System API

## `system.local_time([format: string]) -> number | string`

Get local system time. If no format string provided - then timestamp in seconds
is returned (amount of seconds passed since unix epoch, i.e. January 1, 1970,
00:00:00 UTC).

| Format name | Example value                         |
| ----------- | ------------------------------------- |
| `iso8601`   | `1997-11-21T09:55:06.000000000-06:00` |
| `rfc2822`   | `Fri, 21 Nov 1997 09:55:06 -0600`     |
| `rfc3339`   | `1985-04-12T23:20:50.52Z`             |

Besides some standard names the format can also be specified using `[year]`,
`[month]`, `[day]`, `[hour]`, `[minute]`, `[second]` and more. Proper
documentation can be found [here](https://time-rs.github.io/book/api/format-description.html).

```luau
print(system.local_time()) -- 1767463406
print(system.local_time("[day].[month].[year] [hour]:[minute]:[second]")) -- 03.01.2026 18:03:26
print(system.local_time("rfc2822")) -- Sat, 03 Jan 2026 18:03:26 +0200
```

## `system.utc_time([format: string]) -> number | string`

Similar to `system.local_time`, except a UTC time is returned. If no format
string provided - then the output of this function is a unix timestamp in
seconds.

```luau
print(system.utc_time()) -- 1767456206
print(system.utc_time("[day].[month].[year] [hour]:[minute]:[second]")) -- 03.01.2026 16:03:26
print(system.utc_time("rfc2822")) -- Sat, 03 Jan 2026 16:03:26 +0000
```

## `system.env([name: string]) -> { [name: string]: string } | string | nil`

Get table of all the current processe's environment variables. If variable name
is provided then either its value is returned or `nil` if variable with this
name doesn't exist.

```luau
-- Get table of all the variables.
dbg(system.env())
```

## `system.find(...names: string) -> ...(string | nil)`

Try to find paths to provided filesystem entries. If file or folder is not
found - `nil` is returned.

The search will happen through the `PATH` environment variable. The same result
can be achieved using `system.env` and `fs.exists` APIs.

```luau
local bash, amogus = system.find("bash", "amogus")

dbg(bash) -- likely path to the bash binary (who doesn't have it?)
dbg(amogus) -- likely "nil" since there's no such file/folder
```
