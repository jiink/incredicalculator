//! Scripted on-device audio stress test, enabled by the `audio-bench` feature.
//!
//! Injects key events as if typed: opens Scale keys from the menu, turns the
//! arp on, then holds a chord under each font and logs how often the I2S DMA
//! ran out of filled buffers (each one is an audible gap).

use core::sync::atomic::Ordering;

use defmt::info;
use embassy_time::{Instant, Timer};
use incredicalculator_core::fonts::{self, FontId};
use incredicalculator_core::input::IcKey;

use crate::{
    AUDIO_STARVATION_US_TOTAL, AUDIO_STARVATIONS_TOTAL, INPUT_BUFFER, InputBufferEvent,
    KeyMovement, SHELL_UPDATE_US_TOTAL, SHELL_UPDATES_TOTAL,
};

async fn send(key: IcKey, movement: KeyMovement) {
    INPUT_BUFFER.send(InputBufferEvent { key, movement }).await;
}

async fn tap(key: IcKey) {
    send(key, KeyMovement::Down).await;
    Timer::after_millis(150).await;
    send(key, KeyMovement::Up).await;
    Timer::after_millis(400).await;
}

struct Totals {
    starvations: u32,
    starvation_us: u32,
    updates: u32,
    update_us: u32,
}

fn totals() -> Totals {
    Totals {
        starvations: AUDIO_STARVATIONS_TOTAL.load(Ordering::Relaxed),
        starvation_us: AUDIO_STARVATION_US_TOTAL.load(Ordering::Relaxed),
        updates: SHELL_UPDATES_TOTAL.load(Ordering::Relaxed),
        update_us: SHELL_UPDATE_US_TOTAL.load(Ordering::Relaxed),
    }
}

const CHORD: [IcKey; 3] = [IcKey::Num1, IcKey::Num3, IcKey::Num5];
const HOLD_MS: u64 = 8_000;

#[embassy_executor::task]
pub async fn audio_bench_task() {
    Timer::after_secs(4).await;
    info!("bench: opening Scale keys");
    // Super + Shift together opens the menu from the boot app.
    send(IcKey::Super, KeyMovement::Down).await;
    send(IcKey::Shift, KeyMovement::Down).await;
    Timer::after_millis(150).await;
    send(IcKey::Super, KeyMovement::Up).await;
    send(IcKey::Shift, KeyMovement::Up).await;
    Timer::after_millis(400).await;
    tap(IcKey::Num3).await; // Next page
    if option_env!("AUDIO_BENCH_APP") == Some("Bytebeat") {
        return bytebeat().await;
    }
    tap(IcKey::Num7).await; // First app on page 2: Scale keys
    tap(IcKey::Backspace).await; // Arp: Off -> Up

    let mut round = 0u32;
    loop {
        for font in [FontId::Classic, FontId::Futural, FontId::Times] {
            // `AUDIO_BENCH_FONT=Times` (at build time) benches only that font.
            if option_env!("AUDIO_BENCH_FONT").is_some_and(|only| only != font.label()) {
                continue;
            }
            fonts::set_system_font(font);
            Timer::after_millis(300).await;
            let before = totals();
            let start = Instant::now();
            for key in CHORD {
                send(key, KeyMovement::Down).await;
            }
            Timer::after_millis(HOLD_MS).await;
            for key in CHORD {
                send(key, KeyMovement::Up).await;
            }
            // Let the release tail play out; gaps there count too.
            Timer::after_millis(2_000).await;
            let after = totals();
            let updates = after.updates.wrapping_sub(before.updates);
            let update_us = after.update_us.wrapping_sub(before.update_us);
            info!(
                "BENCH round={} font={} ms={} starve={} starve_total_us={} updates={} avg_update_us={}",
                round,
                font.label(),
                start.elapsed().as_millis(),
                after.starvations.wrapping_sub(before.starvations),
                after.starvation_us.wrapping_sub(before.starvation_us),
                updates,
                update_us / updates.max(1),
            );
        }
        round += 1;
    }
}

/// `AUDIO_BENCH_APP=Bytebeat` (at build time) instead plays Bytebeat, whose
/// samples come from the app on core 0 rather than from the synth.
async fn bytebeat() {
    tap(IcKey::Num9).await; // Third app on page 2: Bytebeat
    tap(IcKey::Equals).await; // Play the preset expression
    let mut round = 0u32;
    loop {
        for font in [FontId::Classic, FontId::Futural, FontId::Times] {
            fonts::set_system_font(font);
            let before = totals();
            Timer::after_millis(HOLD_MS).await;
            let after = totals();
            let updates = after.updates.wrapping_sub(before.updates);
            info!(
                "BENCH bytebeat round={} font={} starve={} starve_total_us={} updates={} avg_update_us={}",
                round,
                font.label(),
                after.starvations.wrapping_sub(before.starvations),
                after.starvation_us.wrapping_sub(before.starvation_us),
                updates,
                after.update_us.wrapping_sub(before.update_us) / updates.max(1),
            );
        }
        round += 1;
    }
}
