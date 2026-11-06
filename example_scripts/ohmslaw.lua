-- Ohm's Law: type any two of voltage, current and resistance; the third updates.
--
-- Keys:
--   0-9    type a digit into the selected box
--   shift  decimal point
--   f1     move to the box above      f2  move to the box below
--   f3     backspace                  f4  clear the selected box
--   f6     reset everything
--
-- The two boxes you edited most recently are the inputs; the remaining box is
-- calculated from them (V = I * R). Typing into the calculated box makes it an
-- input, and the older of the other two becomes the calculated one.

app = { name = "Ohm's Law", color = 0x2a6f97, realtime = false }

local FIELDS = {
    { key = "V", label = "Voltage", unit = "V" },
    { key = "I", label = "Current", unit = "A" },
    { key = "R", label = "Resistance", unit = "ohm" },
}
local MAX_LEN = 12

local BG       = 0x10161d
local BOX      = 0x1d2733
local BOX_CALC = 0x1a3326
local EDGE     = 0x2a6f97
local SELECT   = 0xffed66
local TEXT     = 0xffffff
local DIM      = 0x8899aa
local CALC     = 0x7ee0a1

local txt      -- txt.V / txt.I / txt.R: the text shown in each box
local recent   -- the two most recently edited field keys, newest first
local sel      -- index into FIELDS of the selected box
local fresh    -- true until the first key after moving into a box

-- Non-negative whole number to text, without string.format (this runtime
-- doesn't support most format conversions).
local function digits(n)
    n = math.floor(n + 0.5)
    if n == 0 then return "0" end
    local s = ""
    while n > 0 do
        local d = math.floor(n % 10)
        s = string.char(48 + d) .. s
        n = math.floor(n / 10)
    end
    return s
end

-- abs value rounded to `places` decimals, trailing zeros removed.
local function fixed(a, places)
    local scale = 10 ^ places
    local scaled = math.floor(a * scale + 0.5)
    local whole = math.floor(scaled / scale)
    local frac = scaled - whole * scale
    local s = digits(whole)
    if places > 0 and frac > 0 then
        local f = digits(frac)
        f = string.rep("0", places - #f) .. f
        f = f:gsub("0+$", "")
        s = s .. "." .. f
    end
    return s
end

-- Number to text with about 6 significant digits.
local function fmt(n)
    if n == nil or n ~= n or n == math.huge or n == -math.huge then
        return ""
    end
    if n == 0 then return "0" end
    local sign = n < 0 and "-" or ""
    local a = math.abs(n)
    local e = math.floor(math.log(a) / math.log(10))
    if 10 ^ (e + 1) <= a then e = e + 1 end -- fix float rounding
    if 10 ^ e > a then e = e - 1 end
    if e >= -4 and e <= 9 then
        return sign .. fixed(a, math.max(0, 5 - e))
    end
    -- scientific notation for very large or very small values
    local m = a / 10 ^ e
    local ms = fixed(m, 4)
    if ms == "10" then ms = "1"; e = e + 1 end
    return sign .. ms .. "e" .. (e < 0 and "-" or "") .. digits(math.abs(e))
end

local function value(k)
    local s = txt[k]
    if s == "" or s == "." then return nil end
    return tonumber(s)
end

local function calc_key()
    for _, f in ipairs(FIELDS) do
        if f.key ~= recent[1] and f.key ~= recent[2] then return f.key end
    end
end

local function solve()
    local c = calc_key()
    local V, I, R = value("V"), value("I"), value("R")
    local result
    if c == "V" and I and R then
        result = I * R
    elseif c == "I" and V and R and R ~= 0 then
        result = V / R
    elseif c == "R" and V and I and I ~= 0 then
        result = V / I
    end
    txt[c] = fmt(result)
end

-- Mark field k as just edited, so it becomes (or stays) an input.
local function touch(k)
    if recent[1] ~= k then
        recent[2] = recent[1]
        recent[1] = k
    end
end

function init()
    txt = { V = "12", I = "", R = "100" }
    recent = { "V", "R" }
    sel = 1
    fresh = true
    solve()
end

local function edit(fn)
    local k = FIELDS[sel].key
    if fresh then txt[k] = "" end
    fresh = false
    txt[k] = fn(txt[k])
    touch(k)
    solve()
end

local function move(d)
    sel = (sel - 1 + d) % #FIELDS + 1
    fresh = true
end

function on_key(key)
    if key:match("^%d$") then
        edit(function(s)
            if #s >= MAX_LEN then return s end
            if s == "0" then return key end -- no leading zeros
            return s .. key
        end)
    elseif key == "shift" then
        edit(function(s)
            if s:find(".", 1, true) or #s >= MAX_LEN then return s end
            if s == "" then return "0." end
            return s .. "."
        end)
    elseif key == "f3" then
        fresh = false
        edit(function(s) return s:sub(1, -2) end)
    elseif key == "f4" then
        edit(function() return "" end)
    elseif key == "f1" then
        move(-1)
    elseif key == "f2" then
        move(1)
    elseif key == "f6" then
        init()
    end
end

function update()
    clear(BG)
    line(0, 20, 320, 20, EDGE)
    line(0, 220, 320, 220, EDGE)
    text("OHM'S LAW   V = I x R", 6, 15, 6, TEXT)

    local V, I = value("V"), value("I")
    local p = (V and I) and fmt(V * I) .. " W" or "---"
    text("P = " .. p, 314, 15, 5, DIM, "right")

    local c = calc_key()
    for i, f in ipairs(FIELDS) do
        local y = 28 + (i - 1) * 64
        local is_calc = f.key == c
        round_rect(10, y, 300, 56, 6, is_calc and BOX_CALC or BOX)
        if i == sel then
            rect_outline(10, y, 300, 56, SELECT, 2)
        end

        local label = f.label .. " (" .. f.unit .. ")"
        text(label, 20, y + 16, 5, i == sel and SELECT or DIM)
        if is_calc then
            text("calculated", 300, y + 16, 4, CALC, "right")
        end

        local s = txt[f.key]
        if s == "" then s = is_calc and "---" or "" end
        if i == sel then s = s .. "_" end
        text(s, 20, y + 46, 8, is_calc and CALC or TEXT)
        text(f.key, 300, y + 46, 8, DIM, "right")
    end

    text("F1/F2 move  SHIFT .  F3 del  F4 clr  F6 reset", 160, 235, 4, DIM, "center")
end
