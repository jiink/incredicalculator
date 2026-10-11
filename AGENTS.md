Do not run `cargo build` from the workspace root. For the PC app, use
`cargo build --manifest-path incredicalculator_pc/Cargo.toml`.
For the RP app, use `cargo build --release --manifest-path incredicalculator_rp/Cargo.toml --target thumbv8m.main-none-eabihf`.
A root-level build may include embedded targets and produce irrelevant host-target errors.
Rust formatting: Don’t run cargo fmt --all just to validate a localized change. Check only the touched Rust files with rustfmt --check --config skip_children=true <files>. Don’t reformat unrelated files; if a broad check reports pre-existing differences, report that and use a scoped check instead.
