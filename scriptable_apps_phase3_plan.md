# Scriptable apps, phase 3: scripts on the RP2350 over USB

Phase 3 delivers the part of `scriptable_apps_vision.md` that runs on the
device:

1. Plug the calculator into a PC and it shows up as a USB drive. Opening a
   "USB Drive" app is the step that starts this.
2. Create or edit `my-app.lua` on the drive.
3. Leave USB mode; the app is in the apps menu and runs.
4. Power-cycle; the app is still there.
5. Delete the file; the app is gone from the menu.

## Already in place (phases 1 and 2)

- Lua engine (vendored tsuki in `crates/tsuki-patched`) and `ScriptApp`. An
  app's engine is only loaded while the app is open.
- `IcPlatform` script storage in `crates/incredicalculator_core/src/platform.rs`:
  `scripts_version()`, `list_scripts()`, `read_script()`, plus
  `SCRIPT_EXTENSION` and `MAX_SCRIPT_BYTES` (16 KiB).
- The shell (`sync_scripts` in `shell.rs`) reloads scripted apps whenever
  `scripts_version()` changes:
  - It keeps unchanged apps as they are.
  - An open app whose script changed restarts with the new code.
  - An open app whose script was deleted closes to the menu.
- `list_scripts()` returning `None` makes the shell show the bundled
  `bouncer.lua`. That is what the RP does today. Phase 3 makes it return the
  real list.
- The web app (`incredicalculator_web/src/lib.rs`) is a reference
  implementation of the same three methods, on localStorage.

Phase 3 is RP-only. Core should not need changes beyond small additions such
as a compiled "USB Drive" app.

## Decisions already made

- **Flash layout:** assume 2 MiB of flash, as `memory.x` does. Reserve 512 KiB
  at the end for a FAT filesystem, just below the 16 KiB settings region
  (`SETTINGS_SECTORS` in `incredicalculator_rp/src/settings_storage.rs`).
  - Shrink `FLASH` in `incredicalculator_rp/memory.x` from 2032K to 1520K so
    firmware can never grow into the filesystem.
  - Firmware is about 490 KiB today, so there is plenty of room.
  - If the board's flash chip turns out to be bigger, put the filesystem above
    2 MiB instead and leave `memory.x` alone.
- **USB mode:** a compiled "USB Drive" app in the apps menu.
  - While it is open, the drive is exposed over USB mass storage and the
    calculator does not read the filesystem itself.
  - Leaving the app bumps `scripts_version`, so the shell rescans.
- **First boot:** if the region holds no valid FAT filesystem, format it and
  write `bouncer.lua` into it (from `incredicalculator_core::script::EXAMPLE_SCRIPT`).
- **Limits:** scripts bigger than `MAX_SCRIPT_BYTES` are skipped (or shown as
  an error app). Only `*.lua` files in the root directory count.

## Work items

### 1. Flash block device

- Wrap the 512 KiB region as a block device with 512-byte sectors, the size
  FAT and USB mass storage both use. Flash erases 4 KiB at a time, so writes
  need a 4 KiB read-modify-write buffer.
  - Cache one dirty 4 KiB erase block and flush it when a different block is
    touched, and on leaving USB mode. That avoids an erase for every 512-byte
    sector a host writes.
- Share the one `embassy_rp::flash::Flash` instance with `SettingsStorage`.
  Today `SettingsStorage::new` takes ownership of it, so it needs a shared
  wrapper or a mutex.
- embassy-rp's flash operations already pause core 1 and run from RAM.
  Interrupts are off during an erase (up to tens of ms), so audio will glitch
  while writing. That is acceptable in USB mode.
- Host-test the block device logic against an in-memory "flash" with 4 KiB
  erase semantics (erase sets 0xFF, programming only clears bits).

### 2. FAT filesystem

- 512 KiB at 512-byte clusters is about 1,000 clusters, which makes it
  **FAT12**. Pick a `no_std` FAT crate that supports FAT12 and can format a
  volume. Check `fatfs` (rust-fatfs, `alloc` feature) first.
  `embedded-sdmmc` only does FAT16/32, so it would need a bigger volume.
- Check its RAM use. The heap is 96 KB and the core 0 stack has about 86 KB,
  shared with the Lua engine while a script runs. The filesystem only needs to
  be open while listing or reading scripts, or while USB mode is on.
- Format on first boot, then write `bouncer.lua`.
- Host-test against the in-memory flash: format, write, list, read, delete,
  and remount.

### 3. RP `IcPlatform` script methods

- `list_scripts()`: root directory `*.lua` names, or `None` if the filesystem
  can't be mounted, which falls back to the bundled example.
- `read_script()`: read up to `MAX_SCRIPT_BYTES` and require UTF-8.
- `scripts_version()`: a counter bumped when USB mode ends and after the
  first-boot format.

### 4. USB mass storage

- embassy-usb 0.5 has no mass-storage class (it ships CDC-ACM, CDC-NCM, HID,
  MIDI, UAC1, WebUSB and CMSIS-DAP). Options:
  - Find a maintained embassy-compatible MSC crate.
  - Write the USB Bulk-Only Transport plus the small SCSI subset hosts use:
    INQUIRY, TEST UNIT READY, READ CAPACITY(10), READ(10), WRITE(10),
    REQUEST SENSE, MODE SENSE(6), PREVENT ALLOW MEDIUM REMOVAL, START STOP UNIT.
    That is a few hundred lines. It is the likely route.
- USB clocks are switched off at boot today (see `clk_usb` / `PLL_USB` in
  `incredicalculator_rp/src/main.rs`). USB mode must start the USB PLL and the
  controller, and stop them again on exit to keep idle power low.
- Hold `power::keep_awake()` for the whole of USB mode. The system clock
  divider must stay at 1 while USB and flash are in use.

### 5. "USB Drive" app and mode switching

- A compiled app in `incredicalculator_core` that shows instructions such as
  "Connected — eject on your computer before leaving". It asks the platform to
  enter and leave USB mode.
- This needs a small `IcPlatform` addition, for example
  `set_usb_drive_mode(bool)` and `usb_drive_status() -> Option<...>`, with
  no-op defaults so the web and PC builds are unaffected.
- On leaving:
  1. Flush the cached flash block.
  2. Stop USB.
  3. Remount the filesystem.
  4. Bump `scripts_version`.
- Warn about data loss when the host hasn't ejected. Track
  START STOP UNIT (eject) and PREVENT ALLOW MEDIUM REMOVAL to know whether it
  has. Hosts cache writes, so leaving before ejecting can lose them.

### 6. Measurement

- `incredicalculator_rp/size-report.sh` for flash and static RAM, before and
  after.
- Turn on "Show memory" and check heap and stack peaks in USB mode, and while
  running a script after leaving it.
- Use the memory regression test in
  `crates/incredicalculator_core/tests/script_memory.rs` as the model for a
  filesystem memory test.

## Risks and open questions

- **Flash size:** the board's real flash chip size. Check it before
  fixing the layout.
- **Corruption:** a host writing while the calculator loses power, or leaving
  USB mode without ejecting. FAT plus the dirty-block cache can corrupt.
  Mitigations: flush often, warn on leaving without an eject, and fall back to
  reformatting (with a confirmation) if mounting fails.
- **Flash wear:** hosts write metadata (access times, `.Trashes`,
  `System Volume Information`) often. At 512 KiB that is fine for hobby use,
  but avoid needless rewrites, for example skip programming a block whose
  contents are unchanged.
- **USB clocks:** stopping and restarting the USB PLL at runtime has not been
  done in this firmware yet.
- **Atomics:** some RP2350 atomic retry loops (LDREX/STREX) livelocked in a
  hot path (see `TrackingHeap::alloc` in `incredicalculator_rp/src/memory.rs`).
  Use critical sections, not atomic read-modify-write, in any new hot paths
  such as per-sector counters.

## Test checklist (the vision's steps)

1. Fresh flash: boots, the filesystem gets formatted, and Bouncer appears in
   the menu.
2. Open "USB Drive": the PC mounts a drive containing `bouncer.lua`.
3. Create `my-app.lua`, save, eject, and leave the app: "my-app" is in the
   menu and runs.
4. Edit `my-app.lua` the same way: the app runs the new code.
5. Power-cycle: both apps are still there.
6. Delete `my-app.lua` over USB: it's gone from the menu after leaving the app.
7. Oversized or invalid scripts show an error app; they never crash the
   calculator.
8. Reflashing firmware (picotool and probe-rs) leaves the filesystem intact.
