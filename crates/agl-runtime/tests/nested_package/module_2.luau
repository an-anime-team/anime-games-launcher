return function()
    -- obtain package_1 outputs
    local package_1 = import("package_1")

    -- obtain function from module_1 output of the package_1
    local module_1 = package_1.module_1

    return `Counter: {module_1()}`
end
