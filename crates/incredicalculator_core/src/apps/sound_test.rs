use crate::{app::IcApp, input::IcKey};

pub struct SoundTest {
    active_note: Option<f32>
}

impl SoundTest {
    pub fn new() -> SoundTest {
        SoundTest { active_note: None }
    }
}

impl IcApp for SoundTest {
    fn on_enter(&mut self) {
        ()
    }

    fn on_key(&mut self, key: crate::input::IcKey, ctx: &crate::app::InputContext) {
        // actually, doing it here is not a good place to find when key is lifed and active note should go back to zero. go put this in update func instead.
        // mut n = match key {
        //     IcKey::Num0 => {
        //         self.active_note = 60.0
        //     }
        //     IcKey::Num1 => {
        //         self.active_note = 61.0
        //     }
        //     IcKey::Num2 => {
        //         self.active_note = 62.0
        //     }
        //     IcKey::Num3 => {
        //         self.active_note = 63.0
        //     }
        //     IcKey::Num4 => {
        //         self.active_note = 64.0
        //     }
        //     IcKey::Num5 => {
        //         self.active_note = 65.0
        //     }
        //     IcKey::Num6 => {
        //         self.active_note = 66.0
        //     }
        //     IcKey::Num7 => {
        //         self.active_note = 67.0
        //     }
        //     IcKey::Num8 => {
        //         self.active_note = 68.0
        //     }
        //     IcKey::Num9 => {
        //         self.active_note = 69.0
        //     }
        //     _ => ()
        // };
        // self.active_note = Some(n)
    }

    fn update(&mut self, platform: &mut dyn crate::platform::IcPlatform, ctx: &crate::app::InputContext) {
        todo!()
    }
}