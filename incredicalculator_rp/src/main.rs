#![no_std]
#![no_main]

extern crate alloc;

use core::cell::RefCell;

use defmt::*;
use display_interface_spi::SPIInterface;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_executor::Spawner;
use embassy_futures::select::select_array;
use embassy_rp::gpio::{Input, Level, Output};
use embassy_rp::rom_data;
use embassy_rp::spi;
use embassy_rp::spi::Spi;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::Delay;
use embassy_time::Timer;
use embedded_alloc::LlffHeap as Heap;
use embedded_graphics::image::{Image, ImageRawLE};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Rectangle};
use embedded_graphics::text::Text;
use embedded_graphics::{prelude::*, primitives};
use embedded_graphics_framebuf::FrameBuf;
use incredicalculator_core::input::IcKey;
use incredicalculator_core::platform::{IcPlatform, Shape};
use incredicalculator_core::shell::IcShell;
use mipidsi::Builder;
use mipidsi::models::ST7789;
use mipidsi::options::{Orientation, Rotation};
use {defmt_rtt as _, panic_probe as _};

const DISPLAY_FREQ: u32 = 2_000_000;

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[unsafe(link_section = ".uninit.HEAP_MEM")]
static mut HEAP_MEM: [u8; 64_000] = [0; 64_000];

#[derive(Copy, Clone)]
struct Line {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

impl Line {
    pub fn new() -> Line {
        Line {
            x1: 0.0,
            y1: 0.0,
            x2: 0.0,
            y2: 0.0,
        }
    }
}

pub struct IcRpPlatform {
    line_list: [Line; 100],
    line_idx: usize,
}

impl IcRpPlatform {
    pub fn new() -> IcRpPlatform {
        IcRpPlatform {
            line_list: [Line::new(); 100],
            line_idx: 0,
        }
    }
}

impl IcPlatform for IcRpPlatform {
    fn draw_shape(&mut self, shape: Shape) {
        self.line_list[self.line_idx] = Line {
            x1: shape.start.x as f32,
            y1: shape.start.y as f32,
            x2: shape.end.x as f32,
            y2: shape.end.y as f32,
        };
        if self.line_idx + 1 < self.line_list.len() {
            self.line_idx += 1;
        }
    }
    fn clear_lines(&mut self) {
        self.line_idx = 0;
    }
}

//static mut DATA: [Rgb565; 320 * 240] = [Rgb565::GREEN; 320 * 240];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    unsafe {
        // Get the raw address of the mutable static without creating a shared reference
        let base = core::ptr::addr_of_mut!(HEAP_MEM) as usize;
        // Align up to 8 bytes (RP2040/cortex-m alignment)
        let aligned = (base + 7) & !7;
        // Compute how many bytes we lost to alignment adjustment
        let adjust = aligned - base;
        // Total size of the static buffer (compile-time)
        let total = core::mem::size_of::<[u8; 64_000]>();
        // Remaining usable bytes after alignment
        let usable = total - adjust;
        HEAP.init(aligned, usable);
    }
    let p = embassy_rp::init(Default::default());
    info!("Hello World!");
    let mut test_str = alloc::string::String::from("test");
    info!(
        "alloc works: (len {}) - \"{}\"",
        test_str.len(),
        test_str.as_str()
    );
    let mut btn0 = Input::new(p.PIN_26, embassy_rp::gpio::Pull::Up);
    let mut btn1 = Input::new(p.PIN_12, embassy_rp::gpio::Pull::Up);
    let mut btn2 = Input::new(p.PIN_11, embassy_rp::gpio::Pull::Up);
    let mut btn3 = Input::new(p.PIN_2, embassy_rp::gpio::Pull::Up);
    let mut btn4 = Input::new(p.PIN_1, embassy_rp::gpio::Pull::Up);
    let mut btnl = Input::new(p.PIN_27, embassy_rp::gpio::Pull::Up);
    let mut btnr = Input::new(p.PIN_0, embassy_rp::gpio::Pull::Up);
    let mut led = Output::new(p.PIN_25, Level::Low);
    let rst = p.PIN_4;
    let display_cs = p.PIN_5;
    let dcx = p.PIN_8;
    let mosi = p.PIN_7;
    let clk = p.PIN_6;

    // create SPI
    let mut display_config = spi::Config::default();
    display_config.frequency = DISPLAY_FREQ;
    display_config.phase = spi::Phase::CaptureOnSecondTransition;
    display_config.polarity = spi::Polarity::IdleHigh;

    let spi = Spi::new_blocking_txonly(p.SPI0, clk, mosi, display_config.clone());
    let spi_bus: Mutex<NoopRawMutex, _> = Mutex::new(RefCell::new(spi));

    let display_spi = SpiDeviceWithConfig::new(
        &spi_bus,
        Output::new(display_cs, Level::High),
        display_config,
    );

    let dcx = Output::new(dcx, Level::Low);
    let rst = Output::new(rst, Level::Low);
    // dcx: 0 = command, 1 = data

    // display interface abstraction from SPI and DC
    let di = SPIInterface::new(display_spi, dcx);

    // Define the display from the display interface and initialize it
    let mut display = Builder::new(ST7789, di)
        .display_size(240, 320)
        .reset_pin(rst)
        .orientation(Orientation::new().rotate(Rotation::Deg90))
        .invert_colors(mipidsi::options::ColorInversion::Inverted)
        .init(&mut Delay)
        .unwrap();
    display.clear(Rgb565::RED).unwrap();

    //let mut fbuff = unsafe { FrameBuf::new(&mut DATA, 320, 240); };

    let raw_image_data = ImageRawLE::new(include_bytes!("../assets/ferris.raw"), 86);
    let ferris = Image::new(&raw_image_data, Point::new(34, 68));

    // Display the image
    ferris.draw(&mut display).unwrap();

    let style = MonoTextStyle::new(&FONT_10X20, Rgb565::GREEN);
    Text::new(
        "Hello embedded_graphics \n + incredicalculator",
        Point::new(20, 200),
        style,
    )
    .draw(&mut display)
    .unwrap();
    let mut icalc: IcShell = IcShell::new();
    //let mut b_states = [false; 7];
    let mut ic_rp_platform = IcRpPlatform::new();
    display.clear(Rgb565::CYAN).unwrap();

    loop {
        let futures_array = [
            btn0.wait_for_falling_edge(),
            btn1.wait_for_falling_edge(),
            btn2.wait_for_falling_edge(),
            btn3.wait_for_falling_edge(),
            btn4.wait_for_falling_edge(),
            btnl.wait_for_falling_edge(),
            btnr.wait_for_falling_edge(),
        ];
        let (_, idx) = select_array(futures_array).await;
        match idx {
            0 => icalc.key_down(IcKey::Num0),
            1 => icalc.key_down(IcKey::Num1),
            2 => icalc.key_down(IcKey::Num2),
            3 => icalc.key_down(IcKey::Num3),
            4 => icalc.key_down(IcKey::Num4),
            5 => icalc.key_down(IcKey::Func6),
            6 => icalc.key_down(IcKey::Func5),
            _ => (),
        }
        led.set_high();
        info!("Pre-update");
        icalc.update(&mut ic_rp_platform);
        led.set_low();
        info!("Post-update");
        match idx {
            0 => icalc.key_up(IcKey::Num0),
            1 => icalc.key_up(IcKey::Num1),
            2 => icalc.key_up(IcKey::Num2),
            3 => icalc.key_up(IcKey::Num3),
            4 => icalc.key_up(IcKey::Num4),
            5 => icalc.key_up(IcKey::Func6),
            6 => icalc.key_up(IcKey::Func2),
            _ => (),
        }
        display.clear(Rgb565::BLUE).unwrap();
        for i in 0..ic_rp_platform.line_idx {
            let line = ic_rp_platform.line_list[i];
            primitives::Line::new(
                Point::new(line.x1 as i32, line.y1 as i32),
                Point::new(line.x2 as i32, line.y2 as i32),
            )
            .into_styled(PrimitiveStyle::with_stroke(Rgb565::WHITE, 3))
            .draw(&mut display)
            .unwrap();
        }
        if btnr.is_low() {
            rom_data::reset_to_usb_boot(0, 0);
        }
    }
}
