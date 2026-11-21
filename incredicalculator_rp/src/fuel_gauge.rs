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
const SOC: u8 = 0x04;
const CONFIG: u8 = 0x0C;
const STATUS: u8 = 0x1A;
/// CONFIG.ALSC: alert whenever the state of charge changes by 1%.
const CONFIG_ALSC: u16 = 1 << 6;
/// CONFIG.ALRT: the alert latch that holds the ALRT pin low.
const CONFIG_ALRT: u16 = 1 << 5;
/// STATUS alert flags: SC, HD, VR, VL, VH and RI.
const STATUS_ALERT_FLAGS: u16 = 0x3F00;

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

    /// Releases the ALRT pin. The datasheet wants STATUS cleared before
    /// CONFIG.ALRT, or the alert re-latches.
    pub fn clear_alert(&mut self) -> Result<(), Error> {
        let status = self.read(STATUS)?;
        if status & STATUS_ALERT_FLAGS != 0 {
            self.write(STATUS, status & !STATUS_ALERT_FLAGS)?;
        }
        let config = self.read(CONFIG)?;
        if config & CONFIG_ALRT != 0 {
            self.write(CONFIG, config & !CONFIG_ALRT)?;
        }
        Ok(())
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
