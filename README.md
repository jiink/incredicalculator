# incredicalculator

To build the project:
- Do NOT run `cargo build` in the project's root directory. You will get misled by weird errors. Instead you must first pick whether you want to build the PC program, the RP2350 program, or the browser build.
    - To build the PC program from the project's root directory, run `cargo build --manifest-path incredicalculator_pc/Cargo.toml`
    - To build the RP program from the project's root directory, run `cargo build --release --manifest-path incredicalculator_rp/Cargo.toml --target thumbv8m.main-none-eabihf`
    - To build the browser version, run `rustup target add wasm32-unknown-unknown` once, then `cargo build --target wasm32-unknown-unknown --manifest-path incredicalculator_web/Cargo.toml`
    - Or just cd into the respective subdirectory and run cargo build

Press and release Super alone to open or close the apps menu. In the apps menu, Func1 (Backspace) returns to the previous app; in settings, it returns to the apps menu. While an app is active, Backspace is handled only by that app.

In the scientific calculator, press and release Shift alone to insert a decimal point (`.`). Pressing another key during the tap cancels this shortcut; existing Shift combinations remain unchanged. The programmer calculator is unaffected.

To run the simulator:
- `cd incredicalculator_pc && cargo run`

To run the RP2350 project:
- `cd incredicalculator_rp && cargo run --release`
- (it will still work if you don't put --release, but it will run like 10x slower and have like 3x the memory usage)

To enter ROM BOOTSEL mode, hold any calculator key while powering on or resetting the RP2350. The firmware samples the key matrix once during startup, displays “Entering BOOTSEL mode...” for about a second, then reboots into BOOTSEL. Pressing Super+Shift during normal operation no longer triggers BOOTSEL.

The RP2350 firmware disables the unused USB controller, USB clock, ADC clock, and USB PLL during initialization. USB remains available in ROM BOOTSEL mode; normal calculator operation does not provide USB functionality. RTT logging is unaffected.

When the RP2350 screen has timed out and there is no active audio or real-time app update, the firmware divides the 150 MHz system clock by 12 (12.5 MHz). Cross-core activity guards restore 150 MHz for UI work, key scans, LCD transfers/transitions, battery I2C reads, and audio DMA through the final FIFO drain. SPI keeps its independent 150 MHz peripheral clock, and the reference-clock-based timers continue at their normal rate. This is idle clock scaling, not dormant/deep sleep: PLL_SYS stays on for immediate wakeup.

The RP2350 audio amplifier starts in shutdown (GPIO 27 low), is enabled when an audio buffer is ready, and returns to shutdown when no further buffer arrives for 1 ms after the last DMA transfer. This grace period lets the remaining I2S FIFO samples drain without toggling the amplifier between normally queued buffers. Audio resumes automatically when playback restarts, independently of screen brightness.

On the RP2350, the screen timeout setting dims both backlights to half the configured brightness after half the timeout, then turns them off and puts the ST7789 panel into sleep mode at the full timeout. Pixel transfers are skipped while the panel sleeps; audio and key handling continue. Any key press restores the configured brightness, wakes the panel, and restarts the timer. Panel sleep/wake transitions each require the driver's 120 ms settling delay, so the image may take a moment to return. A timeout of `0` (Never) disables idle dimming and panel sleep; setting brightness to zero alone does not put the panel to sleep.

To run the browser build:
- Use a wasm-capable Rust toolchain and build the web crate from its directory: `cd incredicalculator_web && wasm-pack build --target web --out-dir pkg`
- With the `wasm32-unknown-unknown` target installed, `cd incredicalculator_web && cargo build` also builds for WebAssembly; the web directory's Cargo config selects that target by default.
- On Windows with this project's Rust 1.89 toolchain, install the compatible wasm-pack version with `cargo install --locked --version 0.13.1 wasm-pack` if it is not already available. (`wasm-pack` 0.15.0 requires Rust 1.91 even with `--locked`.) Make sure Python 3 is on `PATH`.
- The easiest way to launch it is:
  - `cd incredicalculator_web && ./start-browser.sh`
  - On Windows, run `incredicalculator_web\start-browser.bat` from the project root.
  - then open `http://127.0.0.1:8000` in a browser
  - if that port is busy, the launcher will kill the stale process and retry automatically; you can override it with `PORT=4173 ./start-browser.sh`
  - On Windows, set a different port with `set PORT=4173` before running `incredicalculator_web\start-browser.bat`.
- The script will build the wasm bundle if needed, then start a local static server for the page.

## Vector graphics JSON

`incredicalculator_core` converts every `*.json` file in `crates/incredicalculator_core/assets/` into static Rust data at build time. The file stem becomes an uppercase Rust constant: `red-line.json` becomes `graphics::RED_LINE`. The firmware does not parse JSON at runtime. Each action contains frames, and each frame contains polygon shapes with point coordinates, fill and border colors, border width, and an `open` flag. Color components are integers from 0 to 255; alpha defaults to 255 when omitted.

For example, after adding `assets/player-idle.json`, draw its first frame with:


To get a .uf2 file:
- edit incredicalculator-rp/.cargo/config.toml and uncomment the line that talks about outputting a uf2 file
- cd into incredicalculator_rp and run `cargo run --release`
- grab the uf2 file from target/thumbv8m.main-none-eabihf/release
