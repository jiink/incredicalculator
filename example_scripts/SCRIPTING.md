# Incredicalculator scripting quick reference

A scripted app is one Lua 5.4 file (`.lua`, at most **16 KiB**). Each file in
the root of the script drive becomes an app in the apps menu. For working
examples, see `bouncer.lua` (animation, raw keys) and `ohmslaw.lua` (number
entry with `math_input`).

## Getting scripts onto the calculator

| Where | How |
|---|---|
| **Calculator** | Plug it into a computer with a USB cable. The `INCREDICALC` drive appears whatever the calculator is doing, even with the screen asleep. Copy `.lua` files to the root of the drive. Subfolders, hidden files and dot files such as macOS `._name` files are ignored. Unplugged, USB is off entirely. |
| **Live editing** | About half a second after the computer saves, the scripts reload: new files show up in the menu, and an open scripted app restarts with the new code or shows the error. |
| **Web app** | Drop `.lua` files onto the page, or use **Add script…** in the ⋯ menu. |
| **PC app** | Only runs the bundled `bouncer.lua`. |

If the computer says the drive needs formatting, the default FAT format works.

## Skeleton

```lua
app = { name = "My App", color = 0x2a6f97, realtime = false }  -- all optional

local count                      -- state lives in top-level variables

function init()                  -- each time the app is opened
    count = 0
end

function on_key(key, action)     -- a key was pressed
    if action == "enter" then count = count + 1 end
end

function update()                -- draw the whole screen
    clear(0x101820)
    text("Count: " .. count, 160, 120, 8, 0xffffff, "center")
end
```

- **`app` table:** `name` defaults to the file name without `.lua`. `color` is the menu card color; without it, a color is picked from the name. `realtime = true` runs `update` every frame (about 60 per second). Otherwise `update` runs only after input.
- **Hooks:** all are optional global functions. Missing hooks do nothing.
- **Lifetime:** the file runs from the top every time the app is opened, then `init()` runs. Nothing is kept after you leave the app, and nothing persists across power cycles.
- **Drawing:** only works inside `update()`. Draw the whole screen every time, starting with `clear()`.
- **Errors:** an error stops the script and shows the message with its line number. Reopen the app, or save the file again while editing live, to retry.

## Keys

`on_key(key, action)` runs once per press. There is no auto-repeat; use `is_down(key)` with `realtime = true` for held keys.

- **`key`** is the physical key, named by its main legend: `"0"`–`"9"`, `"backspace"`, `"/"`, `"*"`, `"-"`, `"+"`, `"="`, `"shift"` or `"super"`.
- **`action`** is what the key's printed legend means with the modifier held. Prefer it, so your app uses the same keys as the built-in ones.
  - It is `nil` for Shift and Super themselves.
  - F1–F4 are the Super legends of * − + =, so `action == "F1"` means Super + *.

| `key` | alone | Shift | Super |
|---|---|---|---|
| `"0"` | `"0"` | `"A"` | `"."` |
| `"1"` | `"1"` | `"B"` | `"end"` |
| `"2"` | `"2"` | `"C"` | `"down"` |
| `"3"` | `"3"` | `"D"` | *(brightness)* |
| `"4"` | `"4"` | `"E"` | `"left"` |
| `"5"` | `"5"` | `"F"` | `"ok"` |
| `"6"` | `"6"` | `"~"` | `"right"` |
| `"7"` | `"7"` | `"("` | `"home"` |
| `"8"` | `"8"` | `")"` | `"up"` |
| `"9"` | `"9"` | `"0x"` | *(volume)* |
| `"backspace"` | `"backspace"` | `"&"` | `"clear"` |
| `"/"` | `"/"` | `"\|"` | `"undo"` |
| `"*"` | `"*"` | `"%"` | `"F1"` |
| `"-"` | `"-"` | `"<<"` | `"F2"` |
| `"+"` | `"+"` | `">>"` | `"F3"` |
| `"="` | `"enter"` | `"^"` | `"F4"` |

- **Reserved:** your app never sees Super + Shift, which returns to the menu, or Super with 9 or 3, which adjust volume and brightness. `is_down` also reports those keys as up until they are released.
- **Modifier modes:** Shift and Super follow the user's key mode (Normal, Sticky or Toggle). `action` and `is_down("shift")` already account for it.

## Number and expression entry: `math_input`

This is the same input box the built-in apps use, so don't write your own text entry.

```lua
local box = math_input{ x = 10, y = 40, w = 300, h = 32 }  -- top level or init()

function on_key(key, action)
    local r = box:key()            -- hand it the key being pressed
    if r == "enter" then result = box:value() end
end

function update()
    clear(0)
    box:draw(true)                 -- true = focused (highlighted, with cursor)
end
```

**Options** (all optional):

| Option | Default | Meaning |
|---|---|---|
| `x`, `y`, `w`, `h` | 10, 40, 300, 40 | position and size |
| `mode` | `"expression"` | `"expression"`, `"integer"` or `"hex"` |
| `text` | empty | starting text |
| `size` | `h * 0.4` | largest text size; long text shrinks to fit |
| `bg`, `focus_bg` | `0x1d2733`, `0xffffff` | fill when unfocused and focused. Text is white unfocused and black focused, so keep `focus_bg` light. |
| `border` | none | outline color |

**Methods** (call them with a colon):

| Method | Does |
|---|---|
| `box:key()` | Only inside `on_key`. Returns `"changed"`, `"handled"`, `"enter"`, `"up"`, `"down"`, `"left"`, `"right"`, or `nil` if the box doesn't use the key. Moving the cursor past either end, or pressing Super+8 or Super+2, returns a direction so you can move focus. |
| `box:draw(focused)` | Only inside `update`. |
| `box:value()` | The number, or `nil` if the box is empty or invalid. Expressions are evaluated, so `3.3/2` gives `1.65`. Full double precision. |
| `box:text()`, `box:set_text(s)`, `box:clear()` | `set_text` returns `false`, and leaves the box unchanged, if `s` doesn't fit or has characters the mode can't type. |

**Keys in expression mode:**
- **Typing:** digits; `/ * - +` from the right column; `.` with Super+0 or Shift+6; parentheses with Shift+7 and Shift+8; `^` with Shift+=.
- **Editing:** ⌫ deletes; Super+⌫ clears; Super+/ undoes.
- **Moving:** Super+4 and Super+6 move the cursor; Super+7 and Super+1 jump to the start and end.

**Limits:**
- At most **8** boxes per script, each holding up to **40** characters.
- Expression mode can't hold `e` notation. Write `1.5*10^-5` instead.

## Drawing

The screen is **320 × 240** with (0, 0) at the top left. Colors are integers `0xRRGGBB`, or `rgb(r, g, b)` with each part 0–255. Coordinates may be fractional; rectangles round down to whole pixels.

| Function | Notes |
|---|---|
| `clear(color)` | fill the screen |
| `line(x1, y1, x2, y2, color [, width])` | width defaults to 2 |
| `rect(x, y, w, h, color)` | filled |
| `rect_outline(x, y, w, h, color, width)` | |
| `round_rect(x, y, w, h, radius, color)` | filled |
| `triangle(x1, y1, x2, y2, x3, y3, color)` | filled |
| `polygon({x1, y1, x2, y2, x3, y3, ...}, color)` | filled; needs at least 3 points |
| `text(s, x, y, size, color [, align])` | align: `"left"` (default), `"center"` or `"right"` |
| `text_width(s, size)` | width in pixels |

**Text:**
- **Font:** a stroke font, ASCII only. Characters outside ASCII draw as placeholders.
- **Size:** capitals are about **2.1 × size** pixels tall, and characters average about **2 × size** pixels wide.
- **Position:** capitals span from `y - 1.2*size` to `y + 0.9*size`. Use `y + 0.9*size` as the baseline.
- **Typical sizes:** 4–5 for small print, 6 for normal text, 8–10 for headings.
- **Line breaks:** `"\n"` starts a new line `2.5 × size` pixels lower.

**Per-frame limit:** at most **512** drawing calls per `update`. Going over stops the script with an error.

## Other functions

| Function | Returns |
|---|---|
| `millis()` | milliseconds since the calculator started (integer) |
| `is_down(key)` | whether a key (named like `on_key`'s `key`) is held now |
| `print(...)` | writes to the log: browser console on web, terminal on PC. Discarded on the calculator. At most 8 lines per hook call are kept. |

## Lua environment

- **base:** `assert error pcall type tostring tonumber pairs ipairs next select rawget rawset rawequal rawlen setmetatable getmetatable load`
- **math:** `abs ceil floor fmod modf max min sqrt exp log sin cos tan asin acos atan deg rad tointeger type ult pi huge maxinteger mininteger random`
  - `math.random()` gives a float in [0, 1); `math.random(m)` gives 1…m; `math.random(m, n)` gives m…n.
- **string:** `byte char find format gsub len lower match rep reverse sub upper pack packsize unpack`, also callable as methods (`s:upper()`).
  - `string.format` supports `%d %i %u %c %x %X %o %f %e %E %g %G %s %q %%`, with flags, width and precision. `%a` is not supported.
- **table:** `concat insert remove sort unpack`
- **Missing:** `string.gmatch`, `table.move`, `utf8`, `os`, `io`, `coroutine` and `require`.
  - Instead of `gmatch`, loop with `string.find` or `string.match`.
  - There is no file access and no persistent storage.
- **Numbers:** integers and floats are separate, as in Lua 5.4. `7 // 2 == 3`, `7 / 2 == 3.5`.

## Limits

| Limit | Value |
|---|---|
| Script file size | 16 KiB |
| Work per hook call | about 100,000 jumps, loop iterations and calls. An endless loop stops with an error instead of freezing. |
| Recursion depth | about 2,000 stack slots |
| Drawing calls per `update` | 512 |
| `math_input` boxes | 8, of 40 characters each |
| Log lines per hook call | 8 |

**Memory:** the calculator has about 96 KB of heap shared with everything else, so keep tables small. Avoid building big strings every frame in realtime apps.
