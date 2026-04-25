# Loadout

[![Built For GNOME](https://img.shields.io/badge/GNOME-4A86CF?logo=gnome&logoColor=white)](https://www.gnome.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Mass manage Steam launch options, Proton versions, and Gamescope presets from Loadout.

## Features

- Scan installed Steam games
- View and edit launch options in bulk
- Apply Proton versions across multiple games
- Save reusable Gamescope presets
- Back up changes before writing them

## Build

```sh
meson setup build
meson compile -C build
```

## Run

```sh
meson devenv -C build src/loadout
```

## License

Loadout is licensed under the MIT License.

TLDR: Do what you like, I don't care!
