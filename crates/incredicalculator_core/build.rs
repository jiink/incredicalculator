use serde_json::Value;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let graphics_path = match env::var_os("IC_GRAPHICS_JSON") {
        Some(path) => PathBuf::from(path),
        None => manifest_dir.join("assets").join("graphics.json"),
    };
    let graphics_path = if graphics_path.is_absolute() {
        graphics_path
    } else {
        manifest_dir.join(graphics_path)
    };
    println!("cargo:rerun-if-env-changed=IC_GRAPHICS_JSON");
    println!("cargo:rerun-if-changed={}", graphics_path.display());

    let source = fs::read_to_string(&graphics_path).unwrap_or_else(|error| {
        panic!(
            "could not read graphics JSON at {}: {error}",
            graphics_path.display()
        )
    });
    let json: Value = serde_json::from_str(&source).unwrap_or_else(|error| {
        panic!(
            "invalid graphics JSON at {}: {error}",
            graphics_path.display()
        )
    });
    let generated = generate(&json).unwrap_or_else(|error| {
        panic!(
            "invalid graphics JSON at {}: {error}",
            graphics_path.display()
        )
    });
    fs::write(out_dir.join("generated_graphics.rs"), generated)
        .expect("could not write generated_graphics.rs");
}

fn generate(root: &Value) -> Result<String, String> {
    let actions = field(root, "actions")?
        .as_array()
        .ok_or("actions must be an array")?;
    let version = root
        .get("meta")
        .and_then(|meta| meta.get("version"))
        .and_then(Value::as_str)
        .unwrap_or("");

    let mut output = String::from(
        "// Generated from graphics JSON by incredicalculator_core/build.rs.\n\
         #[derive(Clone, Copy, Debug)]\n\
         pub struct Color { pub r: u8, pub g: u8, pub b: u8, pub a: u8 }\n\
         #[derive(Clone, Copy, Debug)]\n\
         pub struct Point { pub x: f32, pub y: f32 }\n\
         #[derive(Clone, Copy, Debug)]\n\
         pub struct Polygon { pub points: &'static [Point], pub color: Color, pub border_width: f32, pub border_color: Color, pub open: bool }\n\
         #[derive(Clone, Copy, Debug)]\n\
         pub struct Frame { pub shapes: &'static [Polygon] }\n\
         #[derive(Clone, Copy, Debug)]\n\
         pub struct Action { pub name: &'static str, pub frames: &'static [Frame], pub looped: bool }\n\
         pub const GRAPHICS_VERSION: &str = ",
    );
    writeln!(output, "{version:?};").unwrap();
    output.push_str("pub const GRAPHICS: &[Action] = &[\n");

    for action in actions {
        let name = field(action, "name")?
            .as_str()
            .ok_or("action name must be a string")?;
        let looped = field(action, "loop")?
            .as_bool()
            .ok_or("action loop must be a boolean")?;
        let frames = field(action, "frames")?
            .as_array()
            .ok_or("action frames must be an array")?;
        writeln!(
            output,
            "    Action {{ name: {name:?}, looped: {looped}, frames: &[ "
        )
        .unwrap();

        for frame in frames {
            let shapes = field(frame, "shapes")?
                .as_array()
                .ok_or("frame shapes must be an array")?;
            output.push_str("        Frame { shapes: &[\n");
            for shape in shapes {
                let points = field(shape, "points")?
                    .as_array()
                    .ok_or("shape points must be an array")?;
                if points.len() < 2 {
                    return Err("a shape must contain at least two points".into());
                }
                let fill_color = color(field(shape, "color")?)?;
                let border_color = color(field(shape, "borderColor")?)?;
                let border_width = float(field(shape, "borderWidth")?, "borderWidth")?;
                if border_width < 0.0 {
                    return Err("borderWidth cannot be negative".into());
                }
                let open = field(shape, "open")?
                    .as_bool()
                    .ok_or("shape open must be a boolean")?;

                output.push_str("            Polygon { points: &[\n");
                for point in points {
                    let x = float(field(point, "x")?, "point x")?;
                    let y = float(field(point, "y")?, "point y")?;
                    writeln!(
                        output,
                        "                Point {{ x: {x:?}_f32, y: {y:?}_f32 }},"
                    )
                    .unwrap();
                }
                writeln!(
                    output,
                    "            ], color: {}, border_width: {border_width:?}_f32, border_color: {}, open: {open} }},",
                    color_literal(fill_color),
                    color_literal(border_color),
                )
                .unwrap();
            }
            output.push_str("        ] },\n");
        }
        output.push_str("    ] },\n");
    }
    output.push_str("];\n");
    Ok(output)
}

fn field<'a>(value: &'a Value, name: &str) -> Result<&'a Value, String> {
    value
        .get(name)
        .ok_or_else(|| format!("missing required field `{name}`"))
}

fn float(value: &Value, description: &str) -> Result<f32, String> {
    let number = value
        .as_f64()
        .ok_or_else(|| format!("{description} must be a number"))?;
    let number = number as f32;
    if !number.is_finite() {
        return Err(format!("{description} must fit in an f32"));
    }
    Ok(number)
}

fn color(value: &Value) -> Result<[u8; 4], String> {
    let component = |name: &str, default: Option<u8>| -> Result<u8, String> {
        let Some(value) = value.get(name) else {
            return default.ok_or_else(|| format!("missing required color component `{name}`"));
        };
        let component = value
            .as_u64()
            .ok_or_else(|| format!("color component `{name}` must be an integer from 0 to 255"))?;
        u8::try_from(component)
            .map_err(|_| format!("color component `{name}` must be from 0 to 255"))
    };
    Ok([
        component("r", None)?,
        component("g", None)?,
        component("b", None)?,
        component("a", Some(255))?,
    ])
}

fn color_literal(color: [u8; 4]) -> String {
    format!(
        "Color {{ r: {}, g: {}, b: {}, a: {} }}",
        color[0], color[1], color[2], color[3]
    )
}
