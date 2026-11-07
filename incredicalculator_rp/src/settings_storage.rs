//! Persists the shell's settings blob in a reserved region at the end of flash.
use core::ops::Range;

use defmt::*;
use embassy_rp::flash::ERASE_SIZE;
use incredicalculator_core::platform::SETTINGS_STORAGE_LEN;
use sequential_storage::cache::{Cache, Uncached};
use sequential_storage::map::{MapConfig, MapStorage};

use crate::script_drive::{FLASH_SIZE, SETTINGS_SECTORS, SharedFlash};

// The last sectors of flash, above the script drive.
const SETTINGS_FLASH_RANGE: Range<u32> =
    (FLASH_SIZE - SETTINGS_SECTORS * ERASE_SIZE) as u32..FLASH_SIZE as u32;

const SETTINGS_KEY: u8 = 0;
// Holds the key and value while (de)serializing; the crate rounds up to the
// flash word size internally, so a little headroom is plenty.
const DATA_BUFFER_LEN: usize = SETTINGS_STORAGE_LEN + 16;

pub struct SettingsStorage {
    map: MapStorage<u8, SharedFlash, Cache<Uncached, Uncached, Uncached, u8>>,
}

impl SettingsStorage {
    pub fn new() -> Self {
        Self {
            map: MapStorage::new(
                SharedFlash,
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
