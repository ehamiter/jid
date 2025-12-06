# `jid` (`j`ot `i`t `d`own)

A distraction-free writing app built with [GPUI](https://www.gpui.rs/).

## What it does

`jid` is a minimal text editor for focused writing. No menus, no toolbars—just a centered editor and your words.

- **Auto-save** — Documents save automatically to `~/Documents/jid/`
- **Multiple themes** — Cycle through themes with `Cmd+Shift+T`
- **Focus mode** — Dims all lines except the current one for distraction-free writing
- **Keyboard-first** — All actions via shortcuts

## Usage

```
cargo run
```

## Building the App

To build a macOS app bundle:

```
./build_app.sh
```

To install to Applications:

```
cp -r target/release/jid.app /Applications/
```

## Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| Save | `Cmd+S` |
| Cycle theme | `Cmd+Shift+T` |
| Toggle focus mode | `Cmd+.` |
| Open config | `Cmd+,` |
| Quit | `Cmd+Q` |

## Themes

| Dark | Light |
|:----:|:-----:|
| ![Dark](screenshots/theme-dark.png) | ![Light](screenshots/theme-light.png) |

| Sepia | Ocean | Forest |
|:-----:|:-----:|:------:|
| ![Sepia](screenshots/theme-sepia.png) | ![Ocean](screenshots/theme-ocean.png) | ![Forest](screenshots/theme-forest.png) |

## Configuration

Settings are stored in `~/.config/jid/jid.toml` and automatically update when you change themes or toggle focus mode.

```toml
theme = "dark"                              # dark, light, sepia, ocean, or forest
focus_mode = false                          # Dims text except current line
documents_dir = "/Users/you/Documents/jid"  # Where documents are saved
```

To change the default save location, edit `documents_dir` in the config file.
