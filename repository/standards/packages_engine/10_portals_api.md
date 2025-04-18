# Portals API

In some cases you'd want to interact with the user by showing them some
notification, asking for a choice in a modal dialog, or request access to
some extended privilege APIs or sandboxed files or directories. This API allows
you to directly interact with the user.

| Function          | Description                           |
| ----------------- | ------------------------------------- |
| `portals.toast`   | Show in-app notification.             |
| `portals.notify`  | Show system notification.             |
| `portals.dialog`  | Show in-app modal dialog.             |
| `portals.request` | Request user for extended privileges. |

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

## `portals.request(...)`

TBD
