-- title: Color Converter
-- desc:  Convert a color between RGB, 0-1 floats, hex and RGB565.
-- color: 0x1f5c55
-- icon: VM1:AQQMAQdEZWZhdWx0AQEHAP8AAAggAEAAAABwAND/gACQ/2AAgP8gAKD/8P/g/+D/EAAAAAAA/wQIgABAAGAAcAAwAIAA8P9gAOD/IAAAAPD/QADg/3AAAAAAABH/CFAA4P8wABAAAAAgAMD/AACw/8D/0P+Q/xAAgP9AAKD/AAD7/wUAACAAMgAUAFAA4P8QAPD/8P8QAAD/APcFwP8AAAAAIAAAAPD/4P/g/7n/6v8A4f8ABQAAcAAgAEAAAAAAAOD/IADw/2AAAP///wTg/xAAAAAwACAAEAAAAPD/

-- Color Converter: type a color in any of four forms; the others update.
--
--   F1 (Super + *)   RGB 0-255, one box per channel
--   F2 (Super + -)   normalized channels 0-1 (expressions work: 1/3)
--   F3 (Super + +)   24-bit hex
--   F4 (Super + =)   RGB565 hex
--
-- Enter or Super+4/6 move between the channel boxes.

local BG      = 0x202a2d
local FIELD   = 0x1f5c55
local FOCUS   = 0xe8f27a
local BORDER  = 0x123b38
local FG      = 0xf4f1df
local SUBTLE  = 0x91d8ca

local MODES = { "rgb", "unit", "hex", "rgb565" }
local LABELS = {
    rgb = "RGB 0-255",
    unit = "NORMALIZED CHANNELS 0-1",
    hex = "HEX 0x",
    rgb565 = "RGB565 0x",
}
local F_KEYS = { F1 = "rgb", F2 = "unit", F3 = "hex", F4 = "rgb565" }

local Y = 74
local function box(x, w, mode)
    return math_input{
        x = x, y = Y, w = w, h = 28, mode = mode,
        bg = FIELD, focus_bg = FOCUS, border = BORDER, size = 8,
    }
end

local inputs = {
    rgb = { box(8, 96, "integer"), box(112, 96, "integer"), box(216, 96, "integer") },
    unit = { box(8, 96, "expression"), box(112, 96, "expression"), box(216, 96, "expression") },
    hex = { box(8, 304, "hex") },
    rgb565 = { box(8, 304, "hex") },
}

local mode, focus
local c = { 0x33, 0x66, 0x99 } -- r, g, b

local function rgb24()
    return (c[1] << 16) | (c[2] << 8) | c[3]
end

local function to_565()
    local r = (c[1] * 31 + 127) // 255
    local g = (c[2] * 63 + 127) // 255
    local b = (c[3] * 31 + 127) // 255
    return (r << 11) | (g << 5) | b
end

local function from_565(v)
    local r, g, b = (v >> 11) & 0x1f, (v >> 5) & 0x3f, v & 0x1f
    return { (r << 3) | (r >> 2), (g << 2) | (g >> 4), (b << 3) | (b >> 2) }
end

local function sync()
    for i = 1, 3 do
        inputs.rgb[i]:set_text(string.format("%d", c[i]))
        inputs.unit[i]:set_text(string.format("%.3f", c[i] / 255))
    end
    inputs.hex[1]:set_text(string.format("%06X", rgb24()))
    inputs.rgb565[1]:set_text(string.format("%04X", to_565()))
end

local function set_mode(m)
    mode, focus = m, 1
    sync()
end

local function clamp(v, lo, hi)
    return math.max(lo, math.min(hi, v))
end

-- Reads the box that was just edited into the color.
local function read_input()
    local b = inputs[mode][focus]
    local v = b:value() or 0
    if mode == "rgb" then
        local cl = clamp(math.floor(v), 0, 255)
        if cl ~= v then b:set_text(string.format("%d", cl)) end
        c[focus] = cl
    elseif mode == "unit" then
        local cl = clamp(v, 0.0, 1.0)
        if cl ~= v then b:set_text(string.format("%.3f", cl)) end
        c[focus] = math.floor(cl * 255 + 0.5)
    elseif mode == "hex" then
        v = math.floor(v) & 0xffffff
        c = { v >> 16, (v >> 8) & 0xff, v & 0xff }
    else
        c = from_565(math.min(math.floor(v), 0xffff))
    end
end

function init()
    c = { 0x33, 0x66, 0x99 }
    set_mode("rgb")
end

function on_key(key, action)
    if F_KEYS[action] then
        set_mode(F_KEYS[action])
        return
    end
    local r = inputs[mode][focus]:key()
    local channels = (mode == "rgb" or mode == "unit")
    if r == "changed" then
        read_input()
    elseif r == "enter" and channels then
        focus = focus % 3 + 1
    elseif r == "left" and channels and focus > 1 then
        focus = focus - 1
    elseif r == "right" and channels and focus < 3 then
        focus = focus + 1
    end
end

function update()
    clear(BG)
    text("Color Converter", 8, 15, 6, FG)
    text("Press F1-F4", 8, 35, 6, SUBTLE)

    local y = 60
    text(LABELS[mode], 8, y, 6, FG)
    rect(0, y + 12, 320, 32, rgb(c[1], c[2], c[3]))
    rect_outline(0, y + 12, 320, 32, FG, 2)
    local group = inputs[mode]
    for i = 1, #group do
        group[i]:draw(#group == 1 or i == focus)
    end

    local cy = 124
    text(string.format("HEX 0x%06X", rgb24()), 8, cy, 6, FG)
    text(string.format("DEC (%d, %d, %d)", c[1], c[2], c[3]), 8, cy + 20, 6, FG)
    text(string.format("FLOAT (%.3f, %.3f, %.3f)", c[1] / 255, c[2] / 255, c[3] / 255),
        8, cy + 40, 6, FG)
    text(string.format("RGB565 0x%04X", to_565()), 8, cy + 60, 6, FG)
end
