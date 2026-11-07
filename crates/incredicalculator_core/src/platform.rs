use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use glam::{IVec2, Vec2};
use rgb::*;

pub const CANVAS_WIDTH: u32 = 320;
pub const CANVAS_HEIGHT: u32 = 240;
/// Size of the opaque settings blob a platform persists between sessions.
pub const SETTINGS_STORAGE_LEN: usize = 9;
/// File extension of scripted apps (Lua), without the dot.
pub const SCRIPT_EXTENSION: &str = "lua";
/// Largest script file a platform should store, in bytes.
pub const MAX_SCRIPT_BYTES: usize = 16 * 1024;

pub trait IcPlatform {
    fn draw_line(&mut self, start: Vec2, end: Vec2, color: RGB8, width: u32);
    /// Fills a simple polygon. Points may be clockwise or counterclockwise;
    /// concave must be supported, but not holes nor self-intersections.
    fn draw_polygon(&mut self, points: &[Vec2], fill_color: RGB8);
    fn draw_rectangle(
        &mut self,
        start: IVec2,
        end: IVec2,
        stroke_color: RGB8,
        stroke_width: u32,
        fill_color: Option<RGB8>,
    );
    fn draw_rectangle_rounded(
        &mut self,
        start: IVec2,
        end: IVec2,
        stroke_color: rgb::RGB8,
        stroke_width: u32,
        fill_color: Option<rgb::RGB8>,
        corner_radius: u32,
    );
    fn draw_triangle(
        &mut self,
        vertex1: IVec2,
        vertex2: IVec2,
        vertex3: IVec2,
        stroke_color: RGB8,
        stroke_width: u32,
        fill_color: Option<RGB8>,
    );
    fn clear(&mut self, color: RGB8);
    fn log(&mut self, arg: fmt::Arguments);
    fn millis(&self) -> u64;
    fn get_battery_soc(&self) -> Option<i32>;
    fn is_charging(&self) -> bool {
        false
    }
    fn get_brightness(&self) -> u8;
    fn set_brightness(&mut self, value: u8);
    fn get_volume(&self) -> u8;
    fn set_volume(&mut self, value: u8);
    fn get_screen_timeout_seconds(&self) -> u32;
    fn set_screen_timeout_seconds(&mut self, seconds: u32);
    /// Returns the blob last passed to `save_settings`, if anything was saved.
    /// Platforms without persistent storage can keep these defaults.
    fn load_settings(&mut self) -> Option<[u8; SETTINGS_STORAGE_LEN]> {
        None
    }
    fn save_settings(&mut self, _data: &[u8; SETTINGS_STORAGE_LEN]) {}
    /// Changes whenever installed scripts are added, removed or edited, so the
    /// shell knows to reload its scripted apps. Any change in value works.
    fn scripts_version(&self) -> u32 {
        0
    }
    /// File names (like `bouncer.lua`) of the installed scripts, or `None`
    /// when the platform has no script storage, in which case the shell shows
    /// the example script bundled into the firmware instead.
    fn list_scripts(&mut self) -> Option<Vec<String>> {
        None
    }
    /// Source of an installed script, by a file name from `list_scripts`.
    fn read_script(&mut self, _file_name: &str) -> Option<String> {
        None
    }
    /// State of the USB drive that holds installed scripts, or `None` if the
    /// platform has none. Platforms with one get a "USB Drive" app.
    fn usb_drive_status(&self) -> Option<UsbDriveStatus> {
        None
    }
    /// Shares the script drive with a computer over USB (`true`), or takes it
    /// back (`false`). While shared, the platform doesn't read the drive
    /// itself. Taking it back changes `scripts_version`, once the drive is
    /// safe to read again.
    fn set_usb_drive_mode(&mut self, _shared: bool) {}
    /// RAM usage, for platforms with a fixed memory budget. Shown by the
    /// shell when the "Show memory" setting is on.
    fn memory_stats(&self) -> Option<MemoryStats> {
        None
    }
}

/// What a computer is doing with the script drive, from
/// [`IcPlatform::usb_drive_status`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UsbDriveStatus {
    /// The drive is being offered over USB.
    pub shared: bool,
    /// A computer has set up the USB connection.
    pub connected: bool,
    /// The computer ejected the drive, so its writes have all been saved.
    pub ejected: bool,
    /// The drive holds no readable filesystem. The computer can format it.
    pub unformatted: bool,
}

/// Memory use in bytes. Stack figures are high-water marks: the deepest each
/// core's stack has reached since boot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MemoryStats {
    pub heap_used: usize,
    pub heap_peak: usize,
    pub heap_size: usize,
    pub stack_peaks: [usize; 2],
    pub stack_sizes: [usize; 2],
}

impl MemoryStats {
    /// The highest peak-to-size ratio across the heap and stacks, 0.0..=1.0.
    pub fn worst_fraction(&self) -> f32 {
        let fraction = |used: usize, size: usize| {
            if size == 0 {
                0.0
            } else {
                used as f32 / size as f32
            }
        };
        let mut worst = fraction(self.heap_peak, self.heap_size);
        for (peak, size) in self.stack_peaks.iter().zip(self.stack_sizes) {
            worst = worst.max(fraction(*peak, size));
        }
        worst
    }
}

#[macro_export]
macro_rules! _platform_debug_log {
    ($platform:expr, $($arg:tt)*) => {
        $platform.log(format_args!($($arg)*))
    };
}
pub use _platform_debug_log as debug_log;

pub const fn rgb8_hex(c: u32) -> RGB8 {
    let r = ((c >> 16) & 0xFF) as u8;
    let g = ((c >> 8) & 0xFF) as u8;
    let b = (c & 0xFF) as u8;
    RGB8::new(r, g, b)
}
