-- Ohm's Law: type any two of voltage, current and resistance; the third updates.
--
-- Each value is a math_input box, so it types like the calculator: digits,
-- Super+0 for ".", the operator keys, Shift+7/8 for parentheses, Super+9 to
-- clear and Super+3 to undo. A box can hold an expression such as 3.3/2.
-- Super+8 and Super+2 (or =) move between boxes. F4 (Super+-) resets.
--
-- The two boxes you edited most recently are the inputs; the remaining box is
-- calculated from them (V = I * R). Typing into the calculated box makes it an
-- input, and the older of the other two becomes the calculated one.

app = { name = "Ohm's Law", color = 0x2a6f97 }

local FIELDS = {
    { key = "V", label = "Voltage", unit = "V" },
    { key = "I", label = "Current", unit = "A" },
    { key = "R", label = "Resistance", unit = "ohm" },
}

local BG     = 0x10161d
local EDGE   = 0x2a6f97
local SELECT = 0xffed66
local TEXT   = 0xffffff
local DIM    = 0x8899aa
local CALC   = 0x7ee0a1

local boxes = {}
for i, f in ipairs(FIELDS) do
    boxes[f.key] = math_input{
        x = 10, y = 44 + (i - 1) * 62, w = 300, h = 32,
        bg = 0x1d2733, focus_bg = 0xffffff,
    }
end

local recent -- the two most recently edited field keys, newest first
local sel    -- index into FIELDS of the selected box

-- About 6 significant digits, as an expression the box can hold
-- (1.5e-05 becomes 1.5*10^-5).
local function fmt(n)
    if n == nil or n ~= n or n == math.huge or n == -math.huge then
        return ""
    end
    local s = string.format("%.6g", n)
    local mantissa, exp = s:match("^(.-)e([-+]%d+)$")
    if mantissa then
        s = mantissa .. "*10^" .. tonumber(exp)
    end
    return s
end

local function calc_key()
    for _, f in ipairs(FIELDS) do
        if f.key ~= recent[1] and f.key ~= recent[2] then return f.key end
    end
end

local function solve()
    local c = calc_key()
    local V, I, R = boxes.V:value(), boxes.I:value(), boxes.R:value()
    local result
    if c == "V" and I and R then
        result = I * R
    elseif c == "I" and V and R and R ~= 0 then
        result = V / R
    elseif c == "R" and V and I and I ~= 0 then
        result = V / I
    end
    boxes[c]:set_text(fmt(result))
end

-- Mark field k as just edited, so it becomes (or stays) an input.
local function touch(k)
    if recent[1] ~= k then
        recent[2] = recent[1]
        recent[1] = k
    end
end

function init()
    boxes.V:set_text("12")
    boxes.I:clear()
    boxes.R:set_text("100")
    recent = { "V", "R" }
    sel = 1
    solve()
end

local function move(d)
    sel = (sel - 1 + d) % #FIELDS + 1
end

function on_key(key, action)
    if action == "F4" then
        init()
        return
    end
    local k = FIELDS[sel].key
    local result = boxes[k]:key()
    if result == "changed" then
        touch(k)
        solve()
    elseif result == "up" then
        move(-1)
    elseif result == "down" or result == "enter" then
        move(1)
    end
end

function update()
    clear(BG)
    line(0, 20, 320, 20, EDGE)
    line(0, 220, 320, 220, EDGE)
    text("OHM'S LAW   V = I x R", 6, 15, 6, TEXT)

    local V, I = boxes.V:value(), boxes.I:value()
    local p = (V and I) and fmt(V * I) .. " W" or "---"
    text("P = " .. p, 314, 15, 5, DIM, "right")

    local c = calc_key()
    for i, f in ipairs(FIELDS) do
        local y = 38 + (i - 1) * 62
        local label = f.label .. " (" .. f.unit .. ")"
        text(label, 10, y, 5, i == sel and SELECT or DIM)
        if f.key == c then
            text("calculated", 310, y, 4, CALC, "right")
        end
        boxes[f.key]:draw(i == sel)
    end

    text("Super+8/2 move   Super+9 clear   F4 reset", 160, 235, 4, DIM, "center")
end
