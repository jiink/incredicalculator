//! Register-level driver for the board's MAX17048 battery fuel gauge.
//!
//! The gauge's open-drain ALRT output is latched low by an alert and stays low
//! until [`FuelGauge::clear_alert`] releases it. With [`FuelGauge::enable_alert`]
//! it alerts on every 1% change in state of charge, so the battery level can be
//! read on demand instead of polled.
use embassy_rp::i2c::{Blocking, Error, I2c};
use embassy_rp::peripherals::I2C0;

// Registers (datasheet table 2).
const ADDR: u8 = 0x36;
const VCELL: u8 = 0x02;
const SOC: u8 = 0x04;
const VERSION: u8 = 0x08;
const CONFIG: u8 = 0x0C;
const VALRT: u8 = 0x14;
const CRATE: u8 = 0x16;
const VRESET_ID: u8 = 0x18;
const STATUS: u8 = 0x1A;
/// CONFIG.SLEEP: the gauge is forced into sleep.
const CONFIG_SLEEP: u16 = 1 << 7;
/// CONFIG.ALSC: alert whenever the state of charge changes by 1%.
const CONFIG_ALSC: u16 = 1 << 6;
/// CONFIG.ALRT: the alert latch that holds the ALRT pin low.
const CONFIG_ALRT: u16 = 1 << 5;
/// CONFIG.ATHD: the empty-alert threshold, as 32% minus this value.
const CONFIG_ATHD: u16 = 0x1F;
/// STATUS alert flags: SC, HD, VR, VL, VH and RI.
const STATUS_ALERT_FLAGS: u16 = 0x3F00;
/// Names of the STATUS alert flags, from bit 8 up.
const STATUS_FLAG_NAMES: [&str; 6] = [
    "RI (reset)",
    "VH (voltage high)",
    "VL (voltage low)",
    "VR (voltage reset)",
    "HD (SOC low)",
    "SC (SOC changed 1%)",
];

/// The names of the alert flags set in `flags`, as from
/// [`FuelGauge::clear_alert`].
pub fn alert_flag_names(flags: u16) -> impl Iterator<Item = &'static str> {
    STATUS_FLAG_NAMES
        .iter()
        .enumerate()
        .filter(move |(bit, _)| flags & (1 << (bit + 8)) != 0)
        .map(|(_, name)| *name)
}

/// One reading of the gauge's registers, in real units.
pub struct Readings {
    /// State of charge, in percent.
    pub soc: f32,
    /// Cell voltage, in volts.
    pub voltage: f32,
    /// Rate of change of the state of charge, in percent per hour.
    pub charge_rate: f32,
    pub version: u16,
    /// The ID set at the factory.
    pub id: u8,
    /// Compensation for the cell's chemistry.
    pub rcomp: u8,
    pub sleeping: bool,
    pub soc_change_alert: bool,
    /// Low-battery alert threshold, in percent.
    pub empty_alert: u8,
    /// Alert window for the cell voltage, in volts.
    pub voltage_alert_min: f32,
    pub voltage_alert_max: f32,
}

pub type BoardI2c = I2c<'static, I2C0, Blocking>;

pub struct FuelGauge {
    i2c: BoardI2c,
}

impl FuelGauge {
    pub fn new(i2c: BoardI2c) -> Self {
        Self { i2c }
    }

    /// Turns on the 1%-change alert.
    pub fn enable_alert(&mut self) -> Result<(), Error> {
        let config = self.read(CONFIG)?;
        self.write(CONFIG, config | CONFIG_ALSC)
    }

    /// The state of charge, in whole percent.
    pub fn soc(&mut self) -> Result<u8, Error> {
        // The high byte is whole percent; the low byte is 1/256ths.
        Ok((self.read(SOC)? >> 8) as u8)
    }

    /// Reads all the registers worth showing.
    pub fn readings(&mut self) -> Result<Readings, Error> {
        let config = self.read(CONFIG)?;
        let valrt = self.read(VALRT)?.to_be_bytes();
        Ok(Readings {
            soc: self.read(SOC)? as f32 / 256.0,
            voltage: self.read(VCELL)? as f32 * 78.125e-6,
            charge_rate: self.read(CRATE)? as i16 as f32 * 0.208,
            version: self.read(VERSION)?,
            id: self.read(VRESET_ID)? as u8,
            rcomp: (config >> 8) as u8,
            sleeping: config & CONFIG_SLEEP != 0,
            soc_change_alert: config & CONFIG_ALSC != 0,
            empty_alert: 32 - (config & CONFIG_ATHD) as u8,
            voltage_alert_min: valrt[0] as f32 * 0.02,
            voltage_alert_max: valrt[1] as f32 * 0.02,
        })
    }

    /// Releases the ALRT pin, returning the STATUS alert flags that were set
    /// (see [`alert_flag_names`]). The datasheet wants STATUS cleared before
    /// CONFIG.ALRT, or the alert re-latches.
    pub fn clear_alert(&mut self) -> Result<u16, Error> {
        let status = self.read(STATUS)?;
        let flags = status & STATUS_ALERT_FLAGS;
        if flags != 0 {
            self.write(STATUS, status & !STATUS_ALERT_FLAGS)?;
        }
        let config = self.read(CONFIG)?;
        if config & CONFIG_ALRT != 0 {
            self.write(CONFIG, config & !CONFIG_ALRT)?;
        }
        Ok(flags)
    }

    fn read(&mut self, reg: u8) -> Result<u16, Error> {
        let mut data = [0; 2];
        self.i2c.blocking_write_read(ADDR, &[reg], &mut data)?;
        Ok(u16::from_be_bytes(data))
    }

    fn write(&mut self, reg: u8, value: u16) -> Result<(), Error> {
        let [hi, lo] = value.to_be_bytes();
        self.i2c.blocking_write(ADDR, &[reg, hi, lo])
    }
}
