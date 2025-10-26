#![no_std]
#![no_main]

use core::cell::RefCell;

use defmt::*;
use display_interface_spi::SPIInterface;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output, Input};
use embassy_rp::spi;
use embassy_rp::spi::Spi;
use embassy_rp::rom_data;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::Delay;
use embedded_graphics::image::{Image, ImageRawLE};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
use embedded_graphics::text::Text;
use mipidsi::options::{Orientation, Rotation};
use mipidsi::Builder;
use mipidsi::models::ST7789;
use embassy_time::Timer;
use embedded_alloc::LlffHeap as Heap;
use incredicalculator_core::{IcKey, IcPlatform, IcState};
use {defmt_rtt as _, panic_probe as _};

const DISPLAY_FREQ: u32 = 2_000_000;

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024;
        static HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }
    let p = embassy_rp::init(Default::default());
    info!("Hello World!");
    let mut btn = Input::new(p.PIN_26, embassy_rp::gpio::Pull::Up);
    let bootmode_btn = Input::new(p.PIN_12, embassy_rp::gpio::Pull::Up);
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

    let display_spi = SpiDeviceWithConfig::new(&spi_bus, Output::new(display_cs, Level::High), display_config);

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
    display.clear(Rgb565::GREEN).unwrap();

    let raw_image_data = ImageRawLE::new(include_bytes!("../assets/ferris.raw"), 86);
    let ferris = Image::new(&raw_image_data, Point::new(34, 68));

    // Display the image
    ferris.draw(&mut display).unwrap();

    let style = MonoTextStyle::new(&FONT_10X20, Rgb565::GREEN);
    Text::new(
        "Hello embedded_graphics \n + embassy + RP2040!",
        Point::new(20, 200),
        style,
    )
        .draw(&mut display)
        .unwrap();
    let mut icalc: IcState = IcState::new();
    loop {
        btn.wait_for_falling_edge().await;
        let style = PrimitiveStyleBuilder::new().fill_color(Rgb565::WHITE).build();
        Rectangle::new(Point::new(10, 10), Size::new(16, 16))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();
        info!("led on!");
        info!("{}", icalc.current_eq_len);
        led.set_high();
        Timer::after_millis(250).await;
        let style = PrimitiveStyleBuilder::new().fill_color(Rgb565::BLACK).build();
        Rectangle::new(Point::new(10, 10), Size::new(16, 16))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();
        info!("led off!");
        led.set_low();
        Timer::after_millis(250).await;
        if bootmode_btn.is_low() {
            rom_data::reset_to_usb_boot(0, 0);
        }
    }
}
