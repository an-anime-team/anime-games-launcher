return function()
    -- obtain function returned by module_1.luau
    -- { hash, format, value }
    local module_1 = load("module_1").value

    return `Hello, {module_1()}!`
end
