# Sync API

Some packages would like to communicate with each other, e.g. different version
of the same package. Sync API provides a set of data synchronization primitives
for this.

## Channels

| Function             | Description                                |
| -------------------- | ------------------------------------------ |
| `sync.channel.open`  | Open inter-packages communication channel. |
| `sync.channel.send`  | Send a new message to the channel.         |
| `sync.channel.recv`  | Receive a new message from the channel.    |
| `sync.channel.close` | Close an open channel.                     |

### `sync.channel.open(key: string) -> number`

Subscribe to a channel with a given key (name). After subscription you receive a
special identifier which will be used to hold your read messages history.
You can't read messages which were sent before you obtained this identifier.

```luau
local channel = sync.channel.open("my_package_channel")
```

### `sync.channel.send(handle: number, message: any)`

Send some value to the open channel. Message will be sent to all the packages
which have this channel open except you.

> NOTE: currently not *any* value is supported due to technical difficulties.
> Sent values are also not shared, meaning they all are cloned.

```luau
local channel = sync.channel.open("my_package_channel")

sync.channel.send(channel, "Hello, World!")
sync.channel.send(channel, { 1, 2, 3 })
sync.channel.send(channel, true)

-- messages sent by you will be visible to other
-- channel users, but you will not see them yourself.
print(sync.channel.recv(channel)) -- nil
```

### `sync.channel.recv(handle: number) -> any | nil, bool`

Try to receive a message from the open channel. This is a non-blocking method
which will return `nil` if there's no messages to read. Second returned value
means status of the returned value. Since `nil` could be sent in the channel as
a message, second value indicates its status. For every valid message it's
`true` while for channel end message it's `false`.

```luau
local sender   = sync.channel.open("my_package_channel")
local receiver = sync.channel.open("my_package_channel")

sync.channel.send(sender, 1)
sync.channel.send(sender, 2)
sync.channel.send(sender, nil)
sync.channel.send(sender, 3)

repeat
    local message, status = sync.channel.recv(receiver)

    -- 1, 2, nil, 3
    print(message)
until status

sync.channel.close(sender)
sync.channel.close(receiver)
```

### `sync.channel.close(handle: number)`

Close the open channel. This will clear all the remaining messages and prevent
future writes to your identifier.

```luau
local channel = sync.channel.open("my_package_channel")

-- do some operations

sync.channel.close(channel)
```

## Mutex

Mutex is the most used synchronization primitive. It allows you to block
code execution while another thread (module, package) is using the mutex.

| Function            | Description                |
| ------------------- | -------------------------- |
| `sync.mutex.open`   | Open inter-packages mutex. |
| `sync.mutex.lock`   | Lock an open mutex.        |
| `sync.mutex.unlock` | Unlock an open mutex.      |
| `sync.mutex.close`  | Close an open mutex.       |

### `sync.mutex.open(key: string) -> number`

Get handle to the mutex with given key identifier. This handle is used to
lock and unlock the same mutex from different packages and modules.

```luau
local mutex = sync.mutex.open("my_module_mutex")
```

### `sync.mutex.lock(handle: number)`

Block code execution until the mutex is locked by you. Once another module
unlocks the mutex you (or some another module) will be able to lock it and
continue execution. After unlocking the mutex you allow other modules to
continue execution. This can be used if your module downloads resources
from the internet and you have different versions of the same module. Using
mutex you can block other modules from downloading the resources.

```luau
-- first module
local mutex = sync.mutex.open("my_module_mutex")

sync.mutex.lock(mutex)

-- do some operations

sync.mutex.close(mutex)
```

```luau
-- second module
local mutex = sync.mutex.open("my_module_mutex")

sync.mutex.lock(mutex)

-- do some operations

sync.mutex.unlock(mutex)
```

### `sync.mutex.unlock(handle: number)`

Unlock the mutex, allowing other modules to lock it and continue execution.

```luau
local mutex = sync.mutex.open("my_module_mutex")

sync.mutex.lock(mutex)

-- do some operations

sync.mutex.unlock(mutex)
```

### `sync.mutex.close(handle: number)`

Close the mutex handle. Closing locked mutex will automatically unlock it.

```luau
local mutex = sync.mutex.open("my_module_mutex")

sync.mutex.lock(mutex)

-- do some operations

sync.mutex.close(mutex)
```
