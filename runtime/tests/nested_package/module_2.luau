return function()
    -- obtain package_1 outputs
    -- { hash, format, value }
    local package_1 = load("package_1").value

    -- obtain function from module_1 output of the package_1
    -- { hash, format, value }
    local module_1 = package_1.module_1.value

    return `Counter: {module_1()}`
end
