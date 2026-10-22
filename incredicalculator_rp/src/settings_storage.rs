//! Persists the shell's settings blob in a reserved region at the end of flash.
use core::ops::Range;

use defmt::*;
use embassy_rp::flash::{Async, ERASE_SIZE, Flash};
use embassy_rp::peripherals::FLASH;
use incredicalculator_core::platform::SETTINGS_STORAGE_LEN;
use sequential_storage::cache::{Cache, Uncached};
use sequential_storage::map::{MapConfig, MapStorage};

// Must match the 2 MiB assumed by memory.x, whose FLASH region stops
// `SETTINGS_SECTORS` sectors short so firmware never overlaps this range.
const FLASH_SIZE: usize = 2 * 1024 * 1024;
const SETTINGS_SECTORS: usize = 4;
const SETTINGS_FLASH_RANGE: Range<u32> =
    (FLASH_SIZE - SETTINGS_SECTORS * ERASE_SIZE) as u32..FLASH_SIZE as u32;

const SETTINGS_KEY: u8 = 0;
// Holds the key and value while (de)serializing; the crate rounds up to the
// flash word size internally, so a little headroom is plenty.
const DATA_BUFFER_LEN: usize = SETTINGS_STORAGE_LEN + 16;

pub type BoardFlash = Flash<'static, FLASH, Async, FLASH_SIZE>;

pub struct SettingsStorage {
    map: MapStorage<u8, BoardFlash, Cache<Uncached, Uncached, Uncached, u8>>,
}

impl SettingsStorage {
    pub fn new(flash: BoardFlash) -> Self {
        Self {
            map: MapStorage::new(
                flash,
                const { MapConfig::new(SETTINGS_FLASH_RANGE) },
                Cache::new_uncached(),
            ),
        }
    }

    pub async fn load(&mut self) -> Option<[u8; SETTINGS_STORAGE_LEN]> {
        let mut buffer = [0; DATA_BUFFER_LEN];
        match self.map.fetch_item(&mut buffer, &SETTINGS_KEY).await {
            Ok(data) => data,
            Err(e) => {
                warn!("settings: flash read failed: {}", e);
                None
            }
        }
    }

    pub async fn save(&mut self, data: &[u8; SETTINGS_STORAGE_LEN]) {
        let mut buffer = [0; DATA_BUFFER_LEN];
        match self.map.store_item(&mut buffer, &SETTINGS_KEY, data).await {
            Ok(()) => info!("settings: saved to flash"),
            Err(e) => warn!("settings: flash write failed: {}", e),
        }
    }
}
