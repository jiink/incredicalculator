#![no_std]
#![no_main]

extern crate alloc;

use core::sync::atomic::{AtomicI32, AtomicU32, Ordering};
use core::{fmt, mem};

use defmt::*;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDeviceWithConfig;
use embassy_executor::{Executor, Spawner};
use embassy_futures::select::{Either, Either3, select, select3, select4};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::multicore::{Stack, spawn_core1};
use embassy_rp::peripherals::{PIO0, SPI1};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::pio_programs::i2s::{PioI2sOut, PioI2sOutProgram};
use embassy_rp::pwm::{Config as PwmConfig, Pwm};
use embassy_rp::spi;
use embassy_rp::spi::Spi;
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
use embassy_sync::channel::{Channel, DynamicSender};
use embassy_sync::mutex::Mutex as AsyncMutex;
use embassy_time::Timer;
use embassy_time::{Delay, Instant};
use embedded_alloc::LlffHeap as Heap;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::PrimitiveStyleBuilder;
use glam::{IVec2, Vec2};
use incredicalculator_core::input::IcKey;
use incredicalculator_core::platform::IcPlatform;
use incredicalculator_core::shell::IcShell;
use lcd_async::interface::SpiInterface;
use lcd_async::models::ST7789;
use lcd_async::options::{ColorInversion, Orientation, Rotation};
use lcd_async::raw_framebuf::RawFrameBuf;
use lcd_async::{Builder, Display};
use max170xx::Max17048;
use rgb::RGB8;
use static_cell::{ConstStaticCell, StaticCell};
use {defmt_rtt as _, panic_probe as _};

const DISPLAY_FREQ: u32 = 60_000_000;

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[unsafe(link_section = ".uninit.HEAP_MEM")]
static mut HEAP_MEM: [u8; 64_000] = [0; 64_000];

const RENDER_W: u32 = 320;
const RENDER_H: u32 = 240;
const PIXEL_COUNT: usize = (RENDER_W * RENDER_H) as usize;
const FRAME_BUFFER_SIZE: usize = PIXEL_COUNT * 2;

// A canvas belongs either to the UI (where it is safe to draw into) or to the
// display task (while SPI DMA is reading from it). Never share one canvas
// between those two owners.
struct FrameBuffer {
    pixels: &'static mut [u8; FRAME_BUFFER_SIZE],
}

// `ConstStaticCell` keeps these large zeroed arrays in `.bss`. Do not use
// `StaticCell::init([0; FRAME_BUFFER_SIZE])` here: that would create a 150 KiB
// temporary on the core-0 stack before moving it into the cell.
static CANVAS_DATA0: ConstStaticCell<[u8; FRAME_BUFFER_SIZE]> =
    ConstStaticCell::new([0; FRAME_BUFFER_SIZE]);
static CANVAS_DATA1: ConstStaticCell<[u8; FRAME_BUFFER_SIZE]> =
    ConstStaticCell::new([0; FRAME_BUFFER_SIZE]);
static FREE_FRAME_BUFFERS: Channel<CriticalSectionRawMutex, FrameBuffer, 2> = Channel::new();
static READY_FRAME_BUFFERS: Channel<CriticalSectionRawMutex, FrameBuffer, 2> = Channel::new();

static DISPLAY_SPI_BUS: StaticCell<AsyncMutex<NoopRawMutex, Spi<'static, SPI1, spi::Async>>> =
    StaticCell::new();

type DisplaySpiDevice =
    SpiDeviceWithConfig<'static, NoopRawMutex, Spi<'static, SPI1, spi::Async>, Output<'static>>;
type LcdDisplay = Display<SpiInterface<DisplaySpiDevice, Output<'static>>, ST7789, Output<'static>>;

static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();
static INPUT_BUFFER: Channel<CriticalSectionRawMutex, InputBufferEvent, 32> = Channel::new();

type BoardI2c =
    embassy_rp::i2c::I2c<'static, embassy_rp::peripherals::I2C0, embassy_rp::i2c::Blocking>;
static BATTERY_SOC: AtomicI32 = AtomicI32::new(-1);

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});
const AUDIO_SAMPLE_RATE: u32 = 48_000;
const AUDIO_BIT_DEPTH: u32 = 16;
const AUDIO_BUFFER_SIZE: usize = 2048;
const AUDIO_BUFFER_DURATION_US: u32 = (AUDIO_BUFFER_SIZE as u32 * 1_000_000) / AUDIO_SAMPLE_RATE;
// The PIO FIFO is only a few samples deep. Any synchronous operation longer
// than this can prevent the executor from queuing the next DMA transfer.
const AUDIO_FIFO_COVERAGE_US: u32 = 200;
const AUDIO_SLOW_WORK_WARN_US: u32 = 1_000;

// Keep instrumentation out of the real-time path: the audio task records
// counters only, while the main task prints a compact report once per second.
static AUDIO_BUFFERS_RENDERED: AtomicU32 = AtomicU32::new(0);
static AUDIO_RENDER_MAX_US: AtomicU32 = AtomicU32::new(0);
static AUDIO_QUEUE_SEND_MAX_US: AtomicU32 = AtomicU32::new(0);
static AUDIO_DMA_TRANSFERS: AtomicU32 = AtomicU32::new(0);
static AUDIO_DMA_MAX_US: AtomicU32 = AtomicU32::new(0);
static AUDIO_LATE_DMA_COMPLETIONS: AtomicU32 = AtomicU32::new(0);
static AUDIO_STARVATIONS: AtomicU32 = AtomicU32::new(0);
static AUDIO_STARVATION_MAX_US: AtomicU32 = AtomicU32::new(0);
static SHELL_UPDATES: AtomicU32 = AtomicU32::new(0);
static SHELL_UPDATE_MAX_US: AtomicU32 = AtomicU32::new(0);
static DISPLAY_TRANSFERS: AtomicU32 = AtomicU32::new(0);
static DISPLAY_TRANSFER_MAX_US: AtomicU32 = AtomicU32::new(0);

fn record_max(maximum: &AtomicU32, value: u32) {
    maximum.fetch_max(value, Ordering::Relaxed);
}

fn elapsed_us(start: Instant) -> u32 {
    start.elapsed().as_micros().min(u32::MAX as u64) as u32
}

fn report_audio_diagnostics() {
    let rendered = AUDIO_BUFFERS_RENDERED.swap(0, Ordering::Relaxed);
    let render_max_us = AUDIO_RENDER_MAX_US.swap(0, Ordering::Relaxed);
    let queue_send_max_us = AUDIO_QUEUE_SEND_MAX_US.swap(0, Ordering::Relaxed);
    let dma_transfers = AUDIO_DMA_TRANSFERS.swap(0, Ordering::Relaxed);
    let dma_max_us = AUDIO_DMA_MAX_US.swap(0, Ordering::Relaxed);
    let late_dma_completions = AUDIO_LATE_DMA_COMPLETIONS.swap(0, Ordering::Relaxed);
    let starvations = AUDIO_STARVATIONS.swap(0, Ordering::Relaxed);
    let starvation_max_us = AUDIO_STARVATION_MAX_US.swap(0, Ordering::Relaxed);
    let shell_updates = SHELL_UPDATES.swap(0, Ordering::Relaxed);
    let shell_update_max_us = SHELL_UPDATE_MAX_US.swap(0, Ordering::Relaxed);
    let display_transfers = DISPLAY_TRANSFERS.swap(0, Ordering::Relaxed);
    let display_transfer_max_us = DISPLAY_TRANSFER_MAX_US.swap(0, Ordering::Relaxed);

    info!(
        "audio/s: rendered={} dma={} render_max={}us queue_max={}us dma_max={}us dma_late={} starve={} starve_max={}us shell_updates={} update_max={}us display={} display_max={}us",
        rendered,
        dma_transfers,
        render_max_us,
        queue_send_max_us,
        dma_max_us,
        late_dma_completions,
        starvations,
        starvation_max_us,
        shell_updates,
        shell_update_max_us,
        display_transfers,
        display_transfer_max_us,
    );

    if starvations != 0 {
        warn!(
            "I2S buffer starvation: audio DMA finished before a filled buffer was ready; max wait={}us",
            starvation_max_us
        );
    }
    if late_dma_completions != 0 {
        warn!(
            "I2S DMA completion was serviced late {} times; expected buffer duration is {}us, observed maximum {}us",
            late_dma_completions, AUDIO_BUFFER_DURATION_US, dma_max_us,
        );
    }
}

struct AudioBuffer {
    samples: &'static mut [u32; AUDIO_BUFFER_SIZE],
}
static EMPTY_BUFFERS: Channel<CriticalSectionRawMutex, AudioBuffer, 2> = Channel::new();

static FILLED_BUFFERS: Channel<CriticalSectionRawMutex, AudioBuffer, 2> = Channel::new();
static AUDIO_DMA0: StaticCell<[u32; AUDIO_BUFFER_SIZE]> = StaticCell::new();
static AUDIO_DMA1: StaticCell<[u32; AUDIO_BUFFER_SIZE]> = StaticCell::new();

enum KeyMovement {
    Up,
    Down,
}

struct InputBufferEvent {
    key: IcKey,
    movement: KeyMovement,
}

pub struct IcRpPlatform<'d> {
    canvas: Option<FrameBuffer>,
    backlight: Pwm<'d>,
    backlight2: Pwm<'d>,
    brightness: u8,
    volume: u8,
}

impl<'d> IcRpPlatform<'d> {
    fn new(backlight: Pwm<'d>, backlight2: Pwm<'d>, canvas: FrameBuffer) -> Self {
        Self {
            canvas: Some(canvas),
            backlight,
            backlight2,
            brightness: 128,
            volume: 100,
        }
    }

    fn has_canvas(&self) -> bool {
        self.canvas.is_some()
    }

    fn set_canvas(&mut self, canvas: FrameBuffer) {
        self.canvas = Some(canvas);
    }

    fn take_canvas(&mut self) -> FrameBuffer {
        self.canvas
            .take()
            .expect("UI tried to submit a missing canvas")
    }

    fn canvas_data_mut(&mut self) -> &mut [u8; FRAME_BUFFER_SIZE] {
        self.canvas
            .as_mut()
            .expect("UI tried to draw while its canvas was in flight")
            .pixels
    }

    fn canvas_rgb565_mut(&mut self) -> &mut [Rgb565] {
        let raw = self.canvas_data_mut();
        let len = raw.len() / core::mem::size_of::<Rgb565>();
        unsafe { core::slice::from_raw_parts_mut(raw.as_mut_ptr() as *mut Rgb565, len) }
    }
}

impl<'d> IcPlatform for IcRpPlatform<'d> {
    fn draw_line(&mut self, start: Vec2, end: Vec2, color: RGB8, width: u32) {
        let c565 = rgbu8_to_rgb565(color);
        let canvas = self.canvas_rgb565_mut();
        if width <= 1 {
            draw_line_wu(
                canvas,
                start.x as f32,
                start.y as f32,
                end.x as f32,
                end.y as f32,
                c565,
            );
        } else {
            draw_line_aa_distance(
                canvas,
                start.x as f32,
                start.y as f32,
                end.x as f32,
                end.y as f32,
                width as f32,
                c565,
            );
        }
    }

    fn draw_polygon(&mut self, points: &[Vec2], fill_color: RGB8) {
        fill_simple_polygon(
            self.canvas_rgb565_mut(),
            points,
            rgbu8_to_rgb565(fill_color),
        );
    }

    fn draw_rectangle(
        &mut self,
        start: IVec2,
        end: IVec2,
        stroke_color: RGB8,
        stroke_width: u32,
        fill_color: Option<RGB8>,
    ) {
        let mut fbuf = RawFrameBuf::<Rgb565, _>::new(
            &mut self.canvas_data_mut()[..],
            RENDER_W as usize,
            RENDER_H as usize,
        );
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
        let mut fbuf = RawFrameBuf::<Rgb565, _>::new(
            &mut self.canvas_data_mut()[..],
            RENDER_W as usize,
            RENDER_H as usize,
        );
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

    fn draw_triangle(
        &mut self,
        vertex1: IVec2,
        vertex2: IVec2,
        vertex3: IVec2,
        stroke_color: RGB8,
        stroke_width: u32,
        fill_color: Option<RGB8>,
    ) {
        let mut fbuf = RawFrameBuf::<Rgb565, _>::new(
            &mut self.canvas_data_mut()[..],
            RENDER_W as usize,
            RENDER_H as usize,
        );
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
        .into_styled(style)
        .draw(&mut fbuf)
        .unwrap();
    }

    fn clear(&mut self, color: RGB8) {
        let mut fbuf = RawFrameBuf::<Rgb565, _>::new(
            &mut self.canvas_data_mut()[..],
            RENDER_W as usize,
            RENDER_H as usize,
        );
        fbuf.clear(rgbu8_to_rgb565(color)).unwrap();
    }

    fn log(&mut self, _arg: fmt::Arguments) {}

    fn millis(&self) -> u64 {
        Instant::now().as_millis()
    }

    fn get_battery_soc(&self) -> Option<i32> {
        let s = BATTERY_SOC.load(core::sync::atomic::Ordering::Relaxed);
        match s {
            0.. => Some(s),
            _ => None,
        }
    }

    fn get_brightness(&self) -> u8 {
        self.brightness
    }

    fn set_brightness(&mut self, value: u8) {
        self.brightness = value;
        let duty = ((value as u32 * 0xffff) / 255) as u16;
        let mut config = PwmConfig::default();
        config.top = 0xffff;
        config.compare_b = duty.max(128);
        self.backlight.set_config(&config);
        self.backlight2.set_config(&config);
    }

    fn get_volume(&self) -> u8 {
        self.volume
    }

    fn set_volume(&mut self, value: u8) {
        self.volume = value;
    }
}

#[inline(always)]
fn blend_rgb565(dst_be: Rgb565, src_native: Rgb565, alpha: u8) -> Rgb565 {
    if alpha == 0 {
        return dst_be;
    }
    let src_u16 = unsafe { core::mem::transmute::<Rgb565, u16>(src_native) };
    if alpha == 255 {
        let out_be = src_u16.to_be();
        return unsafe { core::mem::transmute::<u16, Rgb565>(out_be) };
    }

    let dst_u16 = u16::from_be(unsafe { core::mem::transmute::<Rgb565, u16>(dst_be) });

    let a = ((alpha as u32) + 2) >> 2;
    if a == 0 {
        return dst_be;
    }
    if a >= 64 {
        let out_be = src_u16.to_be();
        return unsafe { core::mem::transmute::<u16, Rgb565>(out_be) };
    }
    let inv_a = 64 - a;

    let d = dst_u16 as u32;
    let s = src_u16 as u32;

    // Mask Red (15:11) & Blue (4:0) together: 0b11111_000000_11111 = 0xF81F
    let rb = ((s & 0xF81F) * a + (d & 0xF81F) * inv_a) >> 6;
    // Mask Green (10:5): 0b00000_111111_00000 = 0x07E0
    let g = ((s & 0x07E0) * a + (d & 0x07E0) * inv_a) >> 6;

    let blended = ((rb & 0xF81F) | (g & 0x07E0)) as u16;
    let out_be = blended.to_be();
    unsafe { core::mem::transmute::<u16, Rgb565>(out_be) }
}

#[inline]
fn trunc_f32(x: f32) -> f32 {
    if x >= 0.0 {
        x as i32 as f32
    } else {
        -((-x) as i32 as f32)
    }
}

#[inline]
fn floor_f32(x: f32) -> f32 {
    let truncated = trunc_f32(x);
    if x >= 0.0 || truncated == x {
        truncated
    } else {
        truncated - 1.0
    }
}

#[inline]
fn ceil_f32(x: f32) -> f32 {
    let truncated = trunc_f32(x);
    if x <= 0.0 || truncated == x {
        truncated
    } else {
        truncated + 1.0
    }
}

#[inline]
fn round_f32(x: f32) -> f32 {
    let abs_x = if x < 0.0 { -x } else { x };
    let rounded = if abs_x - floor_f32(abs_x) >= 0.5 {
        floor_f32(abs_x) + 1.0
    } else {
        floor_f32(abs_x)
    };
    if x < 0.0 { -rounded } else { rounded }
}

#[inline(always)]
fn sqrt_fast(x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    #[cfg(all(target_arch = "arm", target_abi = "eabihf", target_feature = "vfp2"))]
    unsafe {
        let res: f32;
        core::arch::asm!(
            "vsqrt.f32 {0}, {1}",
            out(sreg) res,
            in(sreg) x,
            options(nomem, nostack, preserves_flags)
        );
        res
    }
    #[cfg(all(target_arch = "riscv32", target_feature = "f"))]
    unsafe {
        let res: f32;
        core::arch::asm!(
            "fsqrt.s {0}, {1}",
            out(freg) res,
            in(freg) x,
            options(nomem, nostack, preserves_flags)
        );
        res
    }
    #[cfg(not(any(
        all(target_arch = "arm", target_abi = "eabihf", target_feature = "vfp2"),
        all(target_arch = "riscv32", target_feature = "f")
    )))]
    {
        let i = x.to_bits();
        let i = 0x5f3759df - (i >> 1);
        let y = f32::from_bits(i);
        let y = y * (1.5 - 0.5 * x * y * y);
        x * y
    }
}

#[inline]
fn ipart(x: f32) -> i32 {
    floor_f32(x) as i32
}

#[inline]
fn fpart(x: f32) -> f32 {
    x - floor_f32(x)
}

#[inline]
fn rfpart(x: f32) -> f32 {
    1.0 - fpart(x)
}

#[inline]
fn plot_pixel(buf: &mut [Rgb565], x: i32, y: i32, color: Rgb565, alpha: f32) {
    if alpha <= 0.0 || x < 0 || x >= RENDER_W as i32 || y < 0 || y >= RENDER_H as i32 {
        return;
    }
    let idx = (y as usize) * (RENDER_W as usize) + (x as usize);

    let gamma_alpha = alpha * alpha;
    let a = (round_f32(gamma_alpha.min(1.0) * 255.0)) as u8;

    buf[idx] = blend_rgb565(buf[idx], color, a);
}

fn fill_simple_polygon(buf: &mut [Rgb565], points: &[Vec2], color: Rgb565) {
    if points.len() < 3 {
        return;
    }

    let mut min_x = RENDER_W as i32;
    let mut max_x = -1;
    let mut min_y = RENDER_H as i32;
    let mut max_y = -1;
    for point in points {
        min_x = min_x.min(point.x as i32 - 1);
        max_x = max_x.max(point.x as i32 + 1);
        min_y = min_y.min(point.y as i32 - 1);
        max_y = max_y.max(point.y as i32 + 1);
    }
    min_x = min_x.max(0);
    max_x = max_x.min(RENDER_W as i32 - 1);
    min_y = min_y.max(0);
    max_y = max_y.min(RENDER_H as i32 - 1);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let sample = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            let mut inside = false;
            let mut previous = points[points.len() - 1];
            for &current in points {
                if (current.y > sample.y) != (previous.y > sample.y)
                    && sample.x
                        < (previous.x - current.x) * (sample.y - current.y)
                            / (previous.y - current.y)
                            + current.x
                {
                    inside = !inside;
                }
                previous = current;
            }
            if inside {
                buf[y as usize * RENDER_W as usize + x as usize] = color;
            }
        }
    }
}

fn draw_line_wu(
    buf: &mut [Rgb565],
    mut x1: f32,
    mut y1: f32,
    mut x2: f32,
    mut y2: f32,
    color: Rgb565,
) {
    let mut dx = x2 - x1;
    let mut dy = y2 - y1;

    if dx.abs() < 1e-6 && dy.abs() < 1e-6 {
        plot_pixel(buf, round_f32(x1) as i32, round_f32(y1) as i32, color, 1.0);
        return;
    }

    if dx.abs() > dy.abs() {
        if x1 > x2 {
            mem::swap(&mut x1, &mut x2);
            mem::swap(&mut y1, &mut y2);
        }
        dx = x2 - x1;
        dy = y2 - y1;
        let gradient = if dx == 0.0 { 1.0 } else { dy / dx };

        let x_end1 = round_f32(x1);
        let y_end1 = y1 + gradient * (x_end1 - x1);
        let gap1 = rfpart(x1 + 0.5);
        let ix1 = x_end1 as i32;
        let iy1 = ipart(y_end1);
        plot_pixel(buf, ix1, iy1, color, rfpart(y_end1) * gap1);
        plot_pixel(buf, ix1, iy1 + 1, color, fpart(y_end1) * gap1);
        let mut inter_y = y_end1 + gradient;

        let x_end2 = round_f32(x2);
        let y_end2 = y2 + gradient * (x_end2 - x2);
        let gap2 = fpart(x2 + 0.5);
        let ix2 = x_end2 as i32;
        let iy2 = ipart(y_end2);
        plot_pixel(buf, ix2, iy2, color, rfpart(y_end2) * gap2);
        plot_pixel(buf, ix2, iy2 + 1, color, fpart(y_end2) * gap2);

        for x in (ix1 + 1)..ix2 {
            plot_pixel(buf, x, ipart(inter_y), color, rfpart(inter_y));
            plot_pixel(buf, x, ipart(inter_y) + 1, color, fpart(inter_y));
            inter_y += gradient;
        }
    } else {
        if y1 > y2 {
            mem::swap(&mut x1, &mut x2);
            mem::swap(&mut y1, &mut y2);
        }
        dx = x2 - x1;
        dy = y2 - y1;
        let gradient = if dy == 0.0 { 1.0 } else { dx / dy };

        let y_end1 = round_f32(y1);
        let x_end1 = x1 + gradient * (y_end1 - y1);
        let gap1 = rfpart(y1 + 0.5);
        let iy1 = y_end1 as i32;
        let ix1 = ipart(x_end1);
        plot_pixel(buf, ix1, iy1, color, rfpart(x_end1) * gap1);
        plot_pixel(buf, ix1 + 1, iy1, color, fpart(x_end1) * gap1);
        let mut inter_x = x_end1 + gradient;

        let y_end2 = round_f32(y2);
        let x_end2 = x2 + gradient * (y_end2 - y2);
        let gap2 = fpart(y2 + 0.5);
        let iy2 = y_end2 as i32;
        let ix2 = ipart(x_end2);
        plot_pixel(buf, ix2, iy2, color, rfpart(x_end2) * gap2);
        plot_pixel(buf, ix2 + 1, iy2, color, fpart(x_end2) * gap2);

        for y in (iy1 + 1)..iy2 {
            plot_pixel(buf, ipart(inter_x), y, color, rfpart(inter_x));
            plot_pixel(buf, ipart(inter_x) + 1, y, color, fpart(inter_x));
            inter_x += gradient;
        }
    }
}

fn draw_line_aa_distance(
    buf: &mut [Rgb565],
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    thickness: f32,
    color: Rgb565,
) {
    let vx = x2 - x1;
    let vy = y2 - y1;
    let len2 = vx * vx + vy * vy;
    let radius = thickness * 0.5;
    let max_dist = radius + 0.5;
    let max_dist2 = max_dist * max_dist;
    let min_dist = (radius - 0.5).max(0.0);
    let min_dist2 = min_dist * min_dist;

    // Zero-length line (single point dot)
    if len2 < 1e-6 {
        let min_x = (floor_f32(x1 - max_dist) as i32).max(0);
        let max_x = (ceil_f32(x1 + max_dist) as i32).min(RENDER_W as i32 - 1);
        let min_y = (floor_f32(y1 - max_dist) as i32).max(0);
        let max_y = (ceil_f32(y1 + max_dist) as i32).min(RENDER_H as i32 - 1);

        for py in min_y..=max_y {
            let cy = py as f32 + 0.5;
            let dy = cy - y1;
            let dy2 = dy * dy;
            let row_idx = (py as usize) * (RENDER_W as usize);

            for px in min_x..=max_x {
                let cx = px as f32 + 0.5;
                let dx = cx - x1;
                let dist2 = dx * dx + dy2;

                if dist2 >= max_dist2 {
                    continue;
                }

                let alpha = if dist2 <= min_dist2 {
                    255
                } else {
                    let dist = sqrt_fast(dist2);
                    let a = max_dist - dist;
                    ((a * a) * 255.0 + 0.5) as u8
                };

                let idx = row_idx + (px as usize);
                buf[idx] = blend_rgb565(buf[idx], color, alpha);
            }
        }
        return;
    }

    let len = sqrt_fast(len2);
    let inv_len2 = 1.0 / len2;

    let min_x = (floor_f32(x1.min(x2) - max_dist) as i32).max(0);
    let max_x = (ceil_f32(x1.max(x2) + max_dist) as i32).min(RENDER_W as i32 - 1);
    let min_y = (floor_f32(y1.min(y2) - max_dist) as i32).max(0);
    let max_y = (ceil_f32(y1.max(y2) + max_dist) as i32).min(RENDER_H as i32 - 1);

    if min_x > max_x || min_y > max_y {
        return;
    }

    // Conservative scanline slab bounds:
    // Slashes horizontal scanning width to a narrow strip along diagonal/slanted lines.
    let use_slab = vy.abs() > 1e-4;
    let (x_step, x_half_span) = if use_slab {
        let inv_vy = 1.0 / vy;
        let step = vx * inv_vy;
        let half = (max_dist * len * inv_vy).abs() + 0.5;
        (step, half)
    } else {
        (0.0, 0.0)
    };

    for py in min_y..=max_y {
        let cy = py as f32 + 0.5;
        let wy = cy - y1;
        let wy_vy = wy * vy;
        let row_idx = (py as usize) * (RENDER_W as usize);

        // Calculate clamped X-extent for this specific scanline
        let (row_min_x, row_max_x) = if use_slab {
            let x_center = x1 + wy * x_step;
            let r_min = (floor_f32(x_center - x_half_span) as i32).max(min_x);
            let r_max = (ceil_f32(x_center + x_half_span) as i32).min(max_x);
            (r_min, r_max)
        } else {
            (min_x, max_x)
        };

        for px in row_min_x..=row_max_x {
            let cx = px as f32 + 0.5;
            let wx = cx - x1;

            // Clamped projection using precomputed inv_len2 (multiplication instead of division)
            let dot = wx * vx + wy_vy;
            let t = (dot * inv_len2).clamp(0.0, 1.0);
            let qx = x1 + t * vx;
            let qy = y1 + t * vy;

            let dx = cx - qx;
            let dy = cy - qy;
            let dist2 = dx * dx + dy * dy;

            // 1. FAST REJECTION: Skip outside pixels in ~10 cycles without taking a square root
            if dist2 >= max_dist2 {
                continue;
            }

            // 2. OPAQUE SHORTCUT: Full opacity core skips sqrt completely
            let alpha = if dist2 <= min_dist2 {
                255
            } else {
                // 3. AA FRINGE: Hardware sqrt for only the 1px edge fringe
                let dist = sqrt_fast(dist2);
                let a = max_dist - dist;
                ((a * a) * 255.0 + 0.5) as u8
            };

            let idx = row_idx + (px as usize);
            buf[idx] = blend_rgb565(buf[idx], color, alpha);
        }
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
        [
            Some(IcKey::Num7),
            Some(IcKey::Num8),
            Some(IcKey::Num9),
            Some(IcKey::Func3),
        ],
        [
            Some(IcKey::Num4),
            Some(IcKey::Num5),
            Some(IcKey::Num6),
            Some(IcKey::Func4),
        ],
        [
            Some(IcKey::Num1),
            Some(IcKey::Num2),
            Some(IcKey::Num3),
            Some(IcKey::Func5),
        ],
        [
            Some(IcKey::Num0),
            Some(IcKey::Shift),
            Some(IcKey::Super),
            Some(IcKey::Func6),
        ],
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

    // With every row low, a pressed switch holds its column low. This gives
    // the column GPIOs a stable level to use as an idle wake source.
    fn all_rows_low(&mut self) {
        for row in self.rows.iter_mut() {
            row.set_low();
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
            // if this delay isn't here, theres lots of weird inputs that
            // happen with the key on the next row
            cortex_m::asm::delay(100);
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

    fn scan_and_send(&mut self, channel_sender: DynamicSender<'_, InputBufferEvent>) -> bool {
        let current_pressed = self.scan();
        let mut changed = false;
        for idx in 0..IcKey::COUNT {
            if current_pressed[idx] != self.prev_pressed[idx] {
                changed = true;
                if let Some(key) = Self::key_from_index(idx) {
                    if let Err(_) = channel_sender.try_send(InputBufferEvent {
                        key: key,
                        movement: if current_pressed[idx] {
                            KeyMovement::Down
                        } else {
                            KeyMovement::Up
                        },
                    }) {
                        warn!("Input buffer overflowed");
                    };
                }
            }
        }
        self.prev_pressed = current_pressed;
        changed
    }

    fn has_pressed_keys(&self) -> bool {
        self.prev_pressed.iter().any(|&pressed| pressed)
    }

    async fn wait_for_key_press(&mut self) {
        // A level-low wait intentionally also completes when a key is already
        // down. That closes the race between putting the rows into their idle
        // state and arming the GPIO interrupts.
        self.all_rows_low();
        info!("input idle: waiting for a matrix-column interrupt");
        let [col0, col1, col2, col3] = &mut self.cols;
        select4(
            col0.wait_for_low(),
            col1.wait_for_low(),
            col2.wait_for_low(),
            col3.wait_for_low(),
        )
        .await;
        info!("input wake: matrix column asserted; scanning keys");
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
        MS_BEFORE_BOOT,
        gpio_pin_enabled,
        led_gpio_num,
    );
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
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

    // I2S audio init -----
    let Pio {
        common: mut pio_common,
        sm0,
        ..
    } = Pio::new(p.PIO0, Irqs);
    let audio_bit_clock_pin = p.PIN_29; // AKA BCLK or SCK
    let audio_lr_clock_pin = p.PIN_30; // AKA LRCLK or WS
    let audio_data_pin = p.PIN_28; // AKA DIN or SD
    let audio_pio_program = PioI2sOutProgram::new(&mut pio_common);
    // todo: in latest embassy-rp version 0.10.0, Irqs gets passed
    // (see https://github.com/embassy-rs/embassy/pull/5338). Also
    // makes it so you have to call i2c.start(). Try
    // updating embassy-rp to that version, and seeing
    // how that goes.
    // If upgrade successful and having audio trouble, look at this discussion:
    // https://github.com/embassy-rs/embassy/pull/5388
    let i2s = PioI2sOut::new(
        &mut pio_common,
        sm0,
        p.DMA_CH0,
        audio_data_pin,
        audio_bit_clock_pin,
        audio_lr_clock_pin,
        AUDIO_SAMPLE_RATE,
        AUDIO_BIT_DEPTH,
        &audio_pio_program,
    );
    info!(
        "I2S configured: {} Hz, {}-bit stereo, {} frames/buffer ({} us/buffer)",
        AUDIO_SAMPLE_RATE, AUDIO_BIT_DEPTH, AUDIO_BUFFER_SIZE, AUDIO_BUFFER_DURATION_US,
    );
    // --------------------

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
    // row/col
    // row0 gpio13
    // row1 gpio14
    // row2 gpio15
    // row3 gpio16
    // row4 gpio17
    // col0 gpio21
    // col1 gpio20
    // col2 gpio19
    // col3 gpio18
    // row  column  button num  button func
    // row0 col0    no button present
    // row0 col1    no button present
    // row0 col2    switch 1    Func1
    // row0 col3    switch 2    Func2
    // row1 col0    switch 3    Num7
    // row1 col1    switch 4    Num8
    // row1 col2    switch 5    Num9
    // row1 col3    switch 6    Func3
    // row2 col0    switch 7    Num4
    // row2 col1    switch 8    Num5
    // row2 col2    switch 9    Num6
    // row2 col3    switch 10   Func4
    // row3 col0    switch 11   Num1
    // row3 col1    switch 12   Num2
    // row3 col2    switch 13   Num3
    // row3 col3    switch 14   Func5
    // row4 col0    switch 15   Num0
    // row4 col1    switch 16   Shift
    // row4 col2    switch 17   Super
    // row4 col3    switch 18   Func6
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

    if matrix_cols[2].is_low() {
        reboot_into_bootloader();
    }

    let _led = Output::new(p.PIN_22, Level::Low);
    let rst = p.PIN_47;
    let display_cs = p.PIN_45;
    let dcx = p.PIN_42;
    let mosi = p.PIN_43;
    let clk = p.PIN_46;
    let module_bl = p.PIN_31;
    let bare_display_bl = p.PIN_41;
    let lcd_spi_bus = p.SPI1;
    let _audio_shutdown_n = Output::new(p.PIN_27, Level::High);

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
    pwm_config.compare_b = 0xFFFF / 2;
    let backlight = Pwm::new_output_b(p.PWM_SLICE7, module_bl, pwm_config);
    let mut pwm_config2 = PwmConfig::default();
    pwm_config2.top = 0xFFFF;
    pwm_config2.compare_b = 0xFFFF / 2;
    let backlight2 = Pwm::new_output_b(p.PWM_SLICE8, bare_display_bl, pwm_config2);

    // create SPI
    let mut display_config = spi::Config::default();
    display_config.frequency = DISPLAY_FREQ;
    display_config.phase = spi::Phase::CaptureOnSecondTransition;
    display_config.polarity = spi::Polarity::IdleHigh;

    let spi = Spi::new_txonly(lcd_spi_bus, clk, mosi, p.DMA_CH1, display_config.clone());
    let spi_bus: &'static AsyncMutex<NoopRawMutex, Spi<'static, SPI1, spi::Async>> =
        DISPLAY_SPI_BUS.init(AsyncMutex::new(spi));

    let display_spi = SpiDeviceWithConfig::new(
        spi_bus,
        Output::new(display_cs, Level::High),
        display_config,
    );

    // dcx: 0 = command, 1 = data
    let dcx = Output::new(dcx, Level::Low);
    let rst = Output::new(rst, Level::Low);

    // display interface abstraction from the DMA-backed SPI device and DC pin
    let di = SpiInterface::new(display_spi, dcx);

    // Define the display from the display interface and initialize it
    let display: LcdDisplay = Builder::new(ST7789, di)
        .display_size(240, 320)
        .reset_pin(rst)
        .orientation(Orientation::new().rotate(Rotation::Deg270))
        .invert_colors(ColorInversion::Inverted)
        .init(&mut Delay)
        .await
        .unwrap();

    // set up i2c
    // Default I2C config enables internal pull-up resistors.
    let i2c_cfg = embassy_rp::i2c::Config::default();
    let board_i2c = embassy_rp::i2c::I2c::new_blocking(p.I2C0, p.PIN_25, p.PIN_24, i2c_cfg);

    let buf0 = AudioBuffer {
        samples: AUDIO_DMA0.init([0; AUDIO_BUFFER_SIZE]),
    };

    let buf1 = AudioBuffer {
        samples: AUDIO_DMA1.init([0; AUDIO_BUFFER_SIZE]),
    };

    // Initially both buffers are empty.
    EMPTY_BUFFERS.send(buf0).await;
    EMPTY_BUFFERS.send(buf1).await;

    // The UI starts with one canvas; the other is immediately available for a
    // future frame once the display task has finished with it.
    let initial_canvas = FrameBuffer {
        pixels: CANVAS_DATA0.take(),
    };
    let spare_canvas = FrameBuffer {
        pixels: CANVAS_DATA1.take(),
    };
    FREE_FRAME_BUFFERS.send(spare_canvas).await;

    // This board uses a MAX17048 battery fuel gauge
    let fuel_gauge: Max17048<BoardI2c> = Max17048::new(board_i2c);
    unwrap!(spawner.spawn(battery_task(fuel_gauge)));
    unwrap!(spawner.spawn(audio_task(i2s)));
    unwrap!(spawner.spawn(display_task(display)));

    spawn_core1(
        p.CORE1,
        unsafe {
            // consider changing to new syntax: `&mut *&raw mut CORE1_STACK`
            &mut *core::ptr::addr_of_mut!(CORE1_STACK)
        },
        move || {
            let exec1 = EXECUTOR1.init(Executor::new());
            exec1.run(|spawner| {
                unwrap!(spawner.spawn(inputs_core1_task(matrix_rows, matrix_cols)));
            });
        },
    );

    let mut icalc: IcShell = IcShell::new();
    let mut pcm_buffer = [0i16; AUDIO_BUFFER_SIZE];
    let mut ic_rp_platform = IcRpPlatform::new(backlight, backlight2, initial_canvas);

    // render the first frame
    icalc.update(&mut ic_rp_platform);
    READY_FRAME_BUFFERS.send(ic_rp_platform.take_canvas()).await;

    let mut next_audio_report = None;
    let mut screen_dirty = false;
    let mut audio_is_running = false;
    info!("audio idle: no active voices; waiting for input");
    loop {
        // When the synth is silent, wait only for input. While it is active,
        // wake for either fresh input or a DMA buffer that needs rendering.
        // The executor can therefore enter WFI instead of waking every 1 ms
        // to generate silence.
        let (first_input, mut audio_buffer, realtime_tick) =
            match (icalc.has_active_audio(), icalc.requires_realtime_updates()) {
                (true, true) => match select3(
                    INPUT_BUFFER.receive(),
                    EMPTY_BUFFERS.receive(),
                    Timer::after_millis(16),
                )
                .await
                {
                    Either3::First(event) => (Some(event), None, false),
                    Either3::Second(buffer) => (None, Some(buffer), false),
                    Either3::Third(()) => (None, None, true),
                },
                (true, false) => {
                    match select(INPUT_BUFFER.receive(), EMPTY_BUFFERS.receive()).await {
                        Either::First(event) => (Some(event), None, false),
                        Either::Second(buffer) => (None, Some(buffer), false),
                    }
                }
                (false, true) => {
                    match select(INPUT_BUFFER.receive(), Timer::after_millis(16)).await {
                        Either::First(event) => (Some(event), None, false),
                        Either::Second(()) => (None, None, true),
                    }
                }
                (false, false) => (Some(INPUT_BUFFER.receive().await), None, false),
            };

        let mut inputs_changed = false;
        if let Some(event) = first_input {
            match event.movement {
                KeyMovement::Up => {
                    debug!("input key={} up", event.key as usize);
                    icalc.key_up(event.key);
                }
                KeyMovement::Down => {
                    debug!("input key={} down", event.key as usize);
                    icalc.key_down(event.key);
                }
            }
            inputs_changed = true;
        }
        while let Ok(event) = INPUT_BUFFER.try_receive() {
            match event.movement {
                KeyMovement::Up => {
                    debug!("input key={} up", event.key as usize);
                    icalc.key_up(event.key);
                }
                KeyMovement::Down => {
                    debug!("input key={} down", event.key as usize);
                    icalc.key_down(event.key);
                }
            }
            inputs_changed = true;
        }
        if inputs_changed || realtime_tick {
            screen_dirty = true;
        }

        // `IcShell::update` creates and releases notes as part of handling
        // input, so perform it before deciding whether audio needs to run.
        // If both canvases are busy, await one rather than polling for it.
        if screen_dirty && !ic_rp_platform.has_canvas() {
            let canvas = FREE_FRAME_BUFFERS.receive().await;
            ic_rp_platform.set_canvas(canvas);
        }
        if screen_dirty {
            let update_start = Instant::now();
            icalc.update(&mut ic_rp_platform);
            let update_us = elapsed_us(update_start);
            SHELL_UPDATES.fetch_add(1, Ordering::Relaxed);
            record_max(&SHELL_UPDATE_MAX_US, update_us);
            if update_us > AUDIO_SLOW_WORK_WARN_US {
                warn!("shell update took {}us", update_us);
            }

            READY_FRAME_BUFFERS.send(ic_rp_platform.take_canvas()).await;
            screen_dirty = false;
        }

        if icalc.has_active_audio() && !audio_is_running {
            info!("audio active: resuming PCM rendering and I2S DMA");
            audio_is_running = true;
        }

        // Once silent, leave the empty buffers in their channel and let the
        // audio task block waiting for a future sound. This also lets the I2S
        // PIO stall instead of transmitting a continuous stream of zeroes.
        if audio_buffer.is_none() && icalc.has_active_audio() {
            audio_buffer = EMPTY_BUFFERS.try_receive().ok();
        }
        if let Some(buf) = audio_buffer {
            let render_start = Instant::now();
            icalc.fill_audio(&mut pcm_buffer);
            for (dst, &pcm_sample) in buf.samples.iter_mut().zip(pcm_buffer.iter()) {
                let sample_u16 = pcm_sample as u16 as u32;
                *dst = (sample_u16 << 16) | sample_u16;
            }
            let render_us = elapsed_us(render_start);
            AUDIO_BUFFERS_RENDERED.fetch_add(1, Ordering::Relaxed);
            record_max(&AUDIO_RENDER_MAX_US, render_us);
            if render_us > AUDIO_BUFFER_DURATION_US {
                warn!(
                    "audio render took {}us (buffer duration {}us)",
                    render_us, AUDIO_BUFFER_DURATION_US,
                );
            }

            let queue_send_start = Instant::now();
            FILLED_BUFFERS.send(buf).await;
            record_max(&AUDIO_QUEUE_SEND_MAX_US, elapsed_us(queue_send_start));
        }

        if icalc.has_active_audio() {
            let next_report = next_audio_report
                .get_or_insert_with(|| Instant::now() + embassy_time::Duration::from_secs(1));
            if Instant::now() >= *next_report {
                report_audio_diagnostics();
                *next_report = Instant::now() + embassy_time::Duration::from_secs(1);
            }
        } else {
            next_audio_report = None;
            if audio_is_running {
                info!("audio idle: release tail finished; PCM rendering and I2S DMA stopped");
                audio_is_running = false;
            }
        }
    }
}

#[embassy_executor::task]
async fn display_task(mut display: LcdDisplay) {
    loop {
        let canvas = READY_FRAME_BUFFERS.receive().await;
        let display_start = Instant::now();
        display
            .show_raw_data(0, 0, RENDER_W as u16, RENDER_H as u16, &canvas.pixels[..])
            .await
            .unwrap();
        let display_us = elapsed_us(display_start);
        DISPLAY_TRANSFERS.fetch_add(1, Ordering::Relaxed);
        record_max(&DISPLAY_TRANSFER_MAX_US, display_us);

        // The UI may draw into this canvas only after DMA has completed.
        FREE_FRAME_BUFFERS.send(canvas).await;
    }
}

#[embassy_executor::task]
async fn battery_task(mut fuel_gauge: Max17048<BoardI2c>) {
    loop {
        if let Ok(soc) = fuel_gauge.soc() {
            BATTERY_SOC.store(soc as i32, core::sync::atomic::Ordering::Relaxed);
        } else {
            warn!("Error getting battery soc from fuel gauge");
        }
        Timer::after_secs(10).await;
    }
}

#[embassy_executor::task]
async fn audio_task(mut i2s: PioI2sOut<'static, PIO0, 0>) {
    let mut has_started = false;
    loop {
        // `write` only returns after the DMA transfer has drained. If no next
        // buffer is queued at this point, the PIO runs out of FIFO data and an
        // audible gap is expected. Do not log here: RTT logging in this task
        // would itself perturb the timing.
        let buf = match FILLED_BUFFERS.try_receive() {
            Ok(buf) => buf,
            Err(_) => {
                let wait_start = Instant::now();
                let buf = FILLED_BUFFERS.receive().await;
                // Waiting for the very first buffer is expected during boot;
                // after that it means the PIO had no next DMA transfer ready.
                if has_started {
                    AUDIO_STARVATIONS.fetch_add(1, Ordering::Relaxed);
                    record_max(&AUDIO_STARVATION_MAX_US, elapsed_us(wait_start));
                }
                buf
            }
        };

        let dma_start = Instant::now();
        i2s.write(buf.samples).await;
        let dma_us = elapsed_us(dma_start);
        AUDIO_DMA_TRANSFERS.fetch_add(1, Ordering::Relaxed);
        record_max(&AUDIO_DMA_MAX_US, dma_us);
        if dma_us > AUDIO_BUFFER_DURATION_US + AUDIO_FIFO_COVERAGE_US {
            AUDIO_LATE_DMA_COMPLETIONS.fetch_add(1, Ordering::Relaxed);
        }
        has_started = true;
        // ok now you have like 100 uS to start a new DMA transfer before
        // there's a pop (since the PIO only holds like 8 samples in its FIFO)
        EMPTY_BUFFERS.send(buf).await;
    }
}

#[embassy_executor::task]
async fn inputs_core1_task(matrix_rows: [Output<'static>; 5], matrix_cols: [Input<'static>; 4]) {
    info!("Hello from the \"inputs core\"");
    let mut key_matrix = KeyMatrix::new(matrix_rows, matrix_cols);
    let input_buf_sender = INPUT_BUFFER.dyn_sender();
    loop {
        // Release events cannot wake the matrix while every row is low, so
        // keep scanning only until all pressed keys have been released.
        // Otherwise the core-1 executor sleeps in the GPIO interrupt wait.
        if key_matrix.has_pressed_keys() {
            Timer::after_millis(16).await;
        } else {
            key_matrix.wait_for_key_press().await;
        }

        key_matrix.scan_and_send(input_buf_sender);
        if key_matrix.is_pressed(IcKey::Super) && key_matrix.is_pressed(IcKey::Shift) {
            reboot_into_bootloader();
        }
    }
}
