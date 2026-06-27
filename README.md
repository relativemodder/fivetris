# fivetris
This is a Rust rewrite of [four-tris](https://github.com/fiorescarlatto/four-tris), an open source training tool for block-stacking games. It is built for quickly exploring different situations, testing options, and training freely in a Tetris-like environment.

## Creating custom Skins

You can add your own custom skins inside `~/.local/share/fivetris/skins/`.
All custom skins must follow these requirements:
- Must be a `.ini` file.
- Must define palette values for the blocks and UI colors.
- `STYLE=0` uses flat blocks, `STYLE=1` uses inset blocks.

You can follow the palette format used by the built-in skins. A skin section may define `I`, `J`, `S`, `O`, `Z`, `L`, `T`, `G`, `F`, `BKG`, `BOX`, and `TXT` colors.

Example:

```ini
[My Skin]
I=00D0FF
J=4080FF
S=40D040
O=FFE020
Z=FF4020
L=FF8020
T=A040F0
G=CCCCCC
F=2F3136
BKG=2F3136
BOX=000000
TXT=FFFFFF
STYLE=1
```

## Reporting issues, suggestions, feedback, bugs

1. Open an issue in this repository if you are not sure whether something is a bug or expected behavior.
2. Check whether it has already been reported.
3. If not, describe the problem clearly and include the steps to reproduce it.

## Building
- You will need a recent stable Rust toolchain with `cargo`.
- On Linux, make sure the ALSA development package is installed so audio can build correctly.
- Run `cargo run --release` to start the application.
- Run `cargo test` to execute the test suite.

If you want to build the app into a standalone binary you can use `cargo build --release`.

## Code

The application is written in Rust with `eframe` and `egui`, and it uses embedded assets for fonts, textures, and sounds. The code is structured around the game core, rendering, platform helpers, configuration, and app state management.

If you want to add a new feature or contribute in general, open an issue or a pull request.

## Upstream

Original project:

- <https://github.com/fiorescarlatto/four-tris>

## License

This project is licensed under the GNU General Public License v3.0 or later. See [LICENSE](./LICENSE).
