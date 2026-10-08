use serde_json::Value;
use std::collections::BTreeSet;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let assets_dir = manifest_dir.join("assets");
    println!("cargo:rerun-if-changed={}", assets_dir.display());

    let mut graphics_files = fs::read_dir(&assets_dir)
        .unwrap_or_else(|error| panic!("could not read {}: {error}", assets_dir.display()))
        .map(|entry| {
            entry
                .expect("could not read an asset directory entry")
                .path()
        })
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
        })
        .collect::<Vec<_>>();
    graphics_files.sort();
    if graphics_files.is_empty() {
        panic!("no vitmap JSON files found in {}", assets_dir.display());
    }

    let mut generated = generated_header();
    let mut identifiers = BTreeSet::new();
    let mut vitmaps = Vec::new();
    for path in graphics_files {
        println!("cargo:rerun-if-changed={}", path.display());
        let identifier = rust_identifier(&path)
            .unwrap_or_else(|error| panic!("cannot name vitmap from {}: {error}", path.display()));
        if !identifiers.insert(identifier.clone()) {
            panic!("multiple vitmap files map to the Rust name `{identifier}`");
        }

        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("could not read {}: {error}", path.display()));
        let json: Value = serde_json::from_str(&source)
            .unwrap_or_else(|error| panic!("invalid vitmap JSON at {}: {error}", path.display()));
        generate_vitmap(&mut generated, &identifier, &json)
            .unwrap_or_else(|error| panic!("invalid vitmap JSON at {}: {error}", path.display()));
        vitmaps.push(identifier);
    }
    generated.push_str("pub static VITMAPS: &[&Vitmap] = &[\n");
    for vitmap in vitmaps {
        writeln!(generated, "    &{vitmap},").unwrap();
    }
    generated.push_str("];\n");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(out_dir.join("generated_graphics.rs"), generated)
        .expect("could not write generated_graphics.rs");
}

fn generated_header() -> String {
    String::from(
        "// Generated from assets/*.json by incredicalculator_core/build.rs.\n\
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
         #[derive(Clone, Copy, Debug)]\n\
         pub struct Vitmap { pub name: &'static str, pub version: &'static str, pub actions: &'static [Action] }\n",
    )
}

fn rust_identifier(path: &Path) -> Result<String, String> {
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or("file name must be valid Unicode")?;
    let mut identifier = String::new();
    let mut needs_separator = false;
    for character in stem.chars() {
        if character.is_ascii_alphanumeric() {
            if needs_separator && !identifier.is_empty() {
                identifier.push('_');
            }
            identifier.push(character.to_ascii_uppercase());
            needs_separator = false;
        } else {
            needs_separator = true;
        }
    }
    if identifier.is_empty() {
        return Err("file name has no letters or digits".into());
    }
    if identifier.as_bytes()[0].is_ascii_digit() {
        identifier.insert_str(0, "VITMAP_");
    }
    Ok(identifier)
}

fn generate_vitmap(output: &mut String, identifier: &str, root: &Value) -> Result<(), String> {
    let actions = field(root, "actions")?
        .as_array()
        .ok_or("actions must be an array")?;
    let version = root
        .get("meta")
        .and_then(|meta| meta.get("version"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let name = identifier.to_ascii_lowercase();
    writeln!(
        output,
        "pub static {identifier}: Vitmap = Vitmap {{ name: {name:?}, version: {version:?}, actions: &["
    )
    .unwrap();

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
            "    Action {{ name: {name:?}, looped: {looped}, frames: &["
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
    output.push_str("] };\n\n");
    Ok(())
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
