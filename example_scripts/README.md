# Example scripted apps

See [SCRIPTING.md](SCRIPTING.md) for the full scripting quick reference.

Lua scripts that run as apps on the Incredicalculator.

- **Web app:** drop a `.lua` file onto the page, or use **Add script…** in the
  ⋯ menu, then find it in the apps menu.
- **Device:** plug the calculator into a computer with a USB cable; the
  `INCREDICALC` drive appears whatever the calculator is doing. Copy `.lua`
  files onto it and they show up in the apps menu about half a second later.
- **Editing on the device:** with a scripted app open, edit its file on the
  drive and save: about half a second later the app restarts with the new
  code, or shows the error.

`bouncer.lua` is also bundled into the firmware as the default example (it is
put on the drive when the calculator first formats it), and its header comment
documents the script API: hooks, keys, input boxes and drawing.

`ohmslaw.lua` shows `math_input` boxes and the `action` argument of `on_key`:
use those rather than mapping raw keys yourself, so a script types numbers and
moves around with the same keys as the built-in apps.

Scripts must be at most 16 KiB. Every script here is loaded and run for a few
frames by `crates/incredicalculator_core/tests/example_scripts.rs`, so keep
them working when the API changes.
