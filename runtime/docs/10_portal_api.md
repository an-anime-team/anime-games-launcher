# Portal API

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
    message: Localizable;

    // Optional field. Display a clickable button in the toast.
    action?: {
        // Text on the button.
        label: Localizable;

        // Lua function which will be executed when user clicks the button.
        callback: (): void;
    };
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
    title: Localizable;

    // Text of the notification's body.
    message?: Localizable;

    // Notification icon. When unset the launcher's icon will be used.
    // Supports `file://` and freedesktop names.
    icon?: string;
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
    title: Localizable;
    message: Localizable;
    buttons: DialogButton[];
};

type DialogButton = {
    // Name of the dialog button.
    name: string;

    // Text on the button.
    label: Localizable;

    // Color of the button.
    status?: 'normal' | 'suggested' | 'dangerous';
};
```

## `portals.open_file([options: OpenFileOptions]) -> string | string[] | null`

Open system file selection dialog. Block current thread until a file is
selected, returning either `nil` if no file selected, path to selected file,
or list of paths if `multiple = true`.

Selected paths are temporary allowed to be read, but not modified (read-only
access).

```ts
type OpenFileOptions = {
    // Title of the dialog.
    title?: string;

    // Path to the directory open by default.
    directory?: string;

    // Allow selecting more than one file.
    multiple?: boolean;
};
```

```luau
local file_path = portals.open_file({
    title = "Open file"
})

if file_path then
    print(`Selected path: {file_path}`)

    -- Selected path can be read
    assert(path.permissions(file_path).read)
end
```

## `portals.open_folder([options: OpenFolderOptions]) -> string | string[] | null`

Open a system folder selection dialog. Block current thread until a folder
is selected, returning either `nil` if no folder selected, path to selected
folder, or a list of paths to selected folders if `multiple = true`.

Selected paths are temporary allowed to be read, but not modified (read-only
access).

```ts
type OpenFolderOptions = {
    // Title of the dialog.
    title?: string;

    // Path to the directory open by default.
    directory?: string;

    // Allow selecting more than one folder.
    multiple?: boolean;
};
```

```luau
local folder_path = portals.open_folder({
    title = "Open folder"
})

if folder_path then
    print(`Selected path: {folder_path}`)

    -- Selected path can be read
    assert(path.permissions(folder_path).read)
end
```

## `portals.save_file([options: SaveFileOptions]) -> string | null`

Open a system file saving dialog. Block current thread until a file is selected,
returning either `nil` if no file selected or path to the selected file.

Selected path is temporary allowed to be written to (read-write access).

```ts
type SaveFileOptions = {
    // Title of the dialog.
    title?: string;

    // Path to the directory open by default.
    directory?: string;

    // Default name of the file to be saved as.
    file_name?: string;
};
```

```luau
local file_path = portals.save_file({
    title = "Save file",
    file_name = "amogus.txt"
})

if file_path then
    fs.write_file(file_path, "Hello, World!")
end
```
