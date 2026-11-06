my vision for scriptable apps from user perspective:
1. the user should be able to plug in the incredicalculator_rp device into their PC via USB and see it come up as a storage device.
    - (if the user needs to do something to initiate this USB mode, that is acceptable)
1. using their PC, the user creates a file on the storage device like my-app.lua
1. the user edits my-app.lua to create a simple application, saves it, and closes their editor
1. the user opens the app menu on the calculator and is able to locate and run the new app they have created
1. the user power-cycles the calculator
1. the user sees that their scripted app is still there
1. the user is able to go back on and delete the script file, thus deleting and removing the app from the app menu.

---

- if this needs to be broken down into smaller steps, that is acceptable. For instance, perhaps there could at first be one simple example scripted app that has a hard-coded script.
- there must be a way to try out this feature using incredicalculator_web, but the details there are more flexible. perhaps it may involve drag-and-dropping a script file to add one, and to remove one perhaps there is a button near the "clear settings" button in the web UI
- the scripting engine should be modular because in order to choose between Lua and Rhai we need to be able to try out both, swapping between the two at compile time before settling on one, or having different branches or something.
- mind the flash and ram usage on the rp2350. see size-report.sh
- keep the barrier between incredicalculator_core and platform-specific stuff, this barrier is defined in platform.rs
- scripted apps should co-exist with compiled apps. it is acceptable if they are in a different list separate from the compiled apps, but if either work, pick the one with better UX.
