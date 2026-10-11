# incredicalculator

To build the project:
- Do NOT run `cargo build` in the project's root directory. You will get misled by weird errors. Instead you must first pick whether you want to build the PC program, the RP2350 program, or the browser build.
    - To build the PC program from the project's root directory, run `cargo build --manifest-path incredicalculator_pc/Cargo.toml`
    - To build the RP program from the project's root directory, run `cargo build --release --manifest-path incredicalculator_rp/Cargo.toml --target thumbv8m.main-none-eabihf`
    - To build the browser version, run `rustup target add wasm32-unknown-unknown` once, then `cargo build --target wasm32-unknown-unknown --manifest-path incredicalculator_web/Cargo.toml`
    - Or just cd into the respective subdirectory and run cargo build

To run the simulator:
- `cd incredicalculator_pc && cargo run`

To run the RP2350 project:
- `cd incredicalculator_rp && cargo run --release`
- (it will still work if you don't put --release, but it will run like 10x slower and have like 3x the memory usage)

To run the browser build:
- Use a wasm-capable Rust toolchain and build the web crate from its directory: `cd incredicalculator_web && wasm-pack build --target web --out-dir pkg`
- The easiest way to launch it is:
  - `cd incredicalculator_web && ./start-browser.sh`
  - then open `http://127.0.0.1:8000` in a browser
- The script will build the wasm bundle if needed, then start a local static server for the page.

## Vector graphics JSON

`incredicalculator_core` converts every `*.json` file in `crates/incredicalculator_core/assets/` into static Rust data at build time. The file stem becomes an uppercase Rust constant: `red-line.json` becomes `graphics::RED_LINE`. The firmware does not parse JSON at runtime. Each action contains frames, and each frame contains polygon shapes with point coordinates, fill and border colors, border width, and an `open` flag. Color components are integers from 0 to 255; alpha defaults to 255 when omitted.

For example, after adding `assets/player-idle.json`, draw its first frame with:


To get a .uf2 file:
- edit incredicalculator-rp/.cargo/config.toml and uncomment the line that talks about outputting a uf2 file
- cd into incredicalculator_rp and run `cargo run --release`
- grab the uf2 file from target/thumbv8m.main-none-eabihf/release
