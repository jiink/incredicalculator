Do not commit or push unless explicitly instructed to.
Do not run `cargo build` from the workspace root. For the PC app, use
`cargo build --manifest-path incredicalculator_pc/Cargo.toml`.
For the RP app, use `cargo build --release --manifest-path incredicalculator_rp/Cargo.toml --target thumbv8m.main-none-eabihf`.
For the web app, first install the `wasm32-unknown-unknown` target with
`rustup target add wasm32-unknown-unknown`, then build it with
`cargo build --target wasm32-unknown-unknown --manifest-path incredicalculator_web/Cargo.toml` from this project's root directory.
A root-level build may include embedded targets and produce irrelevant host-target errors.
Rust formatting: Don’t run cargo fmt --all just to validate a localized change. Check only the touched Rust files with rustfmt --check --config skip_children=true <files>. Don’t reformat unrelated files; if a broad check reports pre-existing differences, report that and use a scoped check instead.

## Validation

For changes in `crates/incredicalculator_core`, validate compilation with:

`cargo build --manifest-path incredicalculator_pc/Cargo.toml`

Run the host tests with:

`cargo test -p incredicalculator_pc -p incredicalculator_core -p bitwise_expr`

Include `-p incredicalculator_pc` even though it has no tests of its own: it enables the `std` feature of `critical-section`, which the core tests need to link. Testing `incredicalculator_core` alone fails to link with missing `critical_section_1_0_acquire`/`critical_section_1_0_release` symbols. Plain `cargo test` inside `incredicalculator_pc` only tests that package and runs 0 tests. Do not run tests for `incredicalculator_web` or `incredicalculator_rp` on the host; they target other architectures.
