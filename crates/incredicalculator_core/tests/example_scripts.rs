//! Loads every script in the repo's `example_scripts/` folder and runs it
//! for a few frames with some key presses, so a broken example is caught.
#![cfg(feature = "scripting")]

use std::path::Path;

use incredicalculator_core::input::IcKey;
use incredicalculator_core::platform::{MAX_SCRIPT_BYTES, SCRIPT_EXTENSION};
use incredicalculator_core::script::{ActiveEngine, ScriptEngine, ScriptHost};

const KEYS: [IcKey; 8] = [
    IcKey::Num1,
    IcKey::Num5,
    IcKey::Shift,
    IcKey::Num0,
    IcKey::Func1,
    IcKey::Func2,
    IcKey::Func3,
    IcKey::Func4,
];

#[test]
fn every_example_script_runs() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../example_scripts");
    let mut ran = 0;
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some(SCRIPT_EXTENSION) {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let source = std::fs::read_to_string(&path).unwrap();
        assert!(source.len() <= MAX_SCRIPT_BYTES, "{name} is too big");

        let mut host = ScriptHost::new();
        let (mut engine, _) =
            ActiveEngine::load(&source, &mut host).unwrap_or_else(|e| panic!("{name}: {e}"));
        let step = |result: Result<(), String>| {
            result.unwrap_or_else(|e| panic!("{name}: {e}"));
        };
        step(engine.init(&mut host));
        for (frame, key) in (0u64..).zip(KEYS) {
            step(engine.on_key(key, &mut host));
            host.millis = frame * 16;
            host.commands.clear();
            step(engine.update(&mut host));
            assert!(!host.commands.is_empty(), "{name} drew nothing");
        }
        ran += 1;
    }
    assert!(
        ran >= 2,
        "expected the example scripts in {}",
        dir.display()
    );
}
