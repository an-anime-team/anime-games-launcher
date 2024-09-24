local package = load("another-package-module")

return {
    greeting = package.value.hello("World"),

    test_load_valid_input = load("another-package"),
    test_load_valid_output = load("self-reference"),

    test_load_invalid_input = load("file-reference"),
    test_load_invalid_output = load("dxvk"),

    test_load_unexisting_input = load("amogus")
}
