# incredicalculator

To run the simulator:
- `cd incredicalculator_pc && cargo run`

To run the RP2350 project:
- `cd incredicalculator_rp && cargo run --release`
- (it will still work if you don't put --release, but it will run like 10x slower and have like 3x the memory usage)

To get a .uf2 file:
- edit incredicalculator-rp/.cargo/config.toml and uncomment the line that talks about outputting a uf2 file
- cd into incredicalculator_rp and run `cargo run --release`
- grab the uf2 file from target/thumbv8m.main-none-eabihf/release