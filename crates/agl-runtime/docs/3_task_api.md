# Task API

By their nature lua engines are single-threaded. While they provide concurrency
mechanism called "coroutines" it cannot eliminate the nature of the engine.
Coroutines can be paused and executed one after another in a loop, all receiving
part of the global execution time. However, some functions are designed to be
blocking. This API introduces some parallelism into the lua by allowing you to
write your blocking lua functions or coroutines paired with properly
parallelized runtime APIs, e.g. filesystem one.

Understanding this API is necessary for advanced use of many other APIs.

| Function      | Description                        |
| ------------- | ---------------------------------- |
| `task.create` | Create new promise from lua value. |

## `Promise<T>`

This API introduces a special usertype called `Promise`, similar to JavaScript.
This object has a `poll` method which returns a pair of values - the first one
is the promise execution status (`true` - finished, `false` - pending, 
`nil` - aborted), and the second one is the output value of the promise.

Some promises can work fully in background while others will be running on the
lua engine.

```ts
type Promise<T> = {
    // Poll promise value. Throws an error if already finished.
    poll: (): (boolean | null, T);

    // Wait until the promise finishes its execution, blocking the engine 
    // thread to obtain its output value.
    await: (): T;

    // Abort promise execution.
    abort: (): void;

    // Whether the promise was finished.
    finished: boolean;

    // Whether the promise is running in background.
    background: boolean;
};
```

Most of the APIs provide "sync" and "async" functions - first are blocking the
lua engine until function finishes execution, and the second ones return
promises running in background. When polled the lua side only checks their
execution status and result if these are available, while actual execution is
happening in background on the rust side. This allows you to perform multiple
*standard API* operations at the same time *in parallel*.

Promises are supported by the `await` standard library function, which will
keep running the `Promise.poll` method until it finishes, returning its output.
The same can be done using promise's own `Promise.await` which is internally
called by the `await` function.

### Creating a promise in lua

A promise from the lua side can be built from 3 different lua value types:
a function, a coroutine ("thread"), or another arbitrary value.

- A function promise will call the provided callback every time a `Promise.poll`
  method is called, and result of this function is returned to the user. So the
  function provided to the promise must return `boolean | nil, any` values pair.
  If function returns `true, _` pair then the promise will be finished and the
  function will never be called again. The same will happen if the function will
  return `nil, _`, which means the promise was aborted. If `false, _` is
  returned then the function will be kept inside the promise and keep being
  called until any of other mentioned statuses is returned.

- A coroutine (thread) promise will utilize the coroutines' nature and with each
  `Promise.poll` call run `coroutine.resume`, waiting for `coroutine.yield` from
  the inner coroutine function.
  
- A promise built from any other lua type will immediately return this value
  on the very first `Promise.poll` call.

### Foreground promise

Foreground promise is built from lua code snippets (functions, coroutines or
other values) using `task.create` method. Returned promise will be suspended
(paused) by default and you will have to run `Promise.poll` method calls on it
to keep it running and to obtain its output value. Here's an example:

```luau
-- Create an example promise which will count up to 5
local count = 0

local promise = task.create(function()
    count += 1

    if count < 5 then
        return false, count
    else
        return true, count
    end
end)

-- Execute the promise explicitly
-- 
-- Alternatively you could use `promise.finished` as the loop check statement
-- but it locks the promise's inner mutex for a short period of time twice as
-- much, which is not a big deal if you don't worry about performance too much
while true do
    local status, result = promise:poll()

    dbg(result)

    if status then
        break
    end
end
```

As you can see the promise is executed step by step on the lua engine, meaning
it's blocking engine thread. Its `Promise.background` is also equals to `false`.

### Background promise

Background promises are returned by the standard API functions. Unlike
foreground promises which perform some job when polled these are running the
actual job in background on the rust side, and provided `Promise.poll` method
only checks the execution status and, if finished, returns the output value.
This allows background promises to perform multiple operations *in parallel* 
while not blocking the lua engine thread.

Background promises cannot be built from the lua side because executing some lua
code would involve blocking the whole engine thread, making it impossible to
run multiple jobs in parallel. So, background promises are returned only from
the standard library APIs.

## `task.create(task: any) -> Promise`

Create a promise object from provided lua type.

```luau
-- An example foreground promise built from a function
local promise = task.create(function()
    sleep(5000)

    return true, 123
end)

dbg(promise:await()) -- 123
```
