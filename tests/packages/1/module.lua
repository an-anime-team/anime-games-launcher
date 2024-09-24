local package = load("another-package-module")

return {
    greeting = package.value.hello("World")
}
