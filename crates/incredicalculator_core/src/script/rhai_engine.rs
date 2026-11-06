//! [`ScriptEngine`] backed by Rhai.
//!
//! A script keeps its state in `this`, an object map that persists between
//! hook calls (Rhai functions cannot see top-level variables). See
//! `scripts/bouncer.rhai` for the API from the script's side.

use alloc::boxed::Box;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cell::RefCell;

use glam::{IVec2, Vec2};
use rhai::{
    AST, Array, CallFnOptions, Dynamic, Engine, EvalAltResult, FLOAT, INT, ImmutableString, Map,
    Scope,
};

use super::{
    DrawCmd, ScriptEngine, ScriptHost, ScriptMeta, align_from_name, color_from_int, key_from_name,
    key_name,
};
use crate::input::IcKey;
use crate::text::{HAlign, TextStyle, text_width};

/// Script operations allowed per hook call before it is stopped, so an
/// endless loop shows an error instead of freezing the calculator.
const MAX_OPERATIONS: u64 = 100_000;
/// Rhai recurses on the native stack, so nesting is kept shallow.
const MAX_CALL_LEVELS: usize = 12;
const MAX_EXPR_DEPTH: usize = 32;
const MAX_FN_EXPR_DEPTH: usize = 16;
const MAX_STRING_SIZE: usize = 1024;
const MAX_ARRAY_SIZE: usize = 1024;
const MAX_MAP_SIZE: usize = 256;

type HostCell = Rc<RefCell<ScriptHost>>;
type FnResult<T> = Result<T, Box<EvalAltResult>>;

pub struct RhaiEngine {
    engine: Engine,
    ast: AST,
    scope: Scope<'static>,
    /// The script's `this`.
    state: Dynamic,
    host: HostCell,
    has_init: bool,
    has_update: bool,
    has_on_key: bool,
}

impl RhaiEngine {
    /// Calls `name` with `this` bound to the script state, while the caller's
    /// host is swapped into the cell the native functions write to.
    fn call(
        &mut self,
        host: &mut ScriptHost,
        name: &str,
        args: impl rhai::FuncArgs,
    ) -> Result<(), String> {
        core::mem::swap(&mut *self.host.borrow_mut(), host);
        let options = CallFnOptions::new()
            .eval_ast(false)
            .rewind_scope(true)
            .bind_this_ptr(&mut self.state);
        let result = self.engine.call_fn_with_options::<Dynamic>(
            options,
            &mut self.scope,
            &self.ast,
            name,
            args,
        );
        core::mem::swap(&mut *self.host.borrow_mut(), host);
        result.map(|_| ()).map_err(|e| e.to_string())
    }
}

impl ScriptEngine for RhaiEngine {
    const FILE_EXTENSION: &'static str = "rhai";

    fn load(source: &str) -> Result<(Self, ScriptMeta), String> {
        let host: HostCell = Rc::new(RefCell::new(ScriptHost::new()));
        let engine = build_engine(&host);
        let ast = engine.compile(source).map_err(|e| e.to_string())?;
        let mut scope = Scope::new();
        engine
            .run_ast_with_scope(&mut scope, &ast)
            .map_err(|e| e.to_string())?;
        let meta = read_meta(&scope)?;

        let has_fn = |name: &str| ast.iter_functions().any(|f| f.name == name);
        let has_init = has_fn("init");
        let has_update = has_fn("update");
        let has_on_key = has_fn("on_key");
        Ok((
            Self {
                engine,
                ast,
                scope,
                state: Dynamic::from_map(Map::new()),
                host,
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
        self.call(host, "init", ())
    }

    fn update(&mut self, host: &mut ScriptHost) -> Result<(), String> {
        if !self.has_update {
            return Ok(());
        }
        self.call(host, "update", ())
    }

    fn on_key(&mut self, key: IcKey, host: &mut ScriptHost) -> Result<(), String> {
        if !self.has_on_key {
            return Ok(());
        }
        self.call(host, "on_key", (ImmutableString::from(key_name(key)),))
    }
}

/// Reads the optional top-level `app` map, e.g.
/// `let app = #{ name: "Bouncer", color: 0x2a6f97, realtime: true };`
fn read_meta(scope: &Scope) -> Result<ScriptMeta, String> {
    let mut meta = ScriptMeta::default();
    let Some(app) = scope.get("app") else {
        return Ok(meta);
    };
    let Some(app) = app.read_lock::<Map>() else {
        return Err("`app` must be an object map, like #{ name: \"My App\" }".into());
    };
    if let Some(name) = app.get("name") {
        meta.name = Some(
            name.clone()
                .into_string()
                .map_err(|_| String::from("app.name must be a string"))?,
        );
    }
    if let Some(color) = app.get("color") {
        meta.color = Some(
            color
                .as_int()
                .map_err(|_| String::from("app.color must be an integer like 0xRRGGBB"))?
                as u32,
        );
    }
    if let Some(realtime) = app.get("realtime") {
        meta.realtime = realtime
            .as_bool()
            .map_err(|_| String::from("app.realtime must be true or false"))?;
    }
    Ok(meta)
}

fn num(value: &Dynamic) -> FnResult<f32> {
    if let Ok(float) = value.as_float() {
        return Ok(float);
    }
    if let Ok(int) = value.as_int() {
        return Ok(int as f32);
    }
    Err(format!("expected a number, got {}", value.type_name()).into())
}

fn int(value: &Dynamic) -> FnResult<i32> {
    num(value).map(|n| n as i32)
}

fn color(value: &Dynamic) -> FnResult<rgb::RGB8> {
    value
        .as_int()
        .map(|c| color_from_int(c as i64))
        .map_err(|_| format!("expected a color like 0xRRGGBB, got {}", value.type_name()).into())
}

fn push(host: &HostCell, command: DrawCmd) -> FnResult<()> {
    host.borrow_mut().push(command).map_err(|e| e.into())
}

/// Builds an engine with resource limits and the calculator API registered.
///
/// Rhai's packages build their function tables on the heap when an engine is
/// created (`StandardPackage` alone is over 100 KB), so only the small ones
/// are used and a few common functions from the bigger ones are registered
/// by hand in [`register_stdlib`].
fn build_engine(host: &HostCell) -> Engine {
    use rhai::packages::{BasicIteratorPackage, BasicMathPackage, LanguageCorePackage, Package};
    let mut engine = Engine::new_raw();
    engine.register_global_module(LanguageCorePackage::new().as_shared_module());
    engine.register_global_module(BasicMathPackage::new().as_shared_module());
    engine.register_global_module(BasicIteratorPackage::new().as_shared_module());
    register_stdlib(&mut engine, host);
    engine
        .set_max_operations(MAX_OPERATIONS)
        .set_max_call_levels(MAX_CALL_LEVELS)
        .set_max_expr_depths(MAX_EXPR_DEPTH, MAX_FN_EXPR_DEPTH)
        .set_max_string_size(MAX_STRING_SIZE)
        .set_max_array_size(MAX_ARRAY_SIZE)
        .set_max_map_size(MAX_MAP_SIZE);

    let h = host.clone();
    engine.on_print(move |text| h.borrow_mut().log(text));
    engine.on_debug(|_, _, _| {});

    // ---- Inputs ----
    let h = host.clone();
    engine.register_fn("millis", move || h.borrow().millis as INT);
    let h = host.clone();
    engine.register_fn("is_down", move |key: ImmutableString| -> FnResult<bool> {
        let key = key_from_name(&key).ok_or_else(|| format!("unknown key \"{key}\""))?;
        Ok(h.borrow().keys_down[key as usize])
    });

    // ---- Helpers ----
    engine.register_fn(
        "rgb",
        |r: Dynamic, g: Dynamic, b: Dynamic| -> FnResult<INT> {
            let channel = |v: &Dynamic| int(v).map(|c| c.clamp(0, 255) as INT);
            Ok(channel(&r)? << 16 | channel(&g)? << 8 | channel(&b)?)
        },
    );
    engine.register_fn(
        "text_width",
        |text: ImmutableString, size: Dynamic| -> FnResult<FLOAT> {
            let style = TextStyle::new(num(&size)?, rgb::RGB8::default());
            Ok(text_width(&text, style.scale, style.font))
        },
    );

    // ---- Drawing ----
    let h = host.clone();
    engine.register_fn("clear", move |c: Dynamic| {
        push(&h, DrawCmd::Clear(color(&c)?))
    });

    let h = host.clone();
    let line = move |x1: Dynamic, y1: Dynamic, x2: Dynamic, y2: Dynamic, c: Dynamic, w: i32| {
        push(
            &h,
            DrawCmd::Line {
                start: Vec2::new(num(&x1)?, num(&y1)?),
                end: Vec2::new(num(&x2)?, num(&y2)?),
                color: color(&c)?,
                width: w.max(1) as u32,
            },
        )
    };
    let line2 = line.clone();
    engine.register_fn("line", move |x1, y1, x2, y2, c| line2(x1, y1, x2, y2, c, 2));
    engine.register_fn(
        "line",
        move |x1, y1, x2, y2, c, w: Dynamic| -> FnResult<()> { line(x1, y1, x2, y2, c, int(&w)?) },
    );

    let h = host.clone();
    let rect =
        move |x: Dynamic, y: Dynamic, w: Dynamic, ht: Dynamic, c: Dynamic, outline, radius| {
            let start = IVec2::new(int(&x)?, int(&y)?);
            push(
                &h,
                DrawCmd::Rect {
                    start,
                    end: start + IVec2::new(int(&w)?, int(&ht)?),
                    color: color(&c)?,
                    outline,
                    radius,
                },
            )
        };
    let r = rect.clone();
    engine.register_fn("rect", move |x, y, w, h, c| r(x, y, w, h, c, 0, 0));
    let r = rect.clone();
    engine.register_fn(
        "rect_outline",
        move |x, y, w, h, c, width: Dynamic| -> FnResult<()> {
            r(x, y, w, h, c, int(&width)?.max(1) as u32, 0)
        },
    );
    engine.register_fn(
        "round_rect",
        move |x, y, w, h, radius: Dynamic, c| -> FnResult<()> {
            rect(x, y, w, h, c, 0, int(&radius)?.max(0) as u32)
        },
    );

    let h = host.clone();
    engine.register_fn(
        "triangle",
        move |x1: Dynamic,
              y1: Dynamic,
              x2: Dynamic,
              y2: Dynamic,
              x3: Dynamic,
              y3: Dynamic,
              c: Dynamic|
              -> FnResult<()> {
            push(
                &h,
                DrawCmd::Triangle {
                    points: [
                        IVec2::new(int(&x1)?, int(&y1)?),
                        IVec2::new(int(&x2)?, int(&y2)?),
                        IVec2::new(int(&x3)?, int(&y3)?),
                    ],
                    color: color(&c)?,
                },
            )
        },
    );

    let h = host.clone();
    engine.register_fn(
        "polygon",
        move |coords: Array, c: Dynamic| -> FnResult<()> {
            if coords.len() < 6 || coords.len() % 2 != 0 {
                return Err("polygon needs [x1, y1, x2, y2, x3, y3, ...]".into());
            }
            let points = coords
                .chunks(2)
                .map(|p| Ok(Vec2::new(num(&p[0])?, num(&p[1])?)))
                .collect::<FnResult<Vec<_>>>()?;
            push(
                &h,
                DrawCmd::Polygon {
                    points,
                    color: color(&c)?,
                },
            )
        },
    );

    let h = host.clone();
    let text =
        move |s: ImmutableString, x: Dynamic, y: Dynamic, size: Dynamic, c: Dynamic, align| {
            push(
                &h,
                DrawCmd::Text {
                    text: s.to_string(),
                    pos: Vec2::new(num(&x)?, num(&y)?),
                    size: num(&size)?,
                    color: color(&c)?,
                    align,
                },
            )
        };
    let t = text.clone();
    engine.register_fn("text", move |s, x, y, size, c| {
        t(s, x, y, size, c, HAlign::Left)
    });
    engine.register_fn(
        "text",
        move |s, x, y, size, c, align: ImmutableString| -> FnResult<()> {
            let align = align_from_name(&align).ok_or_else(|| {
                format!("align must be \"left\", \"center\" or \"right\", got \"{align}\"")
            })?;
            text(s, x, y, size, c, align)
        },
    );

    engine
}

/// Integer if both are integers, otherwise float.
fn numeric_pair(
    a: &Dynamic,
    b: &Dynamic,
    on_int: fn(INT, INT) -> INT,
    on_float: fn(FLOAT, FLOAT) -> FLOAT,
) -> FnResult<Dynamic> {
    match (a.as_int(), b.as_int()) {
        (Ok(a), Ok(b)) => Ok(Dynamic::from_int(on_int(a, b))),
        _ => Ok(Dynamic::from_float(on_float(num(a)?, num(b)?))),
    }
}

/// Commonly needed functions from packages too big to register whole.
fn register_stdlib(engine: &mut Engine, host: &HostCell) {
    engine
        .register_fn("abs", |x: INT| x.wrapping_abs())
        .register_fn("abs", |x: FLOAT| x.abs())
        .register_fn("min", |a: Dynamic, b: Dynamic| {
            numeric_pair(&a, &b, INT::min, FLOAT::min)
        })
        .register_fn("max", |a: Dynamic, b: Dynamic| {
            numeric_pair(&a, &b, INT::max, FLOAT::max)
        })
        .register_fn("clamp", |x: Dynamic, lo: Dynamic, hi: Dynamic| {
            let low = numeric_pair(&x, &lo, INT::max, FLOAT::max)?;
            numeric_pair(&low, &hi, INT::min, FLOAT::min)
        })
        .register_fn("to_string", |x: Dynamic| x.to_string())
        .register_fn("len", |s: ImmutableString| s.chars().count() as INT)
        .register_fn("len", |a: &mut Array| a.len() as INT)
        .register_fn("push", |a: &mut Array, x: Dynamic| a.push(x))
        .register_fn("pop", |a: &mut Array| a.pop().unwrap_or(Dynamic::UNIT))
        .register_fn("clear", |a: &mut Array| a.clear())
        .register_fn("remove", |a: &mut Array, i: INT| {
            let i = i as usize;
            if i < a.len() {
                a.remove(i)
            } else {
                Dynamic::UNIT
            }
        });

    // xorshift32, seeded from the clock on first use.
    let seed = Rc::new(core::cell::Cell::new(0u32));
    let h = host.clone();
    let next = move || {
        let mut x = seed.get();
        if x == 0 {
            x = (h.borrow().millis as u32) | 1;
        }
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        seed.set(x);
        x
    };
    let n = next.clone();
    engine.register_fn("rand", move || (n() >> 8) as FLOAT / (1u32 << 24) as FLOAT);
    engine.register_fn("rand_int", move |lo: INT, hi: INT| -> FnResult<INT> {
        if hi < lo {
            return Err("rand_int needs lo <= hi".into());
        }
        let span = (hi as i64 - lo as i64 + 1) as u64;
        Ok((lo as i64 + (next() as u64 % span) as i64) as INT)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(source: &str) -> Result<Dynamic, String> {
        let host: HostCell = Rc::new(RefCell::new(ScriptHost::new()));
        build_engine(&host)
            .eval::<Dynamic>(source)
            .map_err(|e| e.to_string())
    }

    #[test]
    fn common_functions_are_available() {
        for (source, expected) in [
            ("let x = 3; `a${x}b`", "a3b"),
            ("[1, 2].len()", "2"),
            ("let a = [1]; a.push(5); a.pop()", "5"),
            ("let a = [1, 2, 3]; a.remove(0); a.len()", "2"),
            ("\"abc\".len()", "3"),
            ("(-2.5).abs()", "2.5"),
            ("abs(-3)", "3"),
            ("min(4, 2)", "2"),
            ("max(1, 2.5)", "2.5"),
            ("clamp(12, 0, 10)", "10"),
            ("clamp(-1.5, 0, 10)", "0.0"),
            ("7.to_string() + \"!\"", "7!"),
            ("let s = 0; for i in 0..5 { s += i; } s", "10"),
            ("sqrt(16.0)", "4.0"),
            ("let r = rand_int(3, 5); r >= 3 && r <= 5", "true"),
            ("let r = rand(); r >= 0.0 && r < 1.0", "true"),
            ("rgb(255, 0, 16)", "16711696"),
            ("text_width(\"\", 6) == 0.0", "true"),
        ] {
            let value = eval(source).unwrap_or_else(|e| panic!("{source}: {e}"));
            assert_eq!(value.to_string(), expected, "{source}");
        }
    }

    #[test]
    fn bad_arguments_are_errors() {
        assert!(eval("is_down(\"enter\")").is_err());
        assert!(eval("clear(\"red\")").is_err());
        assert!(eval("polygon([1, 2, 3], 0)").is_err());
        assert!(eval("text(\"hi\", 0, 0, 6, 0, \"middle\")").is_err());
    }
}
