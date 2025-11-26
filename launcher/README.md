# Anime Games launcher

Experimental games store-like application (internally called "anime steam") powered by custom packages manager and standardized lua scripts for games integration.

Heavily work in progress. Watch for development process in our discord server.

| Store page | Store details page | Library details page |
| - | - | - |
| <img src="repository/pictures/store.png" /> | <img src="repository/pictures/store-details.png" /> | <img src="repository/pictures/library-details.png" /> |

## Builds

This repository uses GitHub CI to automatically check source code on errors using the `flake.nix`.
When new releases are pushed CI compiles release build on latest ubuntu version, prepares
RPM, DEB and AppImage builds (which should not really be used) and publishes compiled build
to our [cachix binary cache](https://an-anime-team.cachix.org).

# Declaration of openness / Декларация открытости

I believe that in a changing world it is extremely important to remain honest
with your users. I believe in open source software, so I think it's important
to state the following.

Я верю что в меняющемся мире крайне важно оставаться честными со своими пользователями.
Я верю в открытое программное обеспечение, поэтому считаю важным заявить следующее.

## English

The project always was, still is and will be free and open for every person
without exclusion, no matter what nationality or religion they have, as long
as I remain its core developer. Restricting access to the project
creates a precedent that can be reused in the future.

There has never been, is not, and will never be intentionally malicious code
that works under certain conditions, whether it is geolocation or any other
user metadata. The project will not collect user telemetry without their
express consent.

## Русский

Проект всегда был, остается и будет бесплатным и открытым для всех людей
без исключений, независимо от их национальности и вероисповедания, до тех пор,
пока я остаюсь его основным разработчиком. Ограничение доступа к проекту
создает прецедент, который может быть использован повторно в будущем.

В проекте никогда не было, нет и не будет умышленно вредоносного кода,
срабатывающего в определенных условиях, будь то геолокация или любые иные
метаданные пользователя. Проект не будет собирать телеметрию пользователей
без их прямого согласия на это.

Licensed under [GPL-3.0](./LICENSE).
