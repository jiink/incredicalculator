# Patches to tsuki 0.4.8

This is a vendored copy of [tsuki](https://github.com/ultimaweapon/tsuki) 0.4.8, a
port of Lua 5.4 to Rust, used for scripted apps. It is
wired in through `[patch.crates-io]` in the workspace `Cargo.toml`. Tests,
benches and dev-dependencies were left out: a path dependency inside the
workspace becomes a workspace member, and its dev-dependencies would otherwise
get resolved into the lockfile.

Every change is marked with a `PATCH (incredicalculator)` comment.

## Builds without `std` or libc

The published crate is `#![no_std]` but still fails to build without `std`, so it
can't build for `thumbv8m.main-none-eabihf` (the RP2350) or
`wasm32-unknown-unknown`.

- `src/no_libc.rs` (new): replaces `libc::realloc`, `free`, `strcmp` and `abs`,
  and provides the `f64` math methods that only exist with `std` (`floor`,
  `powf`, `sin`, ...) through `libm`. The allocator keeps each block's size in
  an 8-byte header, so freeing doesn't rely on callers passing the right old
  size. libc `free` ignored that size, so a wrong one would never have shown up.
- `src/string/table.rs`: `std::alloc::handle_alloc_error` changed to
  `alloc::alloc::handle_alloc_error`.
- `src/builtin/table/mod.rs`, `src/builtin/mod.rs`: `table.sort` no longer needs
  the `rand` feature. It only used randomness to vary the pivot on large arrays,
  so an LCG now does that.
- `src/lib.rs`: `#![allow(warnings)]`, because the C2Rust-generated code has
  about 1,600 warnings, which Cargo hides for crates.io dependencies but not for
  path dependencies.

## Limits for untrusted scripts

- **Budget** (`src/lib.rs`, `src/vm/mod.rs`): `Lua::set_budget(Some(n))` lets
  the VM run `n` jumps, loop iterations and calls before it returns
  `BudgetExhausted`. Those opcodes bound everything else a script can do, so an
  endless loop or runaway recursion ends with an error instead of hanging.
  Straight-line code between them is not counted.
- **Stack** (`src/ldo.rs`): `LUAI_MAXSTACK` cut from 1,000,000 slots to 2,000
  (about 32 KB on 32-bit targets). Deeper recursion fails with `StackOverflow`
  instead of exhausting the heap.

## Known gaps (not patched)

tsuki doesn't implement `table.insert`, `table.remove` or `math.random`;
`incredicalculator_core`'s Lua engine registers its own versions. There is still
no memory cap: a script that allocates without bound panics on out-of-memory.

## `string.format` conversions

- `src/builtin/string/mod.rs`: `%e`, `%E`, `%g` and `%G` work as in C Lua
  (flags, width and precision included). Upstream rejects them, which left
  scripts without a way to print a float with a set number of significant
  digits.
