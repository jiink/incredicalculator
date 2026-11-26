-- title: Face
-- api:   1
-- desc:  A pair of animated robot eyes you steer with the number pad.
-- color: 0x000000
-- realtime: true
-- icon: VM1:AAAMAQdEZWZhdWx0AQEDAP/3AAgB/AEEAgUFBQYEBvwF+wL7AP/3AAj6+/oE+wX+Bf8E//v++vv6AP/3AAT+BgIGAwf9Bw==

-- Face: animated robot eyes, ported from FluxGarage's RoboEyes.
--
--   7 8 9
--   4 5 6   where the eyes look (5 = center); stops the idle wandering
--   1 2 3
--   /  *  -  +   mood: normal, angry, tired, happy
--   =   laugh      bksp   confused      0   sweat

local BG   = 0x000000
local MAIN = 0xFFDA02

local W, H = 320, 240
local EYE_W, EYE_H, SPACE = 50, 100, 20

local e -- all the animation state, rebuilt by init()

local seed
local function random(max)
    if max <= 0 then return 0 end
    seed = (seed * 1664525 + 1013904223) & 0xFFFFFFFF
    return (seed >> 1) % max
end

local function new_state()
    local x = (W - (EYE_W * 2 + SPACE)) // 2
    local y = (H - EYE_H) // 2
    return {
        last_frame = 0,
        tired = false, angry = false, happy = false,
        l = { w = EYE_W, h = 1, h_next = EYE_H, r = 16, r_next = 8, x = x, y = y, nx = x, ny = y },
        r = { w = EYE_W, h = 1, h_next = EYE_H, r = 16, r_next = 8, x = x + EYE_W + SPACE, y = y,
              nx = x + EYE_W + SPACE, ny = y },
        space = SPACE,
        tired_h = 0, angry_h = 0, happy_off = 0,
        h_flicker = false, h_amp = 2, h_alt = false,
        v_flicker = false, v_amp = 10, v_alt = false,
        blink_timer = 0, idle = true, idle_timer = 0,
        confused = false, confused_timer = 0, confused_toggle = true,
        laugh = false, laugh_timer = 0, laugh_toggle = true,
        sweat = false,
        drop = { x_initial = 0, x = 0, y = 4.0, y_max = 100, w = 2.0, h = 4.0 },
    }
end

local function constraint_x() return W - e.l.w - e.space - e.r.w end
local function constraint_y() return H - EYE_H end

local function look(px, py)
    e.idle = false
    local mx, my = constraint_x(), constraint_y()
    e.l.nx, e.l.ny = mx * px // 2, my * py // 2
end

local function set_mood(m)
    e.tired, e.angry, e.happy = m == "tired", m == "angry", m == "happy"
end

local function rrect(x, y, w, h, r, color)
    if w < 1 or h < 1 then return end
    round_rect(x, y, w, h, math.min(r, w // 2, h // 2), color)
end

local function step(now)
    local l, r = e.l, e.r

    l.h = (l.h + l.h_next) // 2
    l.y = l.y + (EYE_H - l.h) // 2
    r.h = (r.h + r.h_next) // 2
    r.y = r.y + (EYE_H - r.h) // 2
    if l.h <= 1 then l.h_next = EYE_H end
    if r.h <= 1 then r.h_next = EYE_H end

    l.w = (l.w + EYE_W) // 2
    r.w = (r.w + EYE_W) // 2
    l.r = (l.r + l.r_next) // 2
    r.r = (r.r + r.r_next) // 2
    e.space = (e.space + SPACE) // 2

    l.x = (l.x + l.nx) // 2
    l.y = (l.y + l.ny) // 2
    r.nx = l.nx + l.w + e.space
    r.ny = l.ny
    r.x = (r.x + r.nx) // 2
    r.y = (r.y + r.ny) // 2

    -- auto-blink every 2-6 seconds
    if now >= e.blink_timer then
        l.h_next, r.h_next = 1, 1
        e.blink_timer = now + 2000 + random(3000)
    end

    if e.laugh then
        if e.laugh_toggle then
            e.v_flicker, e.v_amp, e.laugh_timer, e.laugh_toggle = true, 5, now, false
        elseif now >= e.laugh_timer + 500 then
            e.v_flicker, e.laugh, e.laugh_toggle = false, false, true
        end
    end

    if e.confused then
        if e.confused_toggle then
            e.h_flicker, e.h_amp, e.confused_timer, e.confused_toggle = true, 20, now, false
        elseif now >= e.confused_timer + 500 then
            e.h_flicker, e.confused, e.confused_toggle = false, false, true
        end
    end

    if e.idle and now >= e.idle_timer then
        l.nx = random(constraint_x())
        l.ny = random(constraint_y())
        e.idle_timer = now + 3000 + random(2000)
    end

    if e.h_flicker then
        local d = e.h_alt and e.h_amp or -e.h_amp
        l.x, r.x = l.x + d, r.x + d
        e.h_alt = not e.h_alt
    end
    if e.v_flicker then
        local d = e.v_alt and e.v_amp or -e.v_amp
        l.y, r.y = l.y + d, r.y + d
        e.v_alt = not e.v_alt
    end
end

local function draw_eyes()
    local l, r = e.l, e.r
    clear(BG)
    rrect(l.x, l.y, l.w, l.h, l.r, MAIN)
    rrect(r.x, r.y, r.w, r.h, r.r, MAIN)

    e.tired_h = (e.tired_h + (e.tired and l.h // 2 or 0)) // 2
    if e.tired_h > 0 then
        local h = e.tired_h
        triangle(l.x, l.y - 1, l.x + l.w, l.y - 1, l.x, l.y + h - 1, BG)
        triangle(r.x, r.y - 1, r.x + r.w, r.y - 1, r.x + r.w, r.y + h - 1, BG)
    end

    e.angry_h = (e.angry_h + (e.angry and l.h // 2 or 0)) // 2
    if e.angry_h > 0 then
        local h = e.angry_h
        triangle(l.x, l.y - 1, l.x + l.w, l.y - 1, l.x + l.w, l.y + h - 1, BG)
        triangle(r.x, r.y - 1, r.x + r.w, r.y - 1, r.x, r.y + h - 1, BG)
    end

    e.happy_off = (e.happy_off + (e.happy and l.h // 2 or 0)) // 2
    if e.happy_off > 0 then
        local off = e.happy_off
        rrect(l.x - 1, l.y + l.h - off + 1, l.w + 2, EYE_H, l.r, BG)
        rrect(r.x - 1, r.y + r.h - off + 1, r.w + 2, EYE_H, r.r, BG)
    end

    if e.sweat then
        local d = e.drop
        if d.y <= d.y_max then
            d.y = d.y + 0.5
        else
            d.x_initial = random(30)
            d.y = 2.0
            d.y_max = random(10) + 10
            d.w, d.h = 1.0, 2.0
        end
        if d.y <= d.y_max // 2 then
            d.w, d.h = d.w + 0.5, d.h + 0.5
        else
            d.w, d.h = d.w - 0.1, d.h - 0.5
        end
        d.x = d.x_initial - math.floor(d.w / 2)
        rrect(d.x, math.floor(d.y), math.floor(d.w), math.floor(d.h), 3, MAIN)
    end
end

function init()
    seed = 12345
    e = new_state()
end

local LOOK = {
    ["7"] = { 0, 0 }, ["8"] = { 1, 0 }, ["9"] = { 2, 0 },
    ["4"] = { 0, 1 }, ["5"] = { 1, 1 }, ["6"] = { 2, 1 },
    ["1"] = { 0, 2 }, ["2"] = { 1, 2 }, ["3"] = { 2, 2 },
}
local MOODS = { ["/"] = "normal", ["*"] = "angry", ["-"] = "tired", ["+"] = "happy" }

function on_key(key, action)
    if LOOK[action] then
        look(LOOK[action][1], LOOK[action][2])
    elseif MOODS[action] then
        set_mood(MOODS[action])
    elseif action == "enter" then
        e.laugh, e.laugh_toggle = true, true
    elseif action == "backspace" then
        e.confused, e.confused_toggle = true, true
    elseif action == "0" then
        e.sweat = not e.sweat
    end
end

function update()
    local now = millis()
    if now - e.last_frame >= 20 then
        e.last_frame = now
        step(now)
    end
    draw_eyes()
end
