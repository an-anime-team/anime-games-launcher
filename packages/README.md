# agl-packages

Packages manager and modules runtime.

## Packages manifest

- `[package]` - package's metadata fields.
    - `format` - format of the package.
    - `description` - (optional) description of the package.
    - `authors` - (optional) list of strings representings package's authors.
- `[runtime]` - (optional) modules runtime requirements.
    - `minimal_version` - (optional) minimal required version of the modules
      runtime.
- `[inputs."<name>"]` - (optional) package's inputs.
    - `uri` - relative or absolute path to the resource.
    - `format` - (optional) format of the resource.
    - `hash` - (optional) hash of the resource.
- `[outputs."<name>"]` - package's outputs.
    - `uri` - relative or absolute path to the resource.
    - `format` - (optional) format of the resource.
    - `hash` - (optional) hash of the resource.

### `package.format`

Format of the package. Currently must be equal to `1` and set explicitly.

```toml
[package]
format = 1
...
```

### `package.description`

Description of the package. This is purely a metadata field and is not
validated.

```toml
[package]
description = "Example description"
...
```

### `package.authors`

List of authors of the package, preferably in format `FirstName LastName` or
`FirstName LastName <internet_address>`.

```toml
[package]
authors = [
    "John Doe",
    "Doe John <doe@example.org>"
]
...
```

### `runtime.minimal_version`

Minimal required version of the modules runtime. This will prevent modules
evaluation in older environments (e.g. if they miss required features).

```toml
[runtime]
minimal_version = 1
...
```

### `inputs."<name>".uri` and `outputs."<name>".uri`

Relative or absolute path to the resource which will be downloaded together with
the package. Input resources will become available to the modules of the current
package while output resources will be available for other packages which will
import the current one as input dependency.

```toml
[inputs."icon.png"]
uri = "images/icon.png"
...

[outputs."module.luau"]
uri = "module.luau"
...
```

### `inputs."<name>".format` and `outputs."<name>".format`

Format of the resource. If not specified it will be automatically determined
from the URI.

| Format           | Description                                               |
| ---------------- | --------------------------------------------------------- |
| `file`           | Plain file, downloaded as is                              |
| `package`        | Another package used as dependency                        |
| `module`         | Module evaluated by the packages engine                   |
| `module/luau`    | Luau module                                               |
| `archive`        | Archive which will be extracted and available for modules |
| `archive/tar`    | Tar archive                                               |
| `archive/zip`    | Zip archive                                               |
| `archive/sevenz` | 7z archive                                                |

If no secondary format specified (e.g. just `module` and not `module/luau`) -
then it will be automatically determined as well.

```toml
[inputs."icon.png"]
format = "file"
...

[outputs."module.luau"]
format = "module"
...
```

### `inputs."<name>".hash` and `outputs."<name>".hash`

Hash of the resource which will be validated at package downloading time.

```toml
[inputs."icon.png"]
hash = "0fk62hbjlmukm"
...

[outputs."module.luau"]
hash = "2k55lf96v4b3u"
...
```
