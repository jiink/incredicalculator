use crate::platform::IcPlatform;
use crate::input::{IcKey, KeyState};

pub struct InputContext<'a> {
    pub key_states: &'a [KeyState; IcKey::COUNT]
}

impl<'a> InputContext<'a> {
    pub fn is_down(&self, key: IcKey) -> bool {
        self.key_states[key as usize].is_down
    }
    pub fn is_shifted(&self) -> bool {
        self.key_states[IcKey::Shift as usize].is_down
    }
    pub fn is_super(&self) -> bool {
        self.key_states[IcKey::Super as usize].is_down
    }
}

pub trait IcApp {
    fn on_enter(&mut self);
    fn on_key(&mut self, key: IcKey, ctx: &InputContext);
    fn update(&mut self, platform: &mut dyn IcPlatform, ctx: &InputContext);
}

