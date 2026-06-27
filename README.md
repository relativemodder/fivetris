# fivetris
This is a Rust rewrite of [four-tris](https://github.com/fiorescarlatto/four-tris), an open source training tool for block-stacking games. It is built for quickly exploring different situations, testing options, and training freely in a Tetris-like environment.

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
