//! [`ScriptEngine`] backed by Lua 5.4, via tsuki (Lua ported to Rust; the
//! vendored copy in `crates/tsuki-patched` adds a no_std build, a budget for
//! endless loops and a smaller stack limit).
//!
//! Scripts keep state in ordinary top-level `local`s or globals. See `example_scripts/bouncer.lua` for
//! the API from the script's side.

use alloc::boxed::Box;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cell::{Cell, RefCell};
use core::error::Error;
use core::pin::Pin;

use glam::{IVec2, Vec2};
use tsuki::builtin::{BaseLib, MathLib, StrLib, TableLib};
use tsuki::context::{Args, Context, Ret};
use tsuki::{CallError, Lua, RegKey, Thread, Value, fp};

use super::{
    DrawCmd, ScriptEngine, ScriptHost, ScriptMeta, align_from_name, color_from_int, key_from_name,
    key_name,
};
use crate::input::IcKey;
use crate::text::{HAlign, TextStyle, text_width};

/// Jumps, loop iterations and calls allowed per hook call before it is
/// stopped, so an endless loop shows an error instead of freezing.
const MAX_BUDGET: u32 = 100_000;
/// Fixed so string hashing doesn't depend on a random source.
const HASH_SEED: u32 = 0x1C_A1C0;

/// The Lua state's associated data, reachable from native functions.
struct Host {
    host: RefCell<ScriptHost>,
    /// xorshift32 state for `math.random`; 0 until first seeded.
    rng: Cell<u32>,
}

impl Host {
    fn get(&self) -> core::cell::RefMut<'_, ScriptHost> {
        self.host.borrow_mut()
    }

    fn next_random(&self) -> u32 {
        let mut x = self.rng.get();
        if x == 0 {
            x = (self.get().millis as u32) | 1;
        }
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng.set(x);
        x
    }
}
type Cx<'a> = Context<'a, Host, Args>;
type FnResult<'a> = Result<Context<'a, Host, Ret>, Box<dyn Error>>;

/// Registry slot holding the one thread every hook runs on. Creating a thread
/// per call would leave about a kilobyte of garbage each time, and tsuki's
/// collector paces itself by object count, so the heap would run far ahead of
/// collection.
struct HookThread;

impl RegKey<Host> for HookThread {
    type Value<'a>
        = Thread<Host>
    where
        Host: 'a;
}

pub struct LuaEngine {
    lua: Pin<Rc<Lua<Host>>>,
    has_init: bool,
    has_update: bool,
    has_on_key: bool,
}

impl LuaEngine {
    /// Calls the global function `name`, with the caller's host swapped into
    /// the state the native functions write to.
    fn call(&mut self, host: &mut ScriptHost, name: &str, key: Option<&str>) -> Result<(), String> {
        core::mem::swap(&mut *self.lua.associated_data().get(), host);
        self.lua.set_budget(Some(MAX_BUDGET));
        let thread = self.lua.registry::<HookThread>().expect("hook thread");
        let function = self.lua.global().get_str_key(name);
        let result = match key {
            Some(key) => {
                let key = self.lua.create_str(key);
                thread.call::<()>(function, key)
            }
            None => thread.call::<()>(function, ()),
        };
        self.lua.set_budget(None);
        core::mem::swap(&mut *self.lua.associated_data().get(), host);
        result.map_err(|e| describe(&*e))
    }
}

/// Formats an error with its line number when it has one.
fn describe(error: &(dyn Error + 'static)) -> String {
    match error.downcast_ref::<CallError>() {
        Some(call) => match call.location() {
            Some((chunk, line)) => format!("{chunk}:{line}: {call}"),
            None => call.to_string(),
        },
        None => error.to_string(),
    }
}

impl ScriptEngine for LuaEngine {
    const FILE_EXTENSION: &'static str = "lua";

    fn load(source: &str) -> Result<(Self, ScriptMeta), String> {
        let lua = Lua::with_seed(
            Host {
                host: RefCell::new(ScriptHost::new()),
                rng: Cell::new(0),
            },
            HASH_SEED,
        );
        let fail = |e: &dyn core::fmt::Display| e.to_string();
        lua.use_module(None, true, BaseLib).map_err(|e| fail(&e))?;
        lua.use_module(None, true, MathLib).map_err(|e| fail(&e))?;
        lua.use_module(None, true, StrLib).map_err(|e| fail(&e))?;
        lua.use_module(None, true, TableLib).map_err(|e| fail(&e))?;
        register_api(&lua);

        lua.set_registry::<HookThread>(&lua.create_thread());

        let chunk = lua.load("script", source).map_err(|e| e.to_string())?;
        lua.set_budget(Some(MAX_BUDGET));
        let ran = lua
            .registry::<HookThread>()
            .expect("hook thread")
            .call::<()>(&*chunk, ());
        lua.set_budget(None);
        drop(chunk);
        ran.map_err(|e| describe(&*e))?;
        let meta = read_meta(&lua)?;

        let is_fn = |name: &str| matches!(lua.global().get_str_key(name), Value::LuaFn(_));
        let (has_init, has_update, has_on_key) = (is_fn("init"), is_fn("update"), is_fn("on_key"));
        Ok((
            Self {
                lua,
                has_init,
                has_update,
                has_on_key,
            },
            meta,
        ))
    }

    fn init(&mut self, host: &mut ScriptHost) -> Result<(), String> {
        if !self.has_init {
            return Ok(());
        }
        self.call(host, "init", None)
    }

    fn update(&mut self, host: &mut ScriptHost) -> Result<(), String> {
        if !self.has_update {
            return Ok(());
        }
        self.call(host, "update", None)
    }

    fn on_key(&mut self, key: IcKey, host: &mut ScriptHost) -> Result<(), String> {
        if !self.has_on_key {
            return Ok(());
        }
        self.call(host, "on_key", Some(key_name(key)))
    }
}

/// Reads the optional global `app` table, e.g.
/// `app = { name = "Bouncer", color = 0x2a6f97, realtime = true }`
fn read_meta(lua: &Lua<Host>) -> Result<ScriptMeta, String> {
    let mut meta = ScriptMeta::default();
    let app = match lua.global().get_str_key("app") {
        Value::Nil => return Ok(meta),
        Value::Table(app) => app,
        _ => return Err("`app` must be a table, like { name = \"My App\" }".into()),
    };
    match app.get_str_key("name") {
        Value::Nil => {}
        Value::Str(name) => match name.as_utf8() {
            Some(name) => meta.name = Some(String::from(name)),
            None => return Err("app.name must be text".into()),
        },
        _ => return Err("app.name must be a string".into()),
    }
    match app.get_str_key("color") {
        Value::Nil => {}
        Value::Int(color) => meta.color = Some(color as u32),
        _ => return Err("app.color must be an integer like 0xRRGGBB".into()),
    }
    match app.get_str_key("realtime") {
        Value::Nil | Value::False => {}
        Value::True => meta.realtime = true,
        _ => return Err("app.realtime must be true or false".into()),
    }
    Ok(meta)
}

fn register_api(lua: &Lua<Host>) {
    let g = lua.global();
    g.set_str_key("print", fp!(print));
    g.set_str_key("millis", fp!(millis));
    g.set_str_key("is_down", fp!(is_down));
    g.set_str_key("rgb", fp!(rgb));
    g.set_str_key("text_width", fp!(text_width_fn));
    g.set_str_key("clear", fp!(clear));
    g.set_str_key("line", fp!(line));
    g.set_str_key("rect", fp!(rect));
    g.set_str_key("rect_outline", fp!(rect_outline));
    g.set_str_key("round_rect", fp!(round_rect));
    g.set_str_key("triangle", fp!(triangle));
    g.set_str_key("polygon", fp!(polygon));
    g.set_str_key("text", fp!(text));

    // Standard functions tsuki doesn't implement yet.
    if let Value::Table(table) = g.get_str_key("table") {
        table.set_str_key("insert", fp!(table_insert));
        table.set_str_key("remove", fp!(table_remove));
    }
    if let Value::Table(math) = g.get_str_key("math") {
        math.set_str_key("random", fp!(math_random));
    }
}

/// `table.insert(t, [pos,] value)`
fn table_insert(cx: Cx) -> FnResult {
    let list = cx.arg(1).get_table()?;
    let end = list.len() + 1;
    let (pos, value) = match cx.args() {
        2 => (end, cx.arg(2)),
        3 => {
            let pos_arg = cx.arg(2);
            let pos = pos_arg.to_int()?;
            if pos < 1 || pos > end {
                return Err(pos_arg.error("position out of bounds"));
            }
            (pos, cx.arg(3))
        }
        _ => return Err("wrong number of arguments to 'insert'".into()),
    };
    for i in (pos..end).rev() {
        list.set(i + 1, list.get(i))?;
    }
    list.set(pos, value)?;
    Ok(cx.into())
}

/// `table.remove(t [, pos])`, returning the removed value.
fn table_remove(cx: Cx) -> FnResult {
    let list = cx.arg(1).get_table()?;
    let len = list.len();
    let pos_arg = cx.arg(2);
    let pos = pos_arg.to_nilable_int(false)?.unwrap_or(len);
    if len == 0 && (pos == 0 || pos == len) {
        cx.push(list.get(pos))?;
        return Ok(cx.into());
    }
    if pos < 1 || pos > len + 1 {
        return Err(pos_arg.error("position out of bounds"));
    }
    cx.push(list.get(pos))?;
    for i in pos..len {
        list.set(i, list.get(i + 1))?;
    }
    list.set(pos.max(len), tsuki::Nil)?;
    Ok(cx.into())
}

/// `math.random()` for a float in [0, 1), `math.random(m)` for an integer in
/// [1, m], `math.random(m, n)` for an integer in [m, n].
fn math_random(cx: Cx) -> FnResult {
    let random = cx.associated_data().next_random();
    let (low, high) = match cx.args() {
        0 => {
            cx.push((random >> 8) as f64 / (1u32 << 24) as f64)?;
            return Ok(cx.into());
        }
        1 => (1, cx.arg(1).to_int()?),
        _ => (cx.arg(1).to_int()?, cx.arg(2).to_int()?),
    };
    if high < low {
        return Err("bad argument to 'random' (interval is empty)".into());
    }
    let span = (high - low) as u64 + 1;
    cx.push(low + (random as u64 % span) as i64)?;
    Ok(cx.into())
}

// ---- Argument helpers (Lua arguments are 1-based) ----

fn num(cx: &Cx, n: usize) -> Result<f32, Box<dyn Error>> {
    Ok(f64::from(cx.arg(n).to_float()?) as f32)
}

fn int(cx: &Cx, n: usize) -> Result<i32, Box<dyn Error>> {
    num(cx, n).map(|v| v as i32)
}

fn color(cx: &Cx, n: usize) -> Result<rgb::RGB8, Box<dyn Error>> {
    Ok(color_from_int(cx.arg(n).to_int()?))
}

fn push(cx: &Cx, command: DrawCmd) -> Result<(), Box<dyn Error>> {
    cx.associated_data()
        .get()
        .push(command)
        .map_err(|e| e.into())
}

// ---- Inputs and helpers ----

fn print(cx: Cx) -> FnResult {
    let mut line = String::new();
    for n in 1..=cx.args() {
        if n > 1 {
            line.push('\t');
        }
        // Like Lua's own print: tostring() each argument.
        line.push_str(cx.arg(n).display()?.as_utf8().unwrap_or("?"));
    }
    cx.associated_data().get().log(&line);
    Ok(cx.into())
}

fn millis(cx: Cx) -> FnResult {
    let millis = cx.associated_data().get().millis as i64;
    cx.push(millis)?;
    Ok(cx.into())
}

fn is_down(cx: Cx) -> FnResult {
    let arg = cx.arg(1);
    let name = arg.to_utf8()?;
    let key = key_from_name(name).ok_or_else(|| arg.error(format!("unknown key \"{name}\"")))?;
    let down = cx.associated_data().get().keys_down[key as usize];
    cx.push(down)?;
    Ok(cx.into())
}

fn rgb(cx: Cx) -> FnResult {
    let channel = |n| int(&cx, n).map(|c| c.clamp(0, 255) as i64);
    let value = channel(1)? << 16 | channel(2)? << 8 | channel(3)?;
    cx.push(value)?;
    Ok(cx.into())
}

fn text_width_fn(cx: Cx) -> FnResult {
    let style = TextStyle::new(num(&cx, 2)?, rgb::RGB8::default());
    let width = text_width(cx.arg(1).to_utf8()?, style.scale, style.font);
    cx.push(width as f64)?;
    Ok(cx.into())
}

// ---- Drawing ----

fn clear(cx: Cx) -> FnResult {
    push(&cx, DrawCmd::Clear(color(&cx, 1)?))?;
    Ok(cx.into())
}

fn line(cx: Cx) -> FnResult {
    let width = cx.arg(6).to_nilable_int(false)?.unwrap_or(2).max(1) as u32;
    push(
        &cx,
        DrawCmd::Line {
            start: Vec2::new(num(&cx, 1)?, num(&cx, 2)?),
            end: Vec2::new(num(&cx, 3)?, num(&cx, 4)?),
            color: color(&cx, 5)?,
            width,
        },
    )?;
    Ok(cx.into())
}

/// `x, y, w, h` at arguments 1-4.
fn rect_bounds(cx: &Cx) -> Result<(IVec2, IVec2), Box<dyn Error>> {
    let start = IVec2::new(int(cx, 1)?, int(cx, 2)?);
    Ok((start, start + IVec2::new(int(cx, 3)?, int(cx, 4)?)))
}

fn rect(cx: Cx) -> FnResult {
    let (start, end) = rect_bounds(&cx)?;
    let color = color(&cx, 5)?;
    push(
        &cx,
        DrawCmd::Rect {
            start,
            end,
            color,
            outline: 0,
            radius: 0,
        },
    )?;
    Ok(cx.into())
}

fn rect_outline(cx: Cx) -> FnResult {
    let (start, end) = rect_bounds(&cx)?;
    let color = color(&cx, 5)?;
    let outline = int(&cx, 6)?.max(1) as u32;
    push(
        &cx,
        DrawCmd::Rect {
            start,
            end,
            color,
            outline,
            radius: 0,
        },
    )?;
    Ok(cx.into())
}

fn round_rect(cx: Cx) -> FnResult {
    let (start, end) = rect_bounds(&cx)?;
    let radius = int(&cx, 5)?.max(0) as u32;
    let color = color(&cx, 6)?;
    push(
        &cx,
        DrawCmd::Rect {
            start,
            end,
            color,
            outline: 0,
            radius,
        },
    )?;
    Ok(cx.into())
}

fn triangle(cx: Cx) -> FnResult {
    let point = |n| Ok::<_, Box<dyn Error>>(IVec2::new(int(&cx, n)?, int(&cx, n + 1)?));
    let points = [point(1)?, point(3)?, point(5)?];
    push(
        &cx,
        DrawCmd::Triangle {
            points,
            color: color(&cx, 7)?,
        },
    )?;
    Ok(cx.into())
}

fn polygon(cx: Cx) -> FnResult {
    let arg = cx.arg(1);
    let coords = arg.get_table()?;
    let len = coords.len();
    if len < 6 || len % 2 != 0 {
        return Err(arg.error("polygon needs {x1, y1, x2, y2, x3, y3, ...}"));
    }
    let coord = |i: i64| match coords.get(i) {
        Value::Int(v) => Ok(v as f32),
        Value::Float(v) => Ok(f64::from(v) as f32),
        _ => Err(arg.error("polygon coordinates must be numbers")),
    };
    let points = (0..len / 2)
        .map(|p| Ok(Vec2::new(coord(p * 2 + 1)?, coord(p * 2 + 2)?)))
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    push(
        &cx,
        DrawCmd::Polygon {
            points,
            color: color(&cx, 2)?,
        },
    )?;
    Ok(cx.into())
}

fn text(cx: Cx) -> FnResult {
    let align_arg = cx.arg(6);
    let align = match align_arg.to_nilable_utf8(false)? {
        None => HAlign::Left,
        Some(name) => align_from_name(name)
            .ok_or_else(|| align_arg.error("align must be \"left\", \"center\" or \"right\""))?,
    };
    push(
        &cx,
        DrawCmd::Text {
            text: cx.arg(1).to_utf8()?.to_string(),
            pos: Vec2::new(num(&cx, 2)?, num(&cx, 3)?),
            size: num(&cx, 4)?,
            color: color(&cx, 5)?,
            align,
        },
    )?;
    Ok(cx.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(source: &str) -> Result<ScriptHost, String> {
        let (mut engine, _) = LuaEngine::load(source)?;
        let mut host = ScriptHost::new();
        engine.update(&mut host)?;
        Ok(host)
    }

    #[test]
    fn common_functions_are_available() {
        let host = run(r#"
            function update()
                assert(math.floor(3.7) == 3)
                assert(string.format("%d!", 7) == "7!")
                local t = {1, 2}
                table.insert(t, 5)
                assert(#t == 3 and t[3] == 5)
                table.insert(t, 1, 0)
                assert(#t == 4 and t[1] == 0 and t[4] == 5)
                assert(table.remove(t) == 5 and #t == 3)
                assert(table.remove(t, 1) == 0 and t[1] == 1 and #t == 2)
                table.sort(t, function(a, b) return a > b end)
                assert(t[1] == 2 and t[2] == 1)
                local f = math.random()
                assert(f >= 0 and f < 1)
                assert(rgb(255, 0, 16) == 0xff0010)
                assert(text_width("", 6) == 0)
                local r = math.random(3, 5)
                assert(r >= 3 and r <= 5)
                print("hi", 1, true)
            end
        "#)
        .unwrap();
        assert_eq!(host.logs, ["hi\t1\ttrue"]);
    }

    #[test]
    fn bad_arguments_are_errors() {
        for source in [
            "function update() is_down('enter') end",
            "function update() clear('red') end",
            "function update() polygon({1, 2, 3}, 0) end",
            "function update() text('hi', 0, 0, 6, 0, 'middle') end",
        ] {
            assert!(run(source).is_err(), "{source}");
        }
    }

    #[test]
    fn errors_have_line_numbers() {
        let error = run("function update()\n  local x = nil\n  x.y = 1\nend").unwrap_err();
        assert!(error.contains('3'), "{error}");
    }

    #[test]
    fn runaway_recursion_is_an_error() {
        let error = run("local function f(n) return 1 + f(n + 1) end function update() f(0) end")
            .unwrap_err();
        assert!(!error.is_empty());
    }
}
