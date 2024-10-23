# Introduction

Anime Games Launcher is a universal platform for different
games deployment. It's powered by luau engine with a custom
standard library to improve developers experience and support
operations sandboxing.

Launcher utilizes custom packages manager inspired by nix platform.
Every package has set of input and output resources, which are URIs
to other files or packages, plus other metadata. Resources have
different formats - raw files, archives, other packages and luau modules.

Packages are logically stored in lock files. They contain a set
of metadata and resolved dependency tree. On start launcher loads
the latest available lock file and validates that all the packages
are physically existing on the disk and their hashes are valid.
This prevents intentional packages swapping and ensures consistency
between uses.

Games are split into two different standards - metadata manifest
and lua integration. All the static data of the game - its title,
details, URLs to pictures, tags, etc - stored in special json
files. Launcher fetches them on start to render games info.
Technical details - integration itself - is written on luau
programming language.

Locked games integration packages (lock file) combined with
downloaded games manifests (json files) form a new single
json file called "generation". Generation files allow launcher
to load games added by the user at constant time. Instead of
fetching information from the internet we're loading information
from a single json file, ensure its consistency, verify hashes
and render games-related UI elements. In background launcher
creates a task to create a new generation file by fetching
games packages and manifests again. When new generation is made
and it's different from the currently used one - it will be
automatically loaded on the next launcher start, allowing packages
developers to publish their updates which will be automatically
downloaded by packages users. If loaded generation is broken
(e.g. is missing some package file) - older one will be loaded.
If all the generations are broken - launcher will wait for the new one.

Each game has its own target platform. In most cases
it's `x86_64-windows-native` (a windows game). Launcher allows
developers to specify platforms supported by their
integration package. E.g. a lot of games could be supported by the
`x86_64-linux-wine64`, so developers could list this value
to allow people on linux to launch the game using wine64.

Games launching is happening in isolated environments called "profiles".
Each profile has its own set of settings and, depending on platform,
can enable isolation techniques (sandboxing).

> Despite all the effort I made to sandbox packages and games
> it doesn't mean it's physically impossible - no, it is
> pretty much possible. You should not trust the sandbox if you
> plan to execute intentionally malicious code. I put my best effort
> to make developers lifes' as much difficult as possible to implement
> such a luau module to escape the sandbox, but with enough patience
> somebody will definitely find a way to escape it.
