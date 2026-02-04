#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, FromPrimitive, ToPrimitive)]
#[repr(usize)]
pub enum IcKey {
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Func1,
    Func2,
    Func3,
    Func4,
    Func5,
    Func6,
    Shift,
    Super,
    _Max,
}

impl IcKey {
    pub const COUNT: usize = IcKey::_Max as usize;
}

#[derive(Clone, Copy)]
pub struct KeyState {
    pub is_down: bool,
    pub was_down: bool,
    pub just_pressed: bool,
    pub just_released: bool,
}



impl Default for KeyState {
    fn default() -> Self {
        KeyState {
            is_down: false,
            was_down: false,
            just_pressed: false,
            just_released: false,
        }
    }
}