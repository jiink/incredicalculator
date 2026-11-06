# Example scripted apps

Lua scripts that run as apps on the Incredicalculator.

- **Web app:** drop a `.lua` file onto the page, or use **Add script…** in the
  ⋯ menu, then find it in the apps menu.
- **Device:** loading scripts from storage arrives in phase 3; see
  `scriptable_apps_phase3_plan.md`.

`bouncer.lua` is also bundled into the firmware as the default example, and its
header comment documents the script API: hooks, drawing functions and input.

Scripts must be at most 16 KiB. Every script here is loaded and run for a few
frames by `crates/incredicalculator_core/tests/example_scripts.rs`, so keep
them working when the API changes.
