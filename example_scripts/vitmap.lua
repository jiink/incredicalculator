-- title: Vitmap
-- api:   1
-- desc:  Draws a vector picture made in the vitmap editor.
-- color: 0x2a6f97
-- icon:  VM1:AAAMAQdEZWZhdWx0AQEOAAAAAAz5CAcIBwcCBwIGCAYI+Pj4+Ab+Bv4H+gcAACFMBPkB/AH8BPkEALuxvgT8Af8B/wT8BAA/Ek8E/wECAQIE/wQAu7G+BPkB+fn7+fsBAMLNIwT7+f35/QH7AQAAwMAE/fn/+f8B/QEAAMAABP/5AfkBAf8BAMAAwAQB+QP5AwEBAQDAAAAEA/kG+QYBAwEAAAAABP/1/vT/8wD0AAAAAAQD9gP0BfQF9gAAAAADAfgE9QL4AAAAAAP/9AH4APg=

-- Vitmap: draws a vector picture made in the vitmap editor.
--
-- Export a vitmap from the editor and paste it below. The same picture is
-- this app's icon in the menu, through the `icon:` line in the header above.

local picture = vitmap[[VM1:AAAMAQdEZWZhdWx0AQEOAAAAAAz5CAcIBwcCBwIGCAYI+Pj4+Ab+Bv4H+gcAACFMBPkB/AH8BPkEALuxvgT8Af8B/wT8BAA/Ek8E/wECAQIE/wQAu7G+BPkB+fn7+fsBAMLNIwT7+f35/QH7AQAAwMAE/fn/+f8B/QEAAMAABP/5AfkBAf8BAMAAwAQB+QP5AwEBAQDAAAAEA/kG+QYBAwEAAAAABP/1/vT/8wD0AAAAAAQD9gP0BfQF9gAAAAADAfgE9QL4AAAAAAP/9AH4APg=]]

function update()
    clear(0xaaaaaa)
    -- 1:1 size by default
    picture:draw(160, 120)
    picture:draw(160, 60, { scale = 4.0 })
end
