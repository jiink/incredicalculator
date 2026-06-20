#![no_std]
#![no_main]

extern crate alloc;

use core::{cell::RefCell, fmt};

use defmt::*;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_executor::{Executor, Spawner};
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::multicore::{Stack, spawn_core1};
use embassy_rp::spi;
use embassy_rp::spi::Spi;
use embassy_rp::pwm::{Config as PwmConfig, Pwm}; 
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
use embassy_sync::channel::Channel;
use embassy_time::Delay;
use embassy_time::Timer;
use embedded_alloc::LlffHeap as Heap;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::pixelcolor::{Rgb565};
use embedded_graphics::primitives::{PrimitiveStyle, PrimitiveStyleBuilder};
use embedded_graphics::text::Text;
use embedded_graphics::{prelude::*};
use embedded_graphics_framebuf::FrameBuf;
use incredicalculator_core::input::IcKey;
use incredicalculator_core::platform::IcPlatform;
use incredicalculator_core::shell::IcShell;
use glam::IVec2;
use mipidsi::interface::SpiInterface;
use rgb::RGB8;
use mipidsi::Builder;
use mipidsi::models::ST7789;
use mipidsi::options::{Orientation, Rotation};
use static_cell::StaticCell;

use {defmt_rtt as _, panic_probe as _};

const DISPLAY_FREQ: u32 = 60_000_000;

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[unsafe(link_section = ".uninit.HEAP_MEM")]
static mut HEAP_MEM: [u8; 64_000] = [0; 64_000];

const RENDER_W: u32 = 320;
const RENDER_H: u32 = 240;
const PIXEL_COUNT: usize = (RENDER_W * RENDER_H) as usize;

static mut CANVAS_DATA: [Rgb565; PIXEL_COUNT] = [Rgb565::new(0, 0, 0); PIXEL_COUNT];

static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();
static INPUT_BUFFER: Channel<CriticalSectionRawMutex, InputBufferEvent, 32> = Channel::new();

enum KeyMovement {
    Up,
    Down
}

struct InputBufferEvent {
    key: IcKey,
    movement: KeyMovement
}

pub struct IcRpPlatform {
    pub canvas_data: &'static mut [Rgb565; PIXEL_COUNT],
}

impl IcRpPlatform {
    pub fn new() -> IcRpPlatform {
        IcRpPlatform {
            canvas_data: unsafe { &mut *core::ptr::addr_of_mut!(CANVAS_DATA) }
        }
    }
}

impl IcPlatform for IcRpPlatform {
    fn draw_line(&mut self, start: IVec2, end: IVec2, color: RGB8, width: u32) {
        let mut fbuf = FrameBuf::new(&mut *self.canvas_data, RENDER_W as usize, RENDER_H as usize);
        embedded_graphics::primitives::Line::new(
            embedded_graphics::prelude::Point::new(start.x, start.y),
            embedded_graphics::prelude::Point::new(end.x, end.y),
        )   
        .into_styled(PrimitiveStyle::with_stroke(rgbu8_to_rgb565(color), width))
        .draw(&mut fbuf)
        .unwrap();
    }

    fn draw_rectangle(&mut self, start: IVec2, end: IVec2, stroke_color: RGB8, stroke_width: u32, fill_color: Option<RGB8>) {
        let mut fbuf = FrameBuf::new(&mut *self.canvas_data, RENDER_W as usize, RENDER_H as usize);
        let mut style_builder = PrimitiveStyleBuilder::new()
            .stroke_color(rgbu8_to_rgb565(stroke_color))
            .stroke_width(stroke_width)
            .stroke_alignment(embedded_graphics::primitives::StrokeAlignment::Center);
        if let Some(c) = fill_color {
            style_builder = style_builder.fill_color(rgbu8_to_rgb565(c));
        }
        let style = style_builder.build();
        embedded_graphics::primitives::Rectangle::with_corners(
            embedded_graphics::prelude::Point::new(start.x, start.y),
            embedded_graphics::prelude::Point::new(end.x, end.y),
        )
        .into_styled(style)
        .draw(&mut fbuf)
        .unwrap();
    }

    fn draw_rectangle_rounded(
        &mut self,
        start: IVec2,
        end: IVec2,
        stroke_color: RGB8,
        stroke_width: u32,
        fill_color: Option<RGB8>,
        corner_radius: u32,
    ) {
        let mut fbuf = FrameBuf::new(&mut *self.canvas_data, RENDER_W as usize, RENDER_H as usize);
        let mut style_builder = PrimitiveStyleBuilder::new()
            .stroke_color(rgbu8_to_rgb565(stroke_color))
            .stroke_width(stroke_width)
            .stroke_alignment(embedded_graphics::primitives::StrokeAlignment::Center);
        if let Some(c) = fill_color {
            style_builder = style_builder.fill_color(rgbu8_to_rgb565(c));
        }
        let style = style_builder.build();
        embedded_graphics::primitives::RoundedRectangle::with_equal_corners(
            embedded_graphics::primitives::Rectangle::with_corners(
                embedded_graphics::prelude::Point::new(start.x, start.y),
                embedded_graphics::prelude::Point::new(end.x, end.y),
            ),
            embedded_graphics::prelude::Size::new(corner_radius, corner_radius),
        )
        .into_styled(style)
        .draw(&mut fbuf)
        .unwrap();
    }

    fn draw_triangle(&mut self, vertex1: IVec2, vertex2: IVec2, vertex3: IVec2, stroke_color: RGB8, stroke_width: u32, fill_color: Option<RGB8>) {
        let mut fbuf = FrameBuf::new(&mut *self.canvas_data, RENDER_W as usize, RENDER_H as usize);
        let mut style_builder = PrimitiveStyleBuilder::new()
            .stroke_color(rgbu8_to_rgb565(stroke_color))
            .stroke_width(stroke_width)
            .stroke_alignment(embedded_graphics::primitives::StrokeAlignment::Center);
        if let Some(c) = fill_color {
            style_builder = style_builder.fill_color(rgbu8_to_rgb565(c));
        }
        let style = style_builder.build();
        embedded_graphics::primitives::Triangle::new(
            embedded_graphics::prelude::Point::new(vertex1.x, vertex1.y),
            embedded_graphics::prelude::Point::new(vertex2.x, vertex2.y),
            embedded_graphics::prelude::Point::new(vertex3.x, vertex3.y),
        )
        .into_styled(style).draw(&mut fbuf).unwrap();
    }

    fn draw_string(&mut self, text: &str, pos: IVec2, _size: u32, color: RGB8) {
        let mut fbuf = FrameBuf::new(&mut *self.canvas_data, RENDER_W as usize, RENDER_H as usize);
        
        // using a BUILT-IN FONT!
        let char_style = embedded_graphics::mono_font::MonoTextStyle::new(
            &embedded_graphics::mono_font::ascii::FONT_10X20,
            rgbu8_to_rgb565(color)
        );
        let text_style = embedded_graphics::text::TextStyleBuilder::new()
        .alignment(embedded_graphics::text::Alignment::Left)
        .baseline(embedded_graphics::text::Baseline::Top)
        .build();
        embedded_graphics::text::Text::with_text_style(
            text,
            embedded_graphics::prelude::Point::new(pos.x, pos.y),
            char_style,
            text_style,
        )
        .draw(&mut fbuf)
        .unwrap();
    }

    fn draw_string_f(&mut self, arg: fmt::Arguments, pos: IVec2, size: u32, color: RGB8) {
        let mut buf = [0u8; 128];
        self.draw_string(format_no_std::show(&mut buf, arg).unwrap(), pos, size, color);
    }

    fn clear(&mut self, color: RGB8) {
        self.canvas_data.fill(rgbu8_to_rgb565(color));
    }

    fn log(&mut self, _arg: fmt::Arguments) {}

    fn millis(&self) -> u64 {
        0
    }
}

const MATRIX_ROWS: usize = 5;
const MATRIX_COLS: usize = 4;

struct KeyMatrix<'d> {
    rows: [Output<'d>; MATRIX_ROWS],
    cols: [Input<'d>; MATRIX_COLS],
    prev_pressed: [bool; IcKey::COUNT],
}

impl<'d> KeyMatrix<'d> {
    const MAP: [[Option<IcKey>; MATRIX_COLS]; MATRIX_ROWS] = [
        [None, None, Some(IcKey::Func1), Some(IcKey::Func2)],
        [Some(IcKey::Num7), Some(IcKey::Num8), Some(IcKey::Num9), Some(IcKey::Func3)],
        [Some(IcKey::Num4), Some(IcKey::Num5), Some(IcKey::Num6), Some(IcKey::Func4)],
        [Some(IcKey::Num1), Some(IcKey::Num2), Some(IcKey::Num3), Some(IcKey::Func5)],
        [Some(IcKey::Num0), Some(IcKey::Shift), Some(IcKey::Super), Some(IcKey::Func6)],
    ];

    pub fn new(rows: [Output<'d>; MATRIX_ROWS], cols: [Input<'d>; MATRIX_COLS]) -> Self {
        let mut matrix = KeyMatrix {
            rows,
            cols,
            prev_pressed: [false; IcKey::COUNT],
        };
        matrix.all_rows_high();
        matrix
    }

    fn all_rows_high(&mut self) {
        for row in self.rows.iter_mut() {
            row.set_high();
        }
    }

    fn select_row(&mut self, idx: usize) {
        self.all_rows_high();
        self.rows[idx].set_low();
    }

    fn scan(&mut self) -> [bool; IcKey::COUNT] {
        let mut pressed = [false; IcKey::COUNT];

        for row in 0..MATRIX_ROWS {
            self.select_row(row);
            for col in 0..MATRIX_COLS {
                if self.cols[col].is_low() {
                    if let Some(key) = Self::MAP[row][col] {
                        pressed[key as usize] = true;
                    }
                }
            }
        }

        self.all_rows_high();
        pressed
    }

    fn update_shell(&mut self, shell: &mut IcShell) -> bool {
        let current_pressed = self.scan();
        let mut changed = false;
        for idx in 0..IcKey::COUNT {
            if current_pressed[idx] != self.prev_pressed[idx] {
                changed = true;
                if let Some(key) = Self::key_from_index(idx) {
                    if current_pressed[idx] {
                        shell.key_down(key);
                    } else {
                        shell.key_up(key);
                    }
                }
                
            }
        }
        self.prev_pressed = current_pressed;
        changed
    }

    fn key_from_index(idx: usize) -> Option<IcKey> {
        match idx {
            0 => Some(IcKey::Num0),
            1 => Some(IcKey::Num1),
            2 => Some(IcKey::Num2),
            3 => Some(IcKey::Num3),
            4 => Some(IcKey::Num4),
            5 => Some(IcKey::Num5),
            6 => Some(IcKey::Num6),
            7 => Some(IcKey::Num7),
            8 => Some(IcKey::Num8),
            9 => Some(IcKey::Num9),
            10 => Some(IcKey::Func1),
            11 => Some(IcKey::Func2),
            12 => Some(IcKey::Func3),
            13 => Some(IcKey::Func4),
            14 => Some(IcKey::Func5),
            15 => Some(IcKey::Func6),
            16 => Some(IcKey::Shift),
            17 => Some(IcKey::Super),
            _ => None,
        }
    }

    pub fn is_pressed(&self, key: IcKey) -> bool {
        self.prev_pressed[key as usize]
    }
}

fn rgbu8_to_rgb565(rgbu8_col: rgb::Rgb<u8>) -> Rgb565 {
    Rgb565::new(rgbu8_col.r >> 3, rgbu8_col.g >> 2, rgbu8_col.b >> 3)
}

fn reboot_into_bootloader() {
    const MS_BEFORE_BOOT: u32 = 500;
    info!("REBOOTING INTO BOOTSEL MODE IN {} MS!", &MS_BEFORE_BOOT);
    // see rp2350 datasheet section 5.4.8.24
    let reboot_type_bootsel: u32 = 0x0002; // "reboot into BOOTSEL mode."
    let no_return_on_success: u32 = 0x0100; // "the watchdog hardware is asynchronous. Setting this bit forces this method not to return if the reboot is successfully initiated."
    let gpio_pin_enabled: u32 = 0x20; // "Enable the activity indicator on the specified GPIO"
    let led_gpio_num = 22;
    embassy_rp::rom_data::reboot(
        reboot_type_bootsel | no_return_on_success,
        MS_BEFORE_BOOT, gpio_pin_enabled, led_gpio_num);
}

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
    let test_str = alloc::string::String::from("test");
    info!(
        "alloc works: (len {}) - \"{}\"",
        test_str.len(),
        test_str.as_str()
    );

    // for pico 2
    // let mut btn0 = Input::new(p.PIN_26, embassy_rp::gpio::Pull::Up);
    // let mut btn1 = Input::new(p.PIN_12, embassy_rp::gpio::Pull::Up);
    // let mut btn2 = Input::new(p.PIN_11, embassy_rp::gpio::Pull::Up);
    // let mut btn3 = Input::new(p.PIN_2, embassy_rp::gpio::Pull::Up);
    // let mut btn4 = Input::new(p.PIN_1, embassy_rp::gpio::Pull::Up);
    // let mut btnl = Input::new(p.PIN_27, embassy_rp::gpio::Pull::Up);
    // let mut btnr = Input::new(p.PIN_0, embassy_rp::gpio::Pull::Up);
    // let mut led = Output::new(p.PIN_25, Level::Low);
    // let rst = p.PIN_4;
    // let display_cs = p.PIN_5;
    // let dcx = p.PIN_8;
    // let mosi = p.PIN_7;
    // let clk = p.PIN_6;
    // let lcd_spi_bus = p.SPI0;

    // for incredicalculator board
    // row0: gpio13
    // row1: gpio14
    // row2: gpio15
    // row3: gpio16
    // row4: gpio17
    // col0: gpio21
    // col1: gpio20
    // col2: gpio19
    // col3: gpio18
    // row0, col0: no button present
    // row0, col1: no button present
    // row0, col2: switch 1 - Func1
    // row0, col3: switch 2 - Func2
    // row1, col0: switch 3 - Num7
    // row1, col1: switch 4 - Num8
    // row1, col2: switch 5 - Num9
    // row1, col3: switch 6 - Func3
    // row2, col0: switch 7 - Num4
    // row2, col1: switch 8 - Num5
    // row2, col2: switch 9 - Num6
    // row2, col3: switch 10 - Func4
    // row3, col0: switch 11 - Num1
    // row3, col1: switch 12 - Num2
    // row3, col2: switch 13 - Num3
    // row3, col3: switch 14 - Func5
    // row4, col0: switch 15 - Num0
    // row4, col1: switch 16 - Shift
    // row4, col2: switch 17 - Super
    // row4, col3: switch 18 - Func6
    let matrix_rows = [
        Output::new(p.PIN_13, Level::High),
        Output::new(p.PIN_14, Level::High),
        Output::new(p.PIN_15, Level::High),
        Output::new(p.PIN_16, Level::High),
        Output::new(p.PIN_17, Level::High),
    ];

    let matrix_cols = [
        Input::new(p.PIN_21, Pull::Up),
        Input::new(p.PIN_20, Pull::Up),
        Input::new(p.PIN_19, Pull::Up),
        Input::new(p.PIN_18, Pull::Up),
    ];

    let mut key_matrix = KeyMatrix::new(matrix_rows, matrix_cols);
    let mut led = Output::new(p.PIN_22, Level::Low);
    let rst = p.PIN_47;
    let display_cs = p.PIN_45;
    let dcx = p.PIN_42;
    let mosi = p.PIN_43;
    let clk = p.PIN_46;
    let module_bl = p.PIN_31;
    let bare_display_bl = p.PIN_41;
    let lcd_spi_bus = p.SPI1;

    // ST7789 datasheet: "If not used, please fix this pin at VDDI or DGND."
    let _tft_unused_d0 = Output::new(p.PIN_33, Level::Low);
    let _tft_unused_d1 = Output::new(p.PIN_34, Level::Low);
    let _tft_unused_d2 = Output::new(p.PIN_35, Level::Low);
    let _tft_unused_d3 = Output::new(p.PIN_36, Level::Low);
    let _tft_unused_d4 = Output::new(p.PIN_37, Level::Low);
    let _tft_unused_d5 = Output::new(p.PIN_38, Level::Low);
    let _tft_unused_d6 = Output::new(p.PIN_39, Level::Low);
    let _tft_unused_d7 = Output::new(p.PIN_40, Level::Low);

    // PWM backlight
    let mut pwm_config = PwmConfig::default();
    pwm_config.top = 0xFFFF;
    pwm_config.compare_b = 0xFFFF/2;
    let _backlight = Pwm::new_output_b(p.PWM_SLICE7, module_bl, pwm_config);
    let mut pwm_config2 = PwmConfig::default();
    pwm_config2.top = 0xFFFF;
    pwm_config2.compare_b = 0xFFFF/2;
    let _backlight2 = Pwm::new_output_b(p.PWM_SLICE8, bare_display_bl, pwm_config2);

    // create SPI
    let mut display_config = spi::Config::default();
    display_config.frequency = DISPLAY_FREQ;
    display_config.phase = spi::Phase::CaptureOnSecondTransition;
    display_config.polarity = spi::Polarity::IdleHigh;

    let spi = Spi::new_blocking_txonly(lcd_spi_bus, clk, mosi, display_config.clone());
    let spi_bus: Mutex<NoopRawMutex, _> = Mutex::new(RefCell::new(spi));

    let display_spi = SpiDeviceWithConfig::new(
        &spi_bus,
        Output::new(display_cs, Level::High),
        display_config,
    );

    // dcx: 0 = command, 1 = data
    let dcx = Output::new(dcx, Level::Low);
    let rst = Output::new(rst, Level::Low);

    // display interface abstraction from SPI and DC
    let mut spi_buf = [0_u8; 512];
    let di = SpiInterface::new(display_spi, dcx, &mut spi_buf);

    // Define the display from the display interface and initialize it
    let mut display = Builder::new(ST7789, di)
        .display_size(240, 320)
        .reset_pin(rst)
        .orientation(Orientation::new().rotate(Rotation::Deg270))
        .invert_colors(mipidsi::options::ColorInversion::Inverted)
        .init(&mut Delay)
        .unwrap();
    display.clear(Rgb565::CSS_PURPLE).unwrap();

    spawn_core1(
        p.CORE1,
        unsafe {
            // consider changing to new syntax: `&mut *&raw mut CORE1_STACK`
            &mut *core::ptr::addr_of_mut!(CORE1_STACK)
        },
        move || {
            let exec1 = EXECUTOR1.init(Executor::new());
            exec1.run(
                |spawner|
                {
                    unwrap!(spawner.spawn(inputs_core1_task(led)));
                }
            );
        }
    );

    let style = MonoTextStyle::new(&FONT_10X20, Rgb565::GREEN);
    Text::new(
        "Hello embedded_graphics \n + incredicalculator",
        Point::new(20, 200),
        style,
    )
    .draw(&mut display)
    .unwrap();
    let mut icalc: IcShell = IcShell::new();
    let mut ic_rp_platform = IcRpPlatform::new();
    display.clear(Rgb565::CYAN).unwrap();
    let mut frame_counter: usize = 0;
    loop {
        let keys_changed = key_matrix.update_shell(&mut icalc);
        if keys_changed {
            //force_draw = false;
            //led.set_high();
            if let Err(_) = INPUT_BUFFER.try_send(InputBufferEvent {
                key: IcKey::Num1,
                movement: KeyMovement::Down
            }) {
                warn!("Input buffer overflowed");
            };
            info!("Pre-update");
            icalc.update(&mut ic_rp_platform);
            //led.set_low();
            if let Err(_) = INPUT_BUFFER.try_send(InputBufferEvent {
                key: IcKey::Num1,
                movement: KeyMovement::Up
            }) {
                warn!("Input buffer overflowed");
            };
            info!("Post-update");
            ic_rp_platform.draw_string_f(
                format_args!("{}...", frame_counter % 10),
                glam::IVec2::new(0, 0),
                1,
                RGB8::new(0, 255, 0)
            );
            frame_counter = frame_counter.wrapping_add(1);
            display.fill_contiguous(
                &embedded_graphics::primitives::Rectangle::new(
                    embedded_graphics::prelude::Point::new(0, 0),
                    embedded_graphics::prelude::Size::new(RENDER_W, RENDER_H)
                ),
                ic_rp_platform.canvas_data.iter().copied()
            ).unwrap();
        }
        // try putting this in an "else" block
        embassy_time::Timer::after(embassy_time::Duration::from_millis(10)).await;
        if key_matrix.is_pressed(IcKey::Super) && key_matrix.is_pressed(IcKey::Num3) {
            reboot_into_bootloader();
        }
    }
}

#[embassy_executor::task]
async fn inputs_core1_task(mut led: Output<'static>) {
    info!("Hello from the \"inputs core\"");
    loop {
        match INPUT_BUFFER.receive().await.movement {
            KeyMovement::Up => led.set_high(),
            KeyMovement::Down => led.set_low()
        }
    }
}
