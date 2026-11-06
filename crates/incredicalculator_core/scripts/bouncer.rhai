// Bouncer: an example scripted app.
//
// The optional top-level `app` map sets how the app appears in the menu.
// `realtime: true` runs `update` every frame instead of only after a key press.
let app = #{ name: "Bouncer", color: 0x2a6f97, realtime: true };

// Hooks (all optional):
//   init()       runs once, the first time the app is opened
//   update()     draws the screen; drawing only works here
//   on_key(key)  key is "0"-"9", "f1"-"f6", "shift" or "super"
//
// Functions can't see top-level variables, so keep state on `this`, which
// persists between calls.
//
// Drawing (screen is 320x240, colors are 0xRRGGBB or rgb(r, g, b)):
//   clear(color)
//   line(x1, y1, x2, y2, color [, width])
//   rect(x, y, w, h, color)   rect_outline(x, y, w, h, color, width)
//   round_rect(x, y, w, h, radius, color)
//   triangle(x1, y1, x2, y2, x3, y3, color)
//   polygon([x1, y1, x2, y2, x3, y3, ...], color)
//   text(string, x, y, size, color [, "left" | "center" | "right"])
//   text_width(string, size)
// Input and time: is_down(key), millis().  print(value) writes to the log.

fn init() {
    this.size = 28.0;
    this.x = 40.0;
    this.y = 60.0;
    this.vx = 90.0;   // pixels per second
    this.vy = 70.0;
    this.paused = false;
    this.bounces = 0;
    this.last = millis();
}

fn on_key(key) {
    switch key {
        "5" => { this.paused = !this.paused; }
        "8" => { this.vy -= 40.0; }
        "2" => { this.vy += 40.0; }
        "4" => { this.vx -= 40.0; }
        "6" => { this.vx += 40.0; }
        // Method-call syntax passes `this` along to another hook.
        "0" => { this.init(); }
    }
}

fn bounce_color(n) {
    let colors = [0xff5e5b, 0xffed66, 0x00cecb, 0xd8d8d8, 0x9b5de5];
    colors[n % colors.len()]
}

fn update() {
    let now = millis();
    let dt = (now - this.last) / 1000.0;
    this.last = now;
    if dt > 0.1 { dt = 0.1; }  // don't jump after a pause

    if !this.paused {
        this.x += this.vx * dt;
        this.y += this.vy * dt;
        let right = 320.0 - this.size;
        let bottom = 220.0 - this.size;
        if this.x < 0.0 { this.x = 0.0; this.vx = this.vx.abs(); this.bounces += 1; }
        if this.x > right { this.x = right; this.vx = -this.vx.abs(); this.bounces += 1; }
        if this.y < 20.0 { this.y = 20.0; this.vy = this.vy.abs(); this.bounces += 1; }
        if this.y > bottom { this.y = bottom; this.vy = -this.vy.abs(); this.bounces += 1; }
    }

    clear(0x101820);
    line(0, 20, 320, 20, 0x2a6f97);
    line(0, 220, 320, 220, 0x2a6f97);
    text(`Bounces: ${this.bounces}`, 6, 15, 6, 0xffffff);
    if this.paused {
        text("PAUSED", 314, 15, 6, 0xffed66, "right");
    }
    round_rect(this.x, this.y, this.size, this.size, 6, bounce_color(this.bounces));
    text("2 4 6 8: push   5: pause   0: reset", 160, 235, 5, 0x8899aa, "center");
}
