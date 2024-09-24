local package = load("another-package-module")

return {
    greeting = package.value.hello("World"),

    test_load_valid_input = pcall(load, "another-package"),
    test_load_valid_output = pcall(load, "module"),

    test_load_invalid_input = pcall(load, "file-reference"),
    test_load_invalid_output = pcall(load, "dxvk"),

    test_load_unexisting_input = pcall(load, "amogus")
}
