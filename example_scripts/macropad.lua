-- Macropad: makes the calculator's keys extra keys for the computer it is
-- plugged into. Each key holds its computer keys down for as long as you
-- hold it, so shortcuts and key repeat work like on a real keyboard.
--
-- The calculator shows up as a USB keyboard whenever it is plugged in. On the
-- PC and web builds nothing is typed; each change is written to the log.
app = { name = "Macropad", color = 0x6a4fc4 }

-- Calculator key (as on_key names it) -> label and computer keys. Several
-- keys are pressed in order and let up in reverse, like a person would.
-- Edit this table to make your own pad. Key names are in SCRIPTING.md.
local PAD = {
    ["7"] = { "Copy", "ctrl", "c" },
    ["8"] = { "Paste", "ctrl", "v" },
    ["9"] = { "Undo", "ctrl", "z" },
    ["4"] = { "Tab+", "ctrl", "t" },
    ["5"] = { "Tab-", "ctrl", "w" },
    ["6"] = { "Reopen", "ctrl", "shift", "t" },
    ["1"] = { "Prev", "prev_track" },
    ["2"] = { "Play", "play_pause" },
    ["3"] = { "Next", "next_track" },
    ["0"] = { "Mute", "mute" },
    ["backspace"] = { "Esc", "escape" },
    ["/"] = { "F13", "f13" },
    ["*"] = { "F14", "f14" },
    ["-"] = { "Vol-", "volume_down" },
    ["+"] = { "Vol+", "volume_up" },
    ["="] = { "Enter", "enter" },
}

-- Where each key sits on the keypad: column, row.
local LAYOUT = {
    ["backspace"] = { 4, 1 }, ["/"] = { 4, 2 },
    ["7"] = { 1, 3 }, ["8"] = { 2, 3 }, ["9"] = { 3, 3 }, ["*"] = { 4, 3 },
    ["4"] = { 1, 4 }, ["5"] = { 2, 4 }, ["6"] = { 3, 4 }, ["-"] = { 4, 4 },
    ["1"] = { 1, 5 }, ["2"] = { 2, 5 }, ["3"] = { 3, 5 }, ["+"] = { 4, 5 },
    ["0"] = { 1, 6 }, ["="] = { 4, 6 },
}

local held = {} -- calculator keys currently held

function on_key(key)
    local entry = PAD[key]
    if not entry then return end
    held[key] = true
    for i = 2, #entry do
        hid_press(entry[i])
    end
end

function on_key_up(key)
    local entry = PAD[key]
    if not entry then return end
    held[key] = nil
    for i = #entry, 2, -1 do
        hid_release(entry[i])
    end
end

local W, H, GAP = 72, 33, 4

function update()
    clear(0x161226)
    if hid_connected() then
        text("Macropad", 8, 6, 5, 0xffffff)
    else
        text("Plug into a computer to send keys", 8, 6, 4, 0xffb060)
    end
    for key, pos in pairs(LAYOUT) do
        local x = 8 + (pos[1] - 1) * (W + GAP)
        local y = 26 + (pos[2] - 1) * (H + GAP) - 8
        local on = held[key]
        round_rect(x, y, W, H, 5, on and 0xffd84a or 0x3a2f66)
        text(PAD[key][1], x + W / 2, y + 10, 4, on and 0x000000 or 0xffffff, "center")
    end
end
