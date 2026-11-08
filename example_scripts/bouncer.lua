-- Bouncer: an example scripted app.
--
-- The optional global `app` table sets how the app appears in the menu.
-- `realtime = true` runs `update` every frame instead of only after a key press.
app = { name = "Bouncer", color = 0x2a6f97, realtime = true }

-- Hooks (all optional, defined as global functions):
--   init()               runs each time the app is opened
--   update()             draws the screen; drawing only works here
--   on_key(key, action)  a key was pressed; see Keys below
--   on_key_up(key)       that key was released
--
-- Keep state in variables declared at the top of the file. It lasts while the
-- app is open and starts over each time the app is opened.
--
-- Keys: `key` is the physical key, named by its main legend: "0"-"9",
-- "backspace", "/", "*", "-", "+", "=", "shift" or "super".
-- `action` is what the key's printed legend means with Shift or Super held,
-- so apps can use the same keys as the built-in ones:
--   alone   "0"-"9", "backspace", "/", "*", "-", "+", "enter" (the = key)
--   Shift   0-5 "A"-"F", 6 "~", 7 "(", 8 ")", 9 "0x",
--           backspace "&", / "|", * "%", - "<<", + ">>", = "^"
--   Super   0 ".", 1 "end", 2 "down", 4 "left", 5 "ok", 6 "right",
--           7 "home", 8 "up", backspace "clear", / "undo",
--           * - + = "F1" "F2" "F3" "F4"
-- action is nil for Shift and Super themselves. Apps never see Super + Shift
-- (leaves the app) or Super with 9 or 3 (volume and brightness).
--
-- Typing numbers: math_input{...} makes the input box the calculators use,
-- typed with the same keys (Super+0 for ".", Super+4/6 to move the cursor,
-- Super+backspace to clear, Super+/ to undo, ...). Options, all optional:
--   x, y, w, h, mode ("expression", "integer" or "hex"), text, size,
--   bg, focus_bg, border
-- Make boxes at the top of the file or in init() (at most 8), then:
--   box:key()            in on_key: hand it the key; returns "changed",
--                        "handled", "enter", "up", "down", "left", "right",
--                        or nil if the box doesn't use the key
--   box:draw(focused)    in update
--   box:value()          the number typed or worked out, or nil
--   box:text()   box:set_text(s)   box:clear()
-- See ohmslaw.lua in the project's example_scripts folder.
--
-- Drawing (screen is 320x240, colors are 0xRRGGBB or rgb(r, g, b)):
--   clear(color)
--   line(x1, y1, x2, y2, color [, width])
--   rect(x, y, w, h, color)   rect_outline(x, y, w, h, color, width)
--   round_rect(x, y, w, h, radius, color)
--   triangle(x1, y1, x2, y2, x3, y3, color)
--   polygon({x1, y1, x2, y2, x3, y3, ...}, color)
--   text(string, x, y, size, color [, "left" | "center" | "right"])
--   text_width(string, size)
-- Input and time: is_down(key), millis().  print(...) writes to the log.
-- Typing on the computer over USB: hid_press(key), hid_release(key),
--   hid_release_all(), hid_connected(); see macropad.lua.
-- Also available: Lua's math, string and table libraries, including
-- string.format("%.3g", x) and friends.

local SIZE = 28
local COLORS = { 0xff5e5b, 0xffed66, 0x00cecb, 0xd8d8d8, 0x9b5de5 }

local x, y, vx, vy, paused, bounces, last

function init()
    x, y = 40, 60
    vx, vy = 90, 70 -- pixels per second
    paused = false
    bounces = 0
    last = millis()
end

local nudges = {
    ["8"] = function() vy = vy - 40 end,
    ["2"] = function() vy = vy + 40 end,
    ["4"] = function() vx = vx - 40 end,
    ["6"] = function() vx = vx + 40 end,
}

function on_key(key)
    if key == "5" then
        paused = not paused
    elseif key == "0" then
        init()
    elseif nudges[key] then
        nudges[key]()
    end
end

function update()
    local now = millis()
    local dt = math.min((now - last) / 1000, 0.1) -- don't jump after a pause
    last = now

    if not paused then
        x = x + vx * dt
        y = y + vy * dt
        local right, bottom = 320 - SIZE, 220 - SIZE
        if x < 0 then x = 0; vx = math.abs(vx); bounces = bounces + 1 end
        if x > right then x = right; vx = -math.abs(vx); bounces = bounces + 1 end
        if y < 20 then y = 20; vy = math.abs(vy); bounces = bounces + 1 end
        if y > bottom then y = bottom; vy = -math.abs(vy); bounces = bounces + 1 end
    end

    clear(0x101820)
    line(0, 20, 320, 20, 0x2a6f97)
    line(0, 220, 320, 220, 0x2a6f97)
    text("Bounces: " .. bounces, 6, 15, 6, 0xffffff)
    if paused then
        text("PAUSED", 314, 15, 6, 0xffed66, "right")
    end
    round_rect(x, y, SIZE, SIZE, 6, COLORS[bounces % #COLORS + 1])
    text("2 4 6 8: push   5: pause   0: reset", 160, 235, 5, 0x8899aa, "center")
end
