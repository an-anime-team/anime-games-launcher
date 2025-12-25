return function()
    -- obtain file path from the loaded value
    -- { hash, format, value }
    local file_path = load("file").value

    -- read the file's content
    local file_content = fs.read_file(file_path)

    -- decode this content from utf-8 bytes into a string
    return str.from_bytes(file_content)
end
