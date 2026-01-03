# SQLite API

SQLite is the most used relative database format. This API allows you to
interact with sqlite databases by querying data from them or executing
SQL commands.

| Function           | Description                            |
| ------------------ | -------------------------------------- |
| `sqlite.open`      | Open SQLite database from given file.  |
| `sqlite.exec`      | Execute single SQL command.            |
| `sqlite.batch`     | Execute multiple SQL commands at once. |
| `sqlite.query`     | Query multiple rows from the database. |
| `sqlite.query_row` | Query single row from the database.    |
| `sqlite.close`     | Close SQLite database.                 |

## `sqlite.open(path: string) -> number`

Open SQLite database in given file path or create a new one, returning handle
of the database connection which can be used in other methods.

```luau
local handle = sqlite.open("settings.db")
```

## `sqlite.exec(handle: number, command: string, [params: any[]]) -> Promise<number>`

Execute single SQL command, returning latest *inserted* row id. If no inserts
happened - old value will be returned.

Note that you can't use this method to execute several commands at the same
time. Instead you will have to use `sqlite.batch`.

This method will cache atomic commands after initial call, increasing
performance of the following calls of the same command.

```luau
local handle = sqlite.open("example.db")

-- You can specify params which will be properly fed to the command.
local row_id = sqlite.exec(handle, "INSERT INTO your_table (column1, column2) VALUES (?1, ?2)", {
    "Example value 1",
    "Example value 2"
}):await()

-- Delete just inserted row using its id.
sqlite.exec(handle, "DELETE FROM your_table WHERE rowid = ?1", { row_id }):await()

-- Always close your database connections.
sqlite.close(handle)
```

## `sqlite.batch(handle: humber, command: string) -> Promise<void>`

Execute multiple SQL commands. Unlike `sqlite.exec` here you can't provide
params to the command. Instead you have to modify the command itself and ensure
data escaping manually.

```luau
local handle = sqlite.open("example.db")

local rows = sqlite.batch(handle, [[
    BEGIN TRANSACTION;
        INSERT INTO your_table (column1, column2) VALUES ('value1', 'value2');
        INSERT INTO your_table (column1, column2) VALUES ('value3', 'value4');
        INSERT INTO your_table (column1, column2) VALUES ('value5', 'value6');
    COMMIT;
]]):await()

print(`Query affected {rows} rows`)

-- Always close your database connections.
sqlite.close(handle)
```

## `sqlite.query(handle: number, query: string, [params: any[]]) -> Promise<table[] | nil>`

Query multiple rows from the database. This method will return a list of rows
stored as lua tables or `nil` if no rows were found. Note that this is a
blocking method which will return the whole response at once. If you need
to process large amounts of data - consider adding limits and offsets to
your query.

```luau
local handle = sqlite.open("example.db") -- empty database

-- Will return nil because there's no rows in your_table
if not sqlite.query(handle, "SELECT * FROM your_table"):await() then
    print("No rows matched")
end

-- Insert some example values
sqlite.batch(handle, [[
    INSERT INTO your_table (column1, column2) VALUES ('value1', 'value2');
    INSERT INTO your_table (column1, column2) VALUES ('value3', 'value4');
    INSERT INTO your_table (column1, column2) VALUES ('value5', 'value6');
]]):await()

local rows = sqlite.query(handle, "SELECT rowid, column1, column2 FROM your_table"):await()

for _, row in ipairs(rows) do
    print(`[row {row[1]}] column1 = {row[2]}, column2 = {row[3]}`)
end

-- Always close your database connections.
sqlite.close(handle)
```

## `sqlite.query_row(hande: number, query: string, [params: [any]]) -> Promise<table | nil>`

Query single row from the database. Unlike `sqlite.query` this method will
return the first matched row and stop execution.

This method will return `nil` if no rows matched.

```luau
local handle = sqlite.open("example.db") -- empty database

-- Will return nil because there's no rows in your_table
if not sqlite.query_row(handle, "SELECT * FROM your_table"):await() then
    print("No rows matched")
end

-- Insert some example values
sqlite.batch(handle, [[
    INSERT INTO your_table (column1, column2) VALUES ('value1', 'value2');
    INSERT INTO your_table (column1, column2) VALUES ('value3', 'value4');
    INSERT INTO your_table (column1, column2) VALUES ('value5', 'value6');
]]):await()

local row = sqlite.query_row(handle, "SELECT rowid FROM your_table"):await()

dbg(row[1]) -- 1 (first inserted row)

row = sqlite.query_row(handle, "SELECT rowid FROM your_table WHERE rowid > 1"):await()

dbg(row[1]) -- 2 (second inserted row)

-- Always close your database connections.
sqlite.close(handle)
```

## `sqlite.close(handle: number)`

Close database connection. This method will also run `PRAGMA optimize` task
on the database file before closing it and prevent future uses of given handle.

```luau
local handle = sqlite.open("example.db")

-- Do something

sqlite.close(handle)
```
