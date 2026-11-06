-- Bouncer: an example scripted app.
--
-- The optional global `app` table sets how the app appears in the menu.
-- `realtime = true` runs `update` every frame instead of only after a key press.
app = { name = "Bouncer", color = 0x2a6f97, realtime = true }

-- Hooks (all optional, defined as global functions):
--   init()       runs each time the app is opened
--   update()     draws the screen; drawing only works here
--   on_key(key)  key is "0"-"9", "f1"-"f6", "shift" or "super"
--
-- Keep state in variables declared at the top of the file. It lasts while the
-- app is open and starts over each time the app is opened.
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
-- Also available: Lua's math, string and table libraries.

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
