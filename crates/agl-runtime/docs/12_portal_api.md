# Portal API

In some cases you'd want to interact with the user by showing them some
notification, asking for a choice in a modal dialog, or request access to some
sandboxed files or folders. This API allows you to directly interact with
the user.

| Function             | Description                                   |
| -------------------- | --------------------------------------------- |
| `portal.toast`       | Show in-app notification.                     |
| `portal.notify`      | Show system notification.                     |
| `portal.dialog`      | Show in-app modal dialog.                     |
| `portal.open_file`   | Open system's default file choosing dialog.   |
| `portal.open_folder` | Open system's default folder choosing dialog. |
| `portal.save_file`   | Open system's default file saving dialog.     |

## `portal.toast(options: ToastOptions)`

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
portal.toast({
    message = "Hello, World!",
    action = {
        label = "Click me",
        callback = function()
            portal.toast({
                message = {
                    en = "Thanks",
                    ru = "Спасибо"
                }
            })
        end
    }
})
```

## `portal.notify(options: NotifyOptions)`

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
portal.notify({
    title = {
        en = "Simple notification",
        ru = "Простое уведомление"
    }
})

portal.notify({
    title = "Advanced notification",
    message = "Some text of your notification",
    icon = "violence-symbolic"
})
```

## `portal.dialog(options: DialogOptions) -> string`

Show in-app modal dialog and block the current thread execution until the user
selects an option within the dialog, returning name of selected button.

```ts
type DialogOptions = {
    // Dialog title.
    title: Localizable;

    // Dialog message (body).
    message: Localizable;

    // Optional list of dialog buttons.
    buttons?: DialogButton[];

    // Whether to add default "close" button to the dialog. If set to `false`
    // then this button will not be added, so user will have to choose one of
    // the provided buttons. Has no effect when no buttons provided.
    can_close?: boolean;
};

type DialogButton = {
    // Text on the button.
    label: Localizable;

    // Color of the button.
    status?: 'normal' | 'suggested' | 'dangerous';

    // Lua function which will be executed when user clicks the button.
    callback?: (): void;
};
```

## `portal.open_file([options: OpenFileOptions]) -> string | string[] | null`

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
local file_path = portal.open_file({
    title = "Open file"
})

if file_path then
    print(`Selected path: {file_path}`)

    -- Selected path can be read
    assert(path.permissions(file_path).read)
end
```

## `portal.open_folder([options: OpenFolderOptions]) -> string | string[] | null`

Open a system folder selection dialog. Block current thread until a folder
is selected, returning either `nil` if no folder selected, path to selected
folder, or a list of paths to selected folders if `multiple = true`.

Selected paths are temporary allowed to be written to (read-write access).

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
local folder_path = portal.open_folder({
    title = "Open folder"
})

if folder_path then
    print(`Selected path: {folder_path}`)

    -- Selected path can be read
    assert(path.permissions(folder_path).read)
end
```

## `portal.save_file([options: SaveFileOptions]) -> string | null`

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
local file_path = portal.save_file({
    title = "Save file",
    file_name = "amogus.txt"
})

if file_path then
    fs.write_file(file_path, "Hello, World!")
end
```
