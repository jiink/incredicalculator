use super::*;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::fmt;

type RecordedKeys = Rc<RefCell<Vec<(IcKey, bool, bool)>>>;

#[derive(Default)]
struct TestPlatform {
    rectangles: Vec<(IVec2, IVec2, RGB8, u32, Option<RGB8>)>,
}

impl IcPlatform for TestPlatform {
    fn draw_line(&mut self, _: Vec2, _: Vec2, _: RGB8, _: u32) {}
    fn draw_polygon(&mut self, _: &[Vec2], _: RGB8) {}
    fn draw_rectangle(
        &mut self,
        start: IVec2,
        end: IVec2,
        stroke: RGB8,
        width: u32,
        fill: Option<RGB8>,
    ) {
        self.rectangles.push((start, end, stroke, width, fill));
    }
    fn draw_rectangle_rounded(
        &mut self,
        _: IVec2,
        _: IVec2,
        _: RGB8,
        _: u32,
        _: Option<RGB8>,
        _: u32,
    ) {
    }
    fn draw_triangle(&mut self, _: IVec2, _: IVec2, _: IVec2, _: RGB8, _: u32, _: Option<RGB8>) {}
    fn clear(&mut self, _: RGB8) {
        self.rectangles.clear();
    }
    fn log(&mut self, _: fmt::Arguments) {}
    fn millis(&self) -> u64 {
        0
    }
    fn get_battery_soc(&self) -> Option<i32> {
        None
    }
    fn get_brightness(&self) -> u8 {
        128
    }
    fn set_brightness(&mut self, _: u8) {}
    fn get_volume(&self) -> u8 {
        128
    }
    fn set_volume(&mut self, _: u8) {}
    fn get_screen_timeout_seconds(&self) -> u32 {
        60
    }
    fn set_screen_timeout_seconds(&mut self, _: u32) {}
}

struct RecordingApp(RecordedKeys);

impl IcApp for RecordingApp {
    fn name(&self) -> &str {
        "Test"
    }
    fn icon(&self) -> &'static Vitmap {
        &graphics::CODE_ICON
    }
    fn color(&self) -> RGB8 {
        rgb8_hex(0)
    }
    fn requires_realtime_updates(&self) -> bool {
        false
    }
    fn on_enter(&mut self) {}
    fn on_key(&mut self, key: IcKey, ctx: &InputContext) {
        self.0
            .borrow_mut()
            .push((key, ctx.is_shifted(), ctx.is_super()));
    }
    fn update(&mut self, platform: &mut dyn IcPlatform, _: &InputContext, _: &mut AudioEngine) {
        platform.clear(rgb8_hex(0));
    }
}

fn tap(shell: &mut IcShell, platform: &mut TestPlatform, key: IcKey) {
    shell.key_down(key);
    shell.update(platform);
    shell.key_up(key);
    shell.update(platform);
}

fn setup(mode: KeyMode) -> (IcShell, TestPlatform, RecordedKeys) {
    let mut shell = IcShell::new();
    let mut platform = TestPlatform::default();
    let states = [KeyState::default(); IcKey::COUNT];
    let ctx = InputContext {
        key_states: &states,
        key_mode: KeyMode::Normal,
    };
    shell
        .settings_menu
        .handle_key(IcKey::Num2, &ctx, &mut platform);
    while shell.settings_menu.key_mode() != mode {
        shell
            .settings_menu
            .handle_key(IcKey::Func6, &ctx, &mut platform);
    }
    let recorded = Rc::new(RefCell::new(Vec::new()));
    shell.apps[0] = Box::new(RecordingApp(recorded.clone()));
    (shell, platform, recorded)
}

fn has_border(platform: &TestPlatform, inset: i32, color: u32) -> bool {
    platform.rectangles.contains(&(
        IVec2::splat(inset),
        IVec2::new(
            CANVAS_WIDTH as i32 - 1 - inset,
            CANVAS_HEIGHT as i32 - 1 - inset,
        ),
        rgb8_hex(color),
        2,
        None,
    ))
}

#[test]
fn settings_enter_cycles_all_modes_without_entering_text_editor() {
    let (mut shell, mut platform, _) = setup(KeyMode::Normal);
    shell.active_app_idx = None;
    shell.settings_open = true;
    for mode in [KeyMode::Sticky, KeyMode::Toggle, KeyMode::Normal] {
        tap(&mut shell, &mut platform, IcKey::Func6);
        assert_eq!(shell.settings_menu.key_mode(), mode);
    }
    // Backspace closes settings without also leaving the apps menu.
    tap(&mut shell, &mut platform, IcKey::Func1);
    assert!(!shell.settings_open);
    assert!(shell.active_app_idx.is_none());
}

#[test]
fn sticky_and_toggle_shift_reach_apps_and_show_blue_indicator() {
    for mode in [KeyMode::Sticky, KeyMode::Toggle] {
        let (mut shell, mut platform, recorded) = setup(mode);
        tap(&mut shell, &mut platform, IcKey::Shift);
        assert!(has_border(&platform, 1, 0x0000ff));
        tap(&mut shell, &mut platform, IcKey::Num1);
        assert!(recorded.borrow().contains(&(IcKey::Num1, true, false)));
        assert_eq!(has_border(&platform, 1, 0x0000ff), mode == KeyMode::Toggle);
        tap(&mut shell, &mut platform, IcKey::Num2);
        assert!(
            recorded
                .borrow()
                .contains(&(IcKey::Num2, mode == KeyMode::Toggle, false))
        );
        if mode == KeyMode::Toggle {
            tap(&mut shell, &mut platform, IcKey::Shift);
            assert!(!has_border(&platform, 1, 0x0000ff));
        }
    }
}

#[test]
fn latched_super_shows_green_and_provides_menu_access_without_app_backspace() {
    for mode in [KeyMode::Sticky, KeyMode::Toggle] {
        let (mut shell, mut platform, recorded) = setup(mode);
        tap(&mut shell, &mut platform, IcKey::Super);
        assert_eq!(shell.active_app_idx, Some(0));
        assert!(has_border(&platform, 1, 0x00ff00));
        tap(&mut shell, &mut platform, IcKey::Func1);
        assert!(shell.active_app_idx.is_none());
        assert!(
            !recorded
                .borrow()
                .iter()
                .any(|(key, _, _)| *key == IcKey::Func1)
        );
        tap(&mut shell, &mut platform, IcKey::Func1);
        assert_eq!(shell.active_app_idx, Some(0));
    }
}

#[test]
fn two_super_taps_open_and_close_menu_in_latching_modes() {
    for mode in [KeyMode::Sticky, KeyMode::Toggle] {
        let (mut shell, mut platform, _) = setup(mode);
        for expected_app in [None, Some(0)] {
            let previous_app = shell.active_app_idx;
            tap(&mut shell, &mut platform, IcKey::Super);
            assert_eq!(shell.active_app_idx, previous_app);
            assert!(has_border(&platform, 1, 0x00ff00));
            shell.key_down(IcKey::Super);
            shell.update(&mut platform);
            assert_eq!(shell.active_app_idx, previous_app);
            shell.key_up(IcKey::Super);
            shell.update(&mut platform);
            assert_eq!(shell.active_app_idx, expected_app);
            assert!(!has_border(&platform, 1, 0x00ff00));
        }
    }
}

#[test]
fn toggle_super_used_between_taps_does_not_open_menu_when_disengaged() {
    for key in [IcKey::Num1, IcKey::Shift, IcKey::Func5, IcKey::Func6] {
        let (mut shell, mut platform, _) = setup(KeyMode::Toggle);
        tap(&mut shell, &mut platform, IcKey::Super);
        tap(&mut shell, &mut platform, key);
        tap(&mut shell, &mut platform, IcKey::Super);
        assert_eq!(shell.active_app_idx, Some(0), "interrupting key: {key:?}");
        assert!(!has_border(&platform, 1, 0x00ff00));
        // A fresh uninterrupted pair must still open the menu.
        tap(&mut shell, &mut platform, IcKey::Super);
        tap(&mut shell, &mut platform, IcKey::Super);
        assert!(shell.active_app_idx.is_none());
    }
}

#[test]
fn toggle_super_used_between_taps_does_not_close_menu_when_disengaged() {
    let (mut shell, mut platform, _) = setup(KeyMode::Toggle);
    shell.active_app_idx = None;
    shell.last_active_app_idx = Some(0);
    tap(&mut shell, &mut platform, IcKey::Super);
    tap(&mut shell, &mut platform, IcKey::Shift);
    tap(&mut shell, &mut platform, IcKey::Super);
    assert!(shell.active_app_idx.is_none());
}

#[test]
fn interrupted_super_release_does_not_open_menu_in_latching_modes() {
    for mode in [KeyMode::Sticky, KeyMode::Toggle] {
        let (mut shell, mut platform, _) = setup(mode);
        tap(&mut shell, &mut platform, IcKey::Super);
        shell.key_down(IcKey::Super);
        shell.update(&mut platform);
        tap(&mut shell, &mut platform, IcKey::Num1);
        shell.key_up(IcKey::Super);
        shell.update(&mut platform);
        assert_eq!(shell.active_app_idx, Some(0));
    }
}

#[test]
fn both_modifier_borders_are_visible_without_filling_the_screen() {
    let (mut shell, mut platform, _) = setup(KeyMode::Sticky);
    tap(&mut shell, &mut platform, IcKey::Shift);
    tap(&mut shell, &mut platform, IcKey::Super);
    assert!(has_border(&platform, 1, 0x0000ff));
    assert!(has_border(&platform, 3, 0x00ff00));
}

#[test]
fn normal_super_tap_and_menu_backspace_work_but_app_backspace_stays_in_app() {
    let (mut shell, mut platform, recorded) = setup(KeyMode::Normal);
    tap(&mut shell, &mut platform, IcKey::Func1);
    assert_eq!(shell.active_app_idx, Some(0));
    assert!(recorded.borrow().contains(&(IcKey::Func1, false, false)));
    tap(&mut shell, &mut platform, IcKey::Super);
    assert!(shell.active_app_idx.is_none());
    tap(&mut shell, &mut platform, IcKey::Func1);
    assert_eq!(shell.active_app_idx, Some(0));
    tap(&mut shell, &mut platform, IcKey::Shift);
    assert!(!has_border(&platform, 1, 0x0000ff));
}

#[test]
fn changing_mode_in_settings_clears_active_modifiers() {
    let (mut shell, mut platform, _) = setup(KeyMode::Toggle);
    tap(&mut shell, &mut platform, IcKey::Shift);
    assert!(has_border(&platform, 1, 0x0000ff));
    shell.active_app_idx = None;
    shell.settings_open = true;
    tap(&mut shell, &mut platform, IcKey::Func6);
    assert_eq!(shell.settings_menu.key_mode(), KeyMode::Normal);
    assert!(!has_border(&platform, 1, 0x0000ff));
    tap(&mut shell, &mut platform, IcKey::Func6);
    assert_eq!(shell.settings_menu.key_mode(), KeyMode::Sticky);
    assert!(!has_border(&platform, 1, 0x0000ff));
}

#[test]
fn normal_super_chord_does_not_toggle_menu_on_release() {
    let (mut shell, mut platform, recorded) = setup(KeyMode::Normal);
    shell.key_down(IcKey::Super);
    shell.update(&mut platform);
    tap(&mut shell, &mut platform, IcKey::Num1);
    shell.key_up(IcKey::Super);
    shell.update(&mut platform);
    assert_eq!(shell.active_app_idx, Some(0));
    assert!(recorded.borrow().contains(&(IcKey::Num1, false, true)));
}

#[test]
fn shell_consumed_chord_uses_up_sticky_super() {
    let (mut shell, mut platform, recorded) = setup(KeyMode::Sticky);
    tap(&mut shell, &mut platform, IcKey::Super);
    tap(&mut shell, &mut platform, IcKey::Func6);
    assert!(matches!(
        shell.adjusting_something,
        Some(Adjustable::Brightness)
    ));
    assert!(
        !recorded
            .borrow()
            .iter()
            .any(|(key, _, _)| *key == IcKey::Func6)
    );
    assert!(!has_border(&platform, 1, 0x00ff00));
}
