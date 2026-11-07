use glam::Vec2;
use rgb::RGB8;

use crate::app::{IcApp, InputContext};
use crate::audio_engine::AudioEngine;
use crate::graphics::{self, Vitmap};
use crate::input::IcKey;
use crate::platform::{CANVAS_WIDTH, IcPlatform, UsbDriveStatus, rgb8_hex};
use crate::text::{TextStyle, draw_text, text_height, wrap_text};

const MARGIN: f32 = 10.0;
const BODY_SIZE: f32 = 5.5;
const LINE_GAP: f32 = 8.0;

/// Shares the platform's script drive with a computer over USB for as long
/// as it is open. Leaving it hands the drive back, and the shell reloads the
/// scripted apps.
pub struct UsbDrive;

impl UsbDrive {
    pub fn new() -> Self {
        Self
    }
}

impl Default for UsbDrive {
    fn default() -> Self {
        Self::new()
    }
}

/// Draws wrapped `text` at `y` and returns the `y` below it.
fn paragraph(platform: &mut dyn IcPlatform, text: &str, y: f32, color: RGB8) -> f32 {
    let wrapped = wrap_text(text, CANVAS_WIDTH as f32 - 2.0 * MARGIN, BODY_SIZE);
    let style = TextStyle::new(BODY_SIZE, color);
    draw_text(platform, &wrapped, Vec2::new(MARGIN, y), style);
    y + text_height(&wrapped, BODY_SIZE, style.font) + LINE_GAP
}

impl IcApp for UsbDrive {
    fn name(&self) -> &str {
        "USB Drive"
    }

    fn icon(&self) -> &'static Vitmap {
        &graphics::USB_DRIVE_ICON
    }

    fn color(&self) -> RGB8 {
        rgb8_hex(0x2E5E7E)
    }

    /// Redraws as the computer connects and ejects.
    fn requires_realtime_updates(&self) -> bool {
        true
    }

    fn on_enter(&mut self) {}

    fn on_key(&mut self, _key: IcKey, _ctx: &InputContext) {}

    fn shares_usb_drive(&self) -> bool {
        true
    }

    fn update(
        &mut self,
        platform: &mut dyn IcPlatform,
        _ctx: &InputContext,
        _audio: &mut AudioEngine,
    ) {
        let status = platform.usb_drive_status();
        platform.clear(rgb8_hex(0x10222E));
        draw_text(
            platform,
            "USB Drive",
            Vec2::new(MARGIN, 40.0),
            TextStyle::new(9.0, rgb8_hex(0xFFFFFF)).thickness(3.0),
        );
        let white = rgb8_hex(0xFFFFFF);
        let mut y = 62.0;
        let Some(status) = status else {
            paragraph(platform, "This calculator has no USB drive.", y, white);
            return;
        };
        let UsbDriveStatus {
            shared,
            connected,
            ejected,
            unformatted,
        } = status;
        if !shared {
            paragraph(platform, "Starting...", y, white);
            return;
        }
        if unformatted {
            y = paragraph(
                platform,
                "The drive can't be read. Format it (FAT) on your computer to use it.",
                y,
                rgb8_hex(0xFF9090),
            );
        }
        if !connected {
            paragraph(
                platform,
                "Connect the calculator to a computer with a USB cable.",
                y,
                white,
            );
        } else if ejected {
            paragraph(
                platform,
                "Ejected. It's safe to leave this app; your scripts will reload.",
                y,
                rgb8_hex(0x90FF90),
            );
        } else {
            y = paragraph(
                platform,
                "Connected. Add, edit or delete .lua files on the INCREDICALC drive, \
                 then eject it on your computer.",
                y,
                white,
            );
            paragraph(
                platform,
                "Not ejected yet: leaving now can lose changes your computer hasn't saved.",
                y,
                rgb8_hex(0xFFD060),
            );
        }
        draw_text(
            platform,
            "Shift+Super: leave and reload scripts",
            Vec2::new(MARGIN, 225.0),
            TextStyle::new(5.0, rgb8_hex(0x90A8B8)),
        );
    }
}
