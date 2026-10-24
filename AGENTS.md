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

Do not use `cargo test --manifest-path crates/incredicalculator_core/Cargo.toml` as the default validation command: the core test executable may fail to link because the host test harness does not provide the `critical_section_1_0_acquire` and `critical_section_1_0_release` symbols. Treat that linker failure as an environment/test-harness limitation, not as a failure in the code change. Run core tests only when the required linker support is configured.
