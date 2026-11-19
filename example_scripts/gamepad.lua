-- title: Gamepad
-- desc:  Plays games on the computer it is plugged into, as a USB gamepad.
-- color: 0x2f8f5b

-- Gamepad: makes the calculator a game controller for the computer it is
-- plugged into. Each key holds its button (or tilts the stick) for as long as
-- you hold it.
--
--   8 4 6 2   left stick up, left, right, down (or the d-pad; see 5)
--   5         switch the arrows between the left stick and the d-pad
--   0 =       A, B          1 3   X, Y
--   - +       LB, RB        7 9   LT, RT (pulled by setting the axis)
--   /  *      Back, Start   bksp  Home
--
-- The first press asks whether the app may send input: press Enter (=) to
-- allow it. Leave with Super + Shift, as with any app.
--
-- The calculator shows up as a USB gamepad whenever it is plugged in. On the
-- PC and web builds nothing is sent; each change is written to the log.

-- Calculator key -> gamepad button (names are in SCRIPTING.md).
local BUTTONS = {
    ["0"] = "a", ["="] = "b", ["1"] = "x", ["3"] = "y",
    ["-"] = "lb", ["+"] = "rb",
    ["/"] = "back", ["*"] = "start", ["backspace"] = "home",
}

-- Calculator key -> trigger axis and how far it pulls it, 0 to 1.
local TRIGGERS = { ["7"] = { "lt", 1.0 }, ["9"] = { "rt", 1.0 } }

-- Calculator key -> direction, and the stick position it gives: x and y,
-- -1 to 1, with y positive downwards as gamepads count it.
local ARROWS = {
    ["8"] = { "up", 0, -1 },
    ["2"] = { "down", 0, 1 },
    ["4"] = { "left", -1, 0 },
    ["6"] = { "right", 1, 0 },
}
-- How far the arrows tilt the stick. Diagonals are scaled so the stick
-- never leaves its circle.
local TILT = 1.0
local DIAGONAL = 0.7071

local held = {}        -- calculator keys held now
local use_dpad = false -- arrows drive the d-pad instead of the stick
local stick_x, stick_y = 0, 0

-- Moves the left stick to where the held arrows point.
local function update_stick()
    local x, y = 0, 0
    if not use_dpad then
        for key, arrow in pairs(ARROWS) do
            if held[key] then
                x = x + arrow[2]
                y = y + arrow[3]
            end
        end
    end
    local scale = (x ~= 0 and y ~= 0) and TILT * DIAGONAL or TILT
    stick_x, stick_y = x * scale, y * scale
    gamepad_axis("lx", stick_x)
    gamepad_axis("ly", stick_y)
end

function on_key(key)
    held[key] = true
    if BUTTONS[key] then
        gamepad_press(BUTTONS[key])
    elseif TRIGGERS[key] then
        gamepad_axis(TRIGGERS[key][1], TRIGGERS[key][2])
    elseif ARROWS[key] then
        if use_dpad then
            gamepad_press(ARROWS[key][1])
        else
            update_stick()
        end
    elseif key == "5" then
        -- Let go of whatever the arrows held before switching.
        for k, arrow in pairs(ARROWS) do
            if held[k] then gamepad_release(arrow[1]) end
        end
        use_dpad = not use_dpad
        update_stick()
        if use_dpad then
            for k, arrow in pairs(ARROWS) do
                if held[k] then gamepad_press(arrow[1]) end
            end
        end
    end
end

function on_key_up(key)
    held[key] = nil
    if BUTTONS[key] then
        gamepad_release(BUTTONS[key])
    elseif TRIGGERS[key] then
        gamepad_axis(TRIGGERS[key][1], 0)
    elseif ARROWS[key] then
        if use_dpad then
            gamepad_release(ARROWS[key][1])
        else
            update_stick()
        end
    end
end

-- ---- Drawing ----

local ON, OFF, LABEL = 0xffd84a, 0x2c4a3a, 0xffffff

local function circle(x, y, r, color)
    round_rect(x - r, y - r, r * 2, r * 2, r, color)
end

local function button(key, label, x, y)
    local on = held[key]
    circle(x, y, 13, on and ON or OFF)
    text(label, x, y + 4, 5, on and 0x000000 or LABEL, "center")
end

local function bar(key, label, x, y)
    local on = held[key]
    round_rect(x, y, 70, 18, 5, on and ON or OFF)
    text(label, x + 35, y + 13, 4, on and 0x000000 or LABEL, "center")
end

function update()
    clear(0x10241a)
    if hid_connected() then
        text("Gamepad", 160, 18, 6, 0xffffff, "center")
    else
        text("Plug into a computer to play", 160, 18, 4, 0xffb060, "center")
    end

    -- Shoulders: triggers on top, bumpers below.
    bar("7", "LT  7", 12, 32)
    bar("9", "RT  9", 238, 32)
    bar("-", "LB  -", 12, 54)
    bar("+", "RB  +", 238, 54)

    -- Left: the stick, or the d-pad.
    local cx, cy = 70, 140
    if use_dpad then
        for key, arrow in pairs(ARROWS) do
            local x, y = cx + arrow[2] * 26, cy + arrow[3] * 26
            round_rect(x - 13, y - 13, 26, 26, 4, held[key] and ON or OFF)
            text(key, x, y + 4, 5, held[key] and 0x000000 or LABEL, "center")
        end
        text("D-pad  (5: stick)", cx, 210, 4, 0xa0c0b0, "center")
    else
        circle(cx, cy, 42, OFF)
        circle(cx + stick_x * 28, cy + stick_y * 28, 14, (stick_x ~= 0 or stick_y ~= 0) and ON or 0x6a8a7a)
        text("Stick 8/4/6/2  (5: d-pad)", cx, 210, 4, 0xa0c0b0, "center")
    end

    -- Middle: back, home, start.
    button("/", "Bk", 135, 100)
    button("backspace", "H", 160, 130)
    button("*", "St", 185, 100)

    -- Right: face buttons, in their usual diamond.
    local fx, fy = 255, 140
    button("3", "Y", fx, fy - 30)
    button("1", "X", fx - 30, fy)
    button("=", "B", fx + 30, fy)
    button("0", "A", fx, fy + 30)
    text("A 0  B =  X 1  Y 3", fx, 210, 4, 0xa0c0b0, "center")
end
