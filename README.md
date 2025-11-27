# Anime Games Launcher

A long work in progress universal launcher for Linux (as a primary platform).

The project is split into the following subcrates:

1. Core library [`agl-core`](./core) implements default abstractions over
   various hashing, compression/decompression and archives format, implements
   files downloading mechanism, provides async tasks API and a couple of utility
   structs. Can be fine-grained with features to choose which components you
   need to use.
2. Packages manager [`agl-packages`](./packages) implements a Nix-like packages
   manager with simple inputs and outputs JSON files. This packages manager
   can download files, extract archives and process nested packages as
   dependencies.
3. Modules runtime [`agl-runtime`](./runtime) implements a sandboxed, scoped 
   luau scripts standard library and a runtime struct which can be used to run
   scripts downloaded by the packages manager.
4. Games API [`agl-games-api`](./games-api) - TBD
5. Anime Games Launcher [`anime-games-launcher`](./launcher) - TBD

The whole project and all its components listed in this repo are licensed 
under [GPL-3.0-or-later](./LICENSE)
