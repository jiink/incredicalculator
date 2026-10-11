use core::fmt;
use std::cell::RefCell;
use std::rc::Rc;

use glam::{IVec2, Vec2};
use rgb::RGB8;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console, window, CanvasRenderingContext2d, HtmlCanvasElement};

thread_local! {
    static APP: RefCell<Option<BrowserState>> = RefCell::new(None);
}

struct BrowserState {
    shell: Rc<RefCell<IcShell>>,
    platform: Rc<RefCell<BrowserPlatform>>,
}

use incredicalculator_core::input::IcKey;
use incredicalculator_core::platform::{IcPlatform, CANVAS_HEIGHT, CANVAS_WIDTH};
use incredicalculator_core::shell::IcShell;

struct BrowserPlatform {
    ctx: CanvasRenderingContext2d,
    start_ms: f64,
    brightness: u8,
    volume: u8,
}

impl BrowserPlatform {
    fn rgb_css(color: RGB8) -> String {
        format!("rgb({},{},{})", color.r, color.g, color.b)
    }

    fn rounded_rect_path(&self, x: f64, y: f64, width: f64, height: f64, radius: f64) {
        let x0 = x;
        let y0 = y;
        let x1 = x + width;
        let y1 = y + height;
        let r = radius.min(width / 2.0).min(height / 2.0);

        self.ctx.begin_path();
        self.ctx.move_to(x0 + r, y0);
        self.ctx.line_to(x1 - r, y0);
        let _ = self.ctx.arc_to(x1, y0, x1, y0 + r, r);
        self.ctx.line_to(x1, y1 - r);
        let _ = self.ctx.arc_to(x1, y1, x1 - r, y1, r);
        self.ctx.line_to(x0 + r, y1);
        let _ = self.ctx.arc_to(x0, y1, x0, y1 - r, r);
        self.ctx.line_to(x0, y0 + r);
        let _ = self.ctx.arc_to(x0, y0, x0 + r, y0, r);
        self.ctx.close_path();
    }
}

impl IcPlatform for BrowserPlatform {
    fn draw_line(&mut self, start: Vec2, end: Vec2, color: RGB8, width: u32) {
        self.ctx.begin_path();
        self.ctx.move_to(start.x as f64, start.y as f64);
        self.ctx.line_to(end.x as f64, end.y as f64);
        self.ctx.set_line_width(width as f64);
        self.ctx.set_line_cap("round");
        self.ctx.set_line_join("round");
        self.ctx
            .set_stroke_style(&JsValue::from_str(&Self::rgb_css(color)));
        self.ctx.stroke();
    }

    fn draw_polygon(&mut self, points: &[Vec2], fill_color: RGB8) {
        if points.is_empty() {
            return;
        }

        self.ctx.begin_path();
        self.ctx.move_to(points[0].x as f64, points[0].y as f64);
        for point in points.iter().skip(1) {
            self.ctx.line_to(point.x as f64, point.y as f64);
        }
        self.ctx.close_path();
        self.ctx
            .set_fill_style(&JsValue::from_str(&Self::rgb_css(fill_color)));
        self.ctx.fill();
    }

    fn draw_rectangle(
        &mut self,
        start: IVec2,
        end: IVec2,
        stroke_color: RGB8,
        stroke_width: u32,
        fill_color: Option<RGB8>,
    ) {
        let x = start.x.min(end.x) as f64;
        let y = start.y.min(end.y) as f64;
        let w = (start.x.abs_diff(end.x) as f64).max(0.0);
        let h = (start.y.abs_diff(end.y) as f64).max(0.0);

        if let Some(fill) = fill_color {
            self.ctx
                .set_fill_style(&JsValue::from_str(&Self::rgb_css(fill)));
            self.ctx.fill_rect(x, y, w, h);
        }

        if stroke_width > 0 {
            self.ctx
                .set_stroke_style(&JsValue::from_str(&Self::rgb_css(stroke_color)));
            self.ctx.set_line_width(stroke_width as f64);
            self.ctx.stroke_rect(x, y, w, h);
        }
    }

    fn draw_rectangle_rounded(
        &mut self,
        start: IVec2,
        end: IVec2,
        stroke_color: RGB8,
        stroke_width: u32,
        fill_color: Option<RGB8>,
        corner_radius: u32,
    ) {
        let x = start.x.min(end.x) as f64;
        let y = start.y.min(end.y) as f64;
        let w = (start.x.abs_diff(end.x) as f64).max(0.0);
        let h = (start.y.abs_diff(end.y) as f64).max(0.0);
        let r = corner_radius as f64;

        self.rounded_rect_path(x, y, w, h, r);

        if let Some(fill) = fill_color {
            self.ctx
                .set_fill_style(&JsValue::from_str(&Self::rgb_css(fill)));
            self.ctx.fill();
        }

        if stroke_width > 0 {
            self.ctx
                .set_stroke_style(&JsValue::from_str(&Self::rgb_css(stroke_color)));
            self.ctx.set_line_width(stroke_width as f64);
            self.ctx.stroke();
        }
    }

    fn draw_triangle(
        &mut self,
        vertex1: IVec2,
        vertex2: IVec2,
        vertex3: IVec2,
        stroke_color: RGB8,
        stroke_width: u32,
        fill_color: Option<RGB8>,
    ) {
        self.ctx.begin_path();
        self.ctx.move_to(vertex1.x as f64, vertex1.y as f64);
        self.ctx.line_to(vertex2.x as f64, vertex2.y as f64);
        self.ctx.line_to(vertex3.x as f64, vertex3.y as f64);
        self.ctx.close_path();

        if let Some(fill) = fill_color {
            self.ctx
                .set_fill_style(&JsValue::from_str(&Self::rgb_css(fill)));
            self.ctx.fill();
        }

        if stroke_width > 0 {
            self.ctx
                .set_stroke_style(&JsValue::from_str(&Self::rgb_css(stroke_color)));
            self.ctx.set_line_width(stroke_width as f64);
            self.ctx.stroke();
        }
    }

    fn clear(&mut self, color: RGB8) {
        self.ctx
            .set_fill_style(&JsValue::from_str(&Self::rgb_css(color)));
        self.ctx
            .fill_rect(0.0, 0.0, CANVAS_WIDTH as f64, CANVAS_HEIGHT as f64);
    }

    fn log(&mut self, arg: fmt::Arguments) {
        let message = format!("{}", arg);
        console::log_1(&message.into());
    }

    fn millis(&self) -> u64 {
        window()
            .and_then(|w| w.performance())
            .map(|p| (p.now() - self.start_ms) as u64)
            .unwrap_or(0)
    }

    fn get_battery_soc(&self) -> Option<i32> {
        None
    }

    fn get_brightness(&self) -> u8 {
        self.brightness
    }

    fn set_brightness(&mut self, value: u8) {
        self.brightness = value;
    }

    fn get_volume(&self) -> u8 {
        self.volume
    }

    fn set_volume(&mut self, value: u8) {
        self.volume = value;
    }
}

fn map_code(code: &str) -> Option<IcKey> {
    match code {
        "Digit0" => Some(IcKey::Num0),
        "Digit1" => Some(IcKey::Num1),
        "Digit2" => Some(IcKey::Num2),
        "Digit3" => Some(IcKey::Num3),
        "Digit4" => Some(IcKey::Num4),
        "Digit5" => Some(IcKey::Num5),
        "Digit6" => Some(IcKey::Num6),
        "Digit7" => Some(IcKey::Num7),
        "Digit8" => Some(IcKey::Num8),
        "Digit9" => Some(IcKey::Num9),
        "Numpad0" => Some(IcKey::Num0),
        "Numpad1" => Some(IcKey::Num1),
        "Numpad2" => Some(IcKey::Num2),
        "Numpad3" => Some(IcKey::Num3),
        "Numpad4" => Some(IcKey::Num4),
        "Numpad5" => Some(IcKey::Num5),
        "Numpad6" => Some(IcKey::Num6),
        "Numpad7" => Some(IcKey::Num7),
        "Numpad8" => Some(IcKey::Num8),
        "Numpad9" => Some(IcKey::Num9),
        "KeyA" => Some(IcKey::Func1),
        "KeyB" => Some(IcKey::Func2),
        "KeyC" => Some(IcKey::Func3),
        "KeyD" => Some(IcKey::Func4),
        "KeyE" => Some(IcKey::Func6),
        "KeyF" => Some(IcKey::Func5),
        "Backspace" => Some(IcKey::Func1),
        "Slash" => Some(IcKey::Func2),
        "NumpadMultiply" => Some(IcKey::Func3),
        "Minus" | "NumpadSubtract" => Some(IcKey::Func4),
        "NumpadAdd" => Some(IcKey::Func5),
        "Equal" | "NumpadEnter" => Some(IcKey::Func6),
        "F1" => Some(IcKey::Func1),
        "F2" => Some(IcKey::Func2),
        "F3" => Some(IcKey::Func3),
        "F4" => Some(IcKey::Func4),
        "F5" => Some(IcKey::Func5),
        "F6" => Some(IcKey::Func6),
        "ShiftLeft" | "ShiftRight" => Some(IcKey::Shift),
        "MetaLeft" | "MetaRight" | "ControlLeft" | "ControlRight" => Some(IcKey::Super),
        _ => None,
    }
}

#[wasm_bindgen]
pub fn key_down(code: &str) {
    let Some(key) = map_code(code) else {
        return;
    };

    APP.with(|app| {
        let mut app = app.borrow_mut();
        let Some(app) = app.as_mut() else {
            return;
        };
        app.shell.borrow_mut().key_down(key);
    });
}

#[wasm_bindgen]
pub fn key_up(code: &str) {
    let Some(key) = map_code(code) else {
        return;
    };

    APP.with(|app| {
        let mut app = app.borrow_mut();
        let Some(app) = app.as_mut() else {
            return;
        };
        app.shell.borrow_mut().key_up(key);
    });
}

#[wasm_bindgen]
pub fn tick() {
    APP.with(|app| {
        let mut app = app.borrow_mut();
        let Some(app) = app.as_mut() else {
            return;
        };
        let mut shell = app.shell.borrow_mut();
        let mut platform = app.platform.borrow_mut();
        shell.update(&mut *platform);
    });
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();

    let window = window().expect("browser window should exist");
    let document = window.document().expect("document should exist");

    let canvas = document
        .get_element_by_id("incredicalculator")
        .expect("#incredicalculator canvas should exist")
        .dyn_into::<HtmlCanvasElement>()
        .expect("element is not a canvas");

    canvas.set_width(CANVAS_WIDTH);
    canvas.set_height(CANVAS_HEIGHT);

    let context = canvas
        .get_context("2d")
        .expect("canvas cannot create 2d context")
        .expect("canvas 2d context missing")
        .dyn_into::<CanvasRenderingContext2d>()
        .expect("context is not 2d");
    context.set_image_smoothing_enabled(true);

    let shell = Rc::new(RefCell::new(IcShell::new()));
    let platform = Rc::new(RefCell::new(BrowserPlatform {
        ctx: context,
        start_ms: window.performance().map(|p| p.now()).unwrap_or(0.0),
        brightness: 255,
        volume: 255,
    }));

    APP.with(|slot| {
        *slot.borrow_mut() = Some(BrowserState { shell: Rc::clone(&shell), platform: Rc::clone(&platform) });
    });
}
