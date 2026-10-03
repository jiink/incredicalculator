# incredicalculator

To build the project:
- Do NOT run `cargo build` in the project's root directory. You will get misled by weird errors. Instead you must first pick whether you want to build the PC program or the RP2350 program.
    - To build the PC program from the project's root directory, run `cargo build --manifest-path incredicalculator_pc/Cargo.toml`
    - To build the RP program from the project's root directory, run `cargo build --release --manifest-path incredicalculator_rp/Cargo.toml --target thumbv8m.main-none-eabihf`
    - Or just cd into the respective subdirectory and run cargo build

To run the simulator:
- `cd incredicalculator_pc && cargo run`

To run the RP2350 project:
- `cd incredicalculator_rp && cargo run --release`
- (it will still work if you don't put --release, but it will run like 10x slower and have like 3x the memory usage)

To get a .uf2 file:
- edit incredicalculator-rp/.cargo/config.toml and uncomment the line that talks about outputting a uf2 file
- cd into incredicalculator_rp and run `cargo run --release`
- grab the uf2 file from target/thumbv8m.main-none-eabihf/release