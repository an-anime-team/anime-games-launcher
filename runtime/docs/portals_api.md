# Portals API

In some cases you'd want to interact with the user by showing them some
notification, asking for a choice in a modal dialog, or request access to some
sandboxed files or folders. This API allows you to directly interact with
the user.

| Function              | Description                                   |
| --------------------- | --------------------------------------------- |
| `portals.toast`       | Show in-app notification.                     |
| `portals.notify`      | Show system notification.                     |
| `portals.dialog`      | Show in-app modal dialog.                     |
| `portals.open_file`   | Open system's default file choosing dialog.   |
| `portals.open_folder` | Open system's default folder choosing dialog. |
| `portals.save_file`   | Open system's default file saving dialog.     |

## `portals.toast(options: ToastOptions)`

Show small, optional notification to the user. This is a non-blocking function.

Toasts will be automatically hidden after some time.

```ts
type ToastOptions = {
    // Toast text.
    message: Localizable,

    // Optional field. Display a clickable button in the toast.
    action?: {
        // Text on the button.
        label: Localizable,

        // Lua function which will be executed when user clicks the button.
        callback: (): void
    }
};
```

```luau
portals.toast({
    message = "Hello World!",
    action = {
        label = "Click me",
        callback = function()
            portals.toast({
                message = {
                    en = "Thanks",
                    ru = "Спасибо"
                }
            })
        end
    }
})
```

## `portals.notify(options: NotifyOptions)`

Send in-system notification. This is a non-blocking function.

```ts
type NotifyOptions = {
    // Text of the notification's title.
    title: Localizable,

    // Text of the notification's body.
    message?: Localizable,

    // Notification icon. When unset the launcher's icon will be used.
    // Supports `file://` and freedesktop names.
    icon?: string
};
```

```luau
portals.notify({
    title = {
        en = "Simple notification",
        ru = "Простое уведомление"
    }
})

portals.notify({
    title = "Advanced notification",
    message = "Some text of your notification",
    icon = "violence-symbolic"
})
```

## `portals.dialog(options: DialogOptions) -> string`

Show in-app modal dialog and block the current thread execution until the user
selects an option within the dialog, returning name of selected button.

```ts
type DialogOptions = {
    title: Localizable,
    message: Localizable,
    buttons: DialogButton[]
};

type DialogButton = {
    // Name of the dialog button.
    name: string,

    // Text on the button.
    label: Localizable,

    // Color of the button.
    status?: 'normal' | 'suggested' | 'dangerous'
};
```

## `portals.open_file([options: OpenFileOptions]) -> OpenFileDetails | OpenFileDetails[] | null`

Open system file selection dialog. Block current thread until a file is selected,
returning either `nil` if no file selected or information about selected file
and its filesystem API handle, or list of such information if `multiple = true`.
Handles allow you to interact with the files directly using filesystem API.
Files opened this way are allowed to escape the sandbox.

```ts
type OpenFileOptions = {
    // Title of the dialog.
    title?: string,

    // Path to the directory open by default.
    directory?: string,

    // Allow selecting more than one file.
    multiple?: boolean
};

type OpenFileDetails = {
    path: string,
    handle: number
};
```

```luau
local file = portals.open_file({
    title = "Open file"
})

if file then
    print(`Open file {file.path}`)
    print(str.from_bytes(fs.read(file.handle)))
end
```

## `portals.open_folder([options: OpenFolderOptions]) -> string | string[] | null`

Open a system folder selection dialog. Block current thread until a folder
is selected, returning either `nil` if no folder selected or path to selected
folder / folders if `multiple = true`. Your module will get a permanent access
to this folder(s) and can escape the sandbox to freely read or write there.
Note that once your module is updated (its hash is changed) the access will
disappear so you will need to use paths / filesystem API to verify that the
access is still here and if you need to have it permanently it's better to
implement a small script which will not be modified in future.

```ts
type OpenFolderOptions = {
    // Title of the dialog.
    title?: string,

    // Path to the directory open by default.
    directory?: string,

    // Allow selecting more than one folder.
    multiple?: boolean
};
```

```luau
local folder = portals.open_folder({
    title = "Open folder"
})

if folder then
    fs.write_file(path.join(folder, "amogus.txt"), "sus")
end
```

## `portals.save_file([options: SaveFileOptions]) -> OpenFileDetails | null`

Open a system file saving dialog. Block current thread until a file is selected,
returning either `nil` if no file selected or information about selected file
and its filesystem API handle. Handles allow you to interact with the files
directly using filesystem API. Files created this way are allowed to escape
the sandbox.

```ts
type SaveFileOptions = {
    // Title of the dialog.
    title?: string,

    // Path to the directory open by default.
    directory?: string,

    // Default name of the file to be saved as.
    file_name?: string
};
```

```luau
local file = portals.save_file({
    title = "Save file",
    file_name = "amogus.txt"
})

if file then
    fs.write(file.handle, str.to_bytes("amogus"))
end
```
