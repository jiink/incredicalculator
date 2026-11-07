//! Shares the script drive with a computer as a USB mass storage device
//! while the "USB Drive" app is open.
//!
//! USB is otherwise off: its PLL and clock are only running during a
//! session, which also holds the system clock at full speed. Sector writes
//! from the computer are cached one 4 KiB flash block at a time and flushed
//! when the computer asks (sync or eject), after a short pause in writing,
//! and when the session ends.
use core::cell::Cell;

use defmt::*;
use embassy_futures::select::{Either, select, select4};
use embassy_rp::interrupt::InterruptExt;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_rp::{Peri, bind_interrupts, pac};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant, Timer};
use embassy_usb::control::{InResponse, OutResponse, Recipient, Request, RequestType};
use embassy_usb::driver::{Endpoint, EndpointError, EndpointIn, EndpointOut};
use embassy_usb::{Builder, Handler};
use flash_disk::scsi::{self, Cbw, CswStatus, Data, Scsi};
use flash_disk::{BlockDevice, SECTOR_SIZE};
use incredicalculator_core::platform::UsbDriveStatus;

use crate::power;
use crate::script_drive::{self, with_disk};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

/// Full-speed bulk packets.
const PACKET_SIZE: usize = 64;
/// Cached writes are flushed once the computer has paused writing this long.
const IDLE_FLUSH_DELAY: Duration = Duration::from_millis(500);
// pid.codes test IDs: fine for a hobby device, but get a real VID/PID before
// distributing these widely.
const USB_VID: u16 = 0x1209;
const USB_PID: u16 = 0x0001;

/// `true` to share the drive, `false` to take it back.
static REQUEST: Signal<CriticalSectionRawMutex, bool> = Signal::new();
/// Signaled once a request has taken effect.
static DONE: Signal<CriticalSectionRawMutex, ()> = Signal::new();
/// The computer sent a Bulk-Only Mass Storage Reset.
static BOT_RESET: Signal<CriticalSectionRawMutex, ()> = Signal::new();
static STATUS: critical_section::Mutex<Cell<UsbDriveStatus>> =
    critical_section::Mutex::new(Cell::new(UsbDriveStatus {
        shared: false,
        connected: false,
        ejected: false,
        unformatted: false,
    }));

pub fn status() -> UsbDriveStatus {
    critical_section::with(|cs| STATUS.borrow(cs).get())
}

fn update_status(update: impl FnOnce(&mut UsbDriveStatus)) {
    critical_section::with(|cs| {
        let cell = STATUS.borrow(cs);
        let mut status = cell.get();
        update(&mut status);
        cell.set(status);
    });
}

/// Starts or ends sharing, returning once that is done. After sharing ends,
/// every write from the computer is in flash.
pub async fn set_shared(shared: bool) {
    DONE.reset();
    REQUEST.signal(shared);
    DONE.wait().await;
}

#[embassy_executor::task]
pub async fn usb_drive_task(mut usb: Peri<'static, USB>) {
    loop {
        if !REQUEST.wait().await {
            DONE.signal(());
            continue;
        }
        info!("usb drive: sharing");
        let clock_guard = power::keep_awake();
        script_drive::unmount();
        let unformatted = !script_drive::has_filesystem();
        start_usb_clocks();
        update_status(|status| {
            *status = UsbDriveStatus {
                shared: true,
                unformatted,
                ..UsbDriveStatus::default()
            }
        });
        DONE.signal(());

        run_session(usb.reborrow()).await;

        stop_usb();
        if let Err(_) = with_disk(|disk| disk.release_cache()) {
            error!("usb drive: flushing to flash failed");
        }
        update_status(|status| *status = UsbDriveStatus::default());
        drop(clock_guard);
        info!("usb drive: stopped");
        DONE.signal(());
    }
}

/// Runs the USB device until sharing is turned off.
async fn run_session(usb: Peri<'_, USB>) {
    let driver = Driver::new(usb, Irqs);
    let mut config = embassy_usb::Config::new(USB_VID, USB_PID);
    config.manufacturer = Some("Incredicalculator");
    config.product = Some("Incredicalculator script drive");
    // Mass storage devices need a serial number of at least 12 digits.
    config.serial_number = Some("000000000001");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    let mut config_descriptor = [0; 64];
    let mut bos_descriptor = [0; 32];
    let mut msos_descriptor = [0; 0];
    let mut control_buf = [0; 64];
    let mut control = Control {
        interface: 0,
        configured: false,
    };
    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buf,
    );
    let (mut ep_in, mut ep_out) = {
        let mut function = builder.function(
            scsi::CLASS_MASS_STORAGE,
            scsi::SUBCLASS_SCSI,
            scsi::PROTOCOL_BULK_ONLY,
        );
        let mut interface = function.interface();
        control.interface = interface.interface_number().into();
        let mut alt = interface.alt_setting(
            scsi::CLASS_MASS_STORAGE,
            scsi::SUBCLASS_SCSI,
            scsi::PROTOCOL_BULK_ONLY,
            None,
        );
        (
            alt.endpoint_bulk_in(None, PACKET_SIZE as u16),
            alt.endpoint_bulk_out(None, PACKET_SIZE as u16),
        )
    };
    builder.handler(&mut control);
    let mut device = builder.build();

    let last_write = Cell::new(Instant::now());
    BOT_RESET.reset();
    let bulk_only = async {
        loop {
            ep_out.wait_enabled().await;
            // A new connection or a reset starts from a fresh state.
            let mut scsi = Scsi::new(with_disk(|disk| disk.sector_count()));
            match select(
                serve(&mut ep_in, &mut ep_out, &mut scsi, &last_write),
                BOT_RESET.wait(),
            )
            .await
            {
                Either::First(Err(EndpointError::Disabled)) => {}
                Either::First(Err(EndpointError::BufferOverflow)) => {
                    warn!("usb drive: oversized packet")
                }
                Either::Second(()) => info!("usb drive: reset by host"),
            }
        }
    };
    let idle_flush = async {
        loop {
            Timer::after(IDLE_FLUSH_DELAY / 2).await;
            if last_write.get().elapsed() >= IDLE_FLUSH_DELAY {
                flush();
            }
        }
    };
    let stop = async {
        while REQUEST.wait().await {
            // Already sharing.
            DONE.signal(());
        }
    };
    select4(device.run(), bulk_only, idle_flush, stop).await;
}

/// Writes cached sectors to flash. Then, since the computer may have just
/// formatted the drive (or damaged it), checks for a filesystem again.
fn flush() {
    let result = with_disk(|disk| {
        if disk.is_dirty() {
            disk.flush().map(|()| true)
        } else {
            Ok(false)
        }
    });
    match result {
        Ok(true) => {
            let unformatted = !script_drive::has_filesystem();
            update_status(|status| status.unformatted = unformatted);
        }
        Ok(false) => {}
        Err(_) => error!("usb drive: flushing to flash failed"),
    }
}

/// Handles Bulk-Only Transport commands until an endpoint error.
async fn serve(
    ep_in: &mut impl EndpointIn,
    ep_out: &mut impl EndpointOut,
    scsi: &mut Scsi,
    last_write: &Cell<Instant>,
) -> Result<core::convert::Infallible, EndpointError> {
    let mut packet = [0u8; PACKET_SIZE];
    let mut sector = [0u8; SECTOR_SIZE];
    loop {
        let len = ep_out.read(&mut packet).await?;
        let Some(cbw) = Cbw::parse(&packet[..len]) else {
            // Hosts only send garbage when out of step; they reset after.
            warn!("usb drive: invalid CBW ({} bytes)", len);
            continue;
        };
        let mut reply = [0u8; scsi::REPLY_BUF_LEN];
        let result = scsi.command(&cbw.command, &mut reply);
        let mut transfer = cbw.transfer(result);
        let mut sent = 0usize;
        match transfer.data {
            Data::None => {}
            Data::Reply { len } => {
                write_all(ep_in, &reply[..len]).await?;
                sent = len;
            }
            Data::Read { lba, blocks } => {
                for i in 0..blocks {
                    if with_disk(|disk| disk.read_sector(lba + i, &mut sector)).is_err() {
                        // Keep the transfer the promised length.
                        sector.fill(0);
                        scsi.set_medium_error();
                        transfer.status = CswStatus::Failed;
                    }
                    write_all(ep_in, &sector).await?;
                }
                sent = blocks as usize * SECTOR_SIZE;
            }
            Data::Write { lba, blocks } => {
                for i in 0..blocks {
                    for chunk in sector.chunks_mut(PACKET_SIZE) {
                        read_exact(ep_out, chunk).await?;
                    }
                    if with_disk(|disk| disk.write_sector(lba + i, &sector)).is_err() {
                        scsi.set_medium_error();
                        transfer.status = CswStatus::Failed;
                    }
                    last_write.set(Instant::now());
                }
            }
        }
        if transfer.pad_in && sent % PACKET_SIZE == 0 {
            ep_in.write(&[]).await?;
        }
        let mut discard = transfer.discard_out as usize;
        while discard > 0 {
            let len = ep_out.read(&mut packet).await?;
            discard = discard.saturating_sub(len);
            if len < PACKET_SIZE {
                break;
            }
        }
        if scsi.flush_requested {
            scsi.flush_requested = false;
            flush();
        }
        let ejected = scsi.ejected;
        update_status(|status| status.ejected = ejected);
        ep_in
            .write(&cbw.status(transfer.residue, transfer.status))
            .await?;
    }
}

async fn write_all(ep_in: &mut impl EndpointIn, data: &[u8]) -> Result<(), EndpointError> {
    for chunk in data.chunks(PACKET_SIZE) {
        ep_in.write(chunk).await?;
    }
    Ok(())
}

async fn read_exact(ep_out: &mut impl EndpointOut, buf: &mut [u8]) -> Result<(), EndpointError> {
    let mut filled = 0;
    while filled < buf.len() {
        // Reads take whole packets, so read into a packet-sized buffer.
        let mut packet = [0u8; PACKET_SIZE];
        let len = ep_out.read(&mut packet).await?;
        let take = len.min(buf.len() - filled);
        buf[filled..filled + take].copy_from_slice(&packet[..take]);
        filled += take;
        if len < PACKET_SIZE && filled < buf.len() {
            // A short packet ends the transfer early.
            break;
        }
    }
    Ok(())
}

/// Class requests on the control endpoint, and connection tracking.
struct Control {
    interface: u8,
    configured: bool,
}

impl Control {
    fn is_ours(&self, req: &Request) -> bool {
        req.request_type == RequestType::Class
            && req.recipient == Recipient::Interface
            && req.index == self.interface as u16
    }
}

impl Handler for Control {
    fn reset(&mut self) {
        self.configured = false;
        update_status(|status| status.connected = false);
    }

    fn configured(&mut self, configured: bool) {
        self.configured = configured;
        update_status(|status| {
            status.connected = configured;
            status.ejected = false;
        });
    }

    fn suspended(&mut self, suspended: bool) {
        // Unplugging looks like a suspend: the device can't sense VBUS.
        let connected = !suspended && self.configured;
        update_status(|status| status.connected = connected);
    }

    fn control_out(&mut self, req: Request, _data: &[u8]) -> Option<OutResponse> {
        if !self.is_ours(&req) || req.request != scsi::REQUEST_RESET {
            return None;
        }
        BOT_RESET.signal(());
        Some(OutResponse::Accepted)
    }

    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        if !self.is_ours(&req) || req.request != scsi::REQUEST_GET_MAX_LUN {
            return None;
        }
        buf[0] = 0; // one logical unit
        Some(InResponse::Accepted(&buf[..1]))
    }
}

/// Starts the 48 MHz USB PLL and clock and takes the controller out of
/// reset. `main` stops them at boot, and [`stop_usb`] stops them again.
fn start_usb_clocks() {
    let resets = pac::RESETS;
    resets.reset().modify(|w| w.set_pll_usb(true));
    resets.reset().modify(|w| w.set_pll_usb(false));
    while !resets.reset_done().read().pll_usb() {}

    // 12 MHz crystal * 120 = 1440 MHz VCO, / 6 / 5 = 48 MHz.
    let pll = pac::PLL_USB;
    pll.pwr().write(|w| {
        w.set_pd(true);
        w.set_vcopd(true);
        w.set_postdivpd(true);
        w.set_dsmpd(true);
    });
    pll.cs().write(|w| w.set_refdiv(1));
    pll.fbdiv_int().write(|w| w.set_fbdiv_int(120));
    pll.pwr().write(|w| {
        w.set_pd(false);
        w.set_vcopd(false);
        w.set_postdivpd(true);
        w.set_dsmpd(true);
    });
    let mut timeout = 1_000_000u32;
    while !pll.cs().read().lock() {
        timeout -= 1;
        if timeout == 0 {
            error!("usb drive: USB PLL did not lock");
            break;
        }
    }
    pll.prim().write(|w| {
        w.set_postdiv1(6);
        w.set_postdiv2(5);
    });
    pll.pwr().modify(|w| w.set_postdivpd(false));

    let clocks = pac::CLOCKS;
    clocks.clk_usb_div().write(|w| w.set_int(1));
    clocks.clk_usb_ctrl().write(|w| {
        w.set_auxsrc(pac::clocks::vals::ClkUsbCtrlAuxsrc::CLKSRC_PLL_USB);
        w.set_enable(true);
    });

    resets.reset().modify(|w| w.set_usbctrl(false));
    while !resets.reset_done().read().usbctrl() {}
}

/// Disconnects from the computer and powers USB back down.
fn stop_usb() {
    embassy_rp::interrupt::USBCTRL_IRQ.disable();
    // Holding the controller in reset also drops the D+ pull-up, which the
    // computer sees as unplugging.
    pac::RESETS.reset().modify(|w| w.set_usbctrl(true));
    pac::CLOCKS.clk_usb_ctrl().modify(|w| w.set_enable(false));
    pac::PLL_USB.pwr().modify(|w| {
        w.set_pd(true);
        w.set_dsmpd(true);
        w.set_postdivpd(true);
        w.set_vcopd(true);
    });
    pac::RESETS.reset().modify(|w| w.set_pll_usb(true));
}
