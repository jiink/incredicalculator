#![no_std]
#![no_main]

extern crate alloc;

use core::cell::RefCell;

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{gpio::{self, Pull}, spi};
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_time::Timer;
use gpio::{Level, Output, Input};
use {defmt_rtt as _, panic_probe as _};
use incredicalculator_core::{IcKey, IcPlatform, IcState};
use embedded_alloc::LlffHeap as Heap;
use embassy_sync::blocking_mutex::Mutex;
use display_interface_spi::SPIInterface;
use mipidsi::{models::ST7789, options::{ColorInversion, Orientation}, Builder};
use embassy_time::Delay;
use embedded_graphics::pixelcolor::Rgb565;

#[global_allocator]
static HEAP: Heap = Heap::empty();

// Program metadata for `picotool info`.
// This isn't needed, but it's recomended to have these minimal entries.
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"Incredicalculator"),
    embassy_rp::binary_info::rp_program_description!(
        c"It's not done, sir."
    ),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024;
        static HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }

    let p = embassy_rp::init(Default::default());
    let mut btn = Input::new(p.PIN_26, Pull::Up);
    let mut led = Output::new(p.PIN_25, Level::Low);
    let mosi = p.PIN_7;
    let clk = p.PIN_6;
    let chip_sel = p.PIN_5;
    let data_cmd = p.PIN_8;
    let disp_reset = p.PIN_4;
    let mut spi_cfg = spi::Config::default();
    spi_cfg.frequency = 2_000_000;
    spi_cfg.phase = spi::Phase::CaptureOnSecondTransition;
    spi_cfg.polarity = spi::Polarity::IdleHigh;
    let spi = embassy_rp::spi::Spi::new_blocking_txonly(p.SPI0, clk, mosi, spi_cfg);
    let spi_bus = Mutex::new(RefCell::new(spi));
    let disp_spi = SpiDeviceWithConfig::new(&spi_bus, Output::new(chip_sel, Level::High), spi_cfg);
    let mut buffer = [0u8; 640]; // todo: play with the size of this thing
    let disp_iface = SPIInterface::new(spi, data_cmd);
    let mut display = Builder::new(ST7789, disp_iface)
        .display_size(240, 320)
        .reset_pin(disp_reset)
        .orientation(Orientation::new())
        .init(&mut Delay)
        .unwrap();
    display.clear(Rgb565::RED).unwrap();
    let mut icalc: IcState = IcState::new();
    loop {
        btn.wait_for_falling_edge().await;
        info!("led on!");
        led.set_high();
        Timer::after_millis(250).await;

        info!("led off!");
        led.set_low();
        Timer::after_millis(250).await;
    }
}
