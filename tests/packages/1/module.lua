local package = load("another-package-module")

return {
    greeting = package.hello("World")
}
