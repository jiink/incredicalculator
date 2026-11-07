//! The device side of USB mass storage (Bulk-Only Transport) for a single
//! disk: parsing command blocks, answering the SCSI commands PCs use, and
//! building status replies. Moving the data over USB is up to the caller:
//!
//! 1. Read a 31-byte CBW from the bulk OUT endpoint; [`Cbw::parse`] it.
//! 2. Pass its command to [`Scsi::command`], which says what data to move.
//! 3. Move it (see [`Transfer`]), then send [`Cbw::status`] on bulk IN.

use crate::SECTOR_SIZE;

pub const CBW_LEN: usize = 31;
pub const CSW_LEN: usize = 13;
/// Mass storage interface class, SCSI transparent subclass, Bulk-Only protocol.
pub const CLASS_MASS_STORAGE: u8 = 0x08;
pub const SUBCLASS_SCSI: u8 = 0x06;
pub const PROTOCOL_BULK_ONLY: u8 = 0x50;
/// Class requests on the interface's control endpoint.
pub const REQUEST_GET_MAX_LUN: u8 = 0xFE;
pub const REQUEST_RESET: u8 = 0xFF;
/// Room for the longest reply [`Scsi::command`] writes.
pub const REPLY_BUF_LEN: usize = 36;

const CBW_SIGNATURE: u32 = 0x4342_5355;
const CSW_SIGNATURE: u32 = 0x5342_5355;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CswStatus {
    Passed = 0,
    Failed = 1,
    /// The host and device disagree about the data phase. The host answers
    /// with a reset.
    PhaseError = 2,
}

/// Command Block Wrapper: one command from the host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cbw {
    pub tag: u32,
    /// Bytes the host expects to move in the data phase.
    pub data_len: u32,
    /// Direction of that data: to the host (IN) or from it.
    pub data_in: bool,
    pub command: [u8; 16],
}

impl Cbw {
    /// `None` if `bytes` is not a valid CBW, after which the device should
    /// wait for the host to reset it.
    pub fn parse(bytes: &[u8]) -> Option<Cbw> {
        if bytes.len() != CBW_LEN {
            return None;
        }
        let u32_at = |i: usize| u32::from_le_bytes(bytes[i..i + 4].try_into().unwrap());
        let command_len = bytes[14] as usize;
        if u32_at(0) != CBW_SIGNATURE || !(1..=16).contains(&command_len) {
            return None;
        }
        let mut command = [0; 16];
        command[..command_len].copy_from_slice(&bytes[15..15 + command_len]);
        Some(Cbw {
            tag: u32_at(4),
            data_len: u32_at(8),
            data_in: bytes[12] & 0x80 != 0,
            command,
        })
    }

    /// Command Status Wrapper for this command. `residue` is how many of the
    /// host's expected bytes were not moved.
    pub fn status(&self, residue: u32, status: CswStatus) -> [u8; CSW_LEN] {
        let mut csw = [0; CSW_LEN];
        csw[0..4].copy_from_slice(&CSW_SIGNATURE.to_le_bytes());
        csw[4..8].copy_from_slice(&self.tag.to_le_bytes());
        csw[8..12].copy_from_slice(&residue.to_le_bytes());
        csw[12] = status as u8;
        csw
    }

    /// How to run the data phase for `reply`, given what the host expects.
    pub fn transfer(&self, reply: Result<Reply, ()>) -> Transfer {
        let expected = self.data_len;
        let mismatch = Transfer {
            data: Data::None,
            pad_in: self.data_in,
            discard_out: if self.data_in { 0 } else { expected },
            residue: expected,
            status: CswStatus::PhaseError,
        };
        let (data, len, data_in) = match reply {
            Err(()) => {
                return Transfer {
                    status: CswStatus::Failed,
                    ..mismatch
                };
            }
            Ok(Reply::None) => (Data::None, 0, self.data_in),
            Ok(Reply::Data { len }) => {
                // Hosts may ask for less than the whole reply.
                let len = (len as u32).min(expected);
                (Data::Reply { len: len as usize }, len, true)
            }
            Ok(Reply::Read { lba, blocks }) => (
                Data::Read { lba, blocks },
                blocks * SECTOR_SIZE as u32,
                true,
            ),
            Ok(Reply::Write { lba, blocks }) => (
                Data::Write { lba, blocks },
                blocks * SECTOR_SIZE as u32,
                false,
            ),
        };
        if len > expected || (len > 0 && data_in != self.data_in) {
            return mismatch;
        }
        Transfer {
            data,
            // A short IN transfer has to end with a short packet; the caller
            // sends a zero-length one if the data ended on a packet boundary.
            pad_in: self.data_in && len < expected,
            discard_out: if self.data_in { 0 } else { expected - len },
            residue: expected - len,
            status: CswStatus::Passed,
        }
    }
}

/// What a command needs moved, as returned by [`Scsi::command`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reply {
    /// No data.
    None,
    /// Send the first `len` bytes of the reply buffer to the host.
    Data { len: usize },
    /// Send these sectors to the host.
    Read { lba: u32, blocks: u32 },
    /// Receive these sectors from the host and write them.
    Write { lba: u32, blocks: u32 },
}

/// The data phase of one command, from [`Cbw::transfer`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Transfer {
    pub data: Data,
    /// After `data`, end the IN transfer early with a zero-length packet if
    /// the data sent was a whole number of packets.
    pub pad_in: bool,
    /// After `data`, read and drop this many bytes the host sends.
    pub discard_out: u32,
    pub residue: u32,
    pub status: CswStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Data {
    None,
    Reply { len: usize },
    Read { lba: u32, blocks: u32 },
    Write { lba: u32, blocks: u32 },
}

/// SCSI sense data: why the last command failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Sense {
    key: u8,
    code: u8,
    qualifier: u8,
}

const NO_SENSE: Sense = Sense {
    key: 0,
    code: 0,
    qualifier: 0,
};
const MEDIUM_NOT_PRESENT: Sense = Sense {
    key: 0x02,
    code: 0x3A,
    qualifier: 0,
};
const INVALID_COMMAND: Sense = Sense {
    key: 0x05,
    code: 0x20,
    qualifier: 0,
};
const LBA_OUT_OF_RANGE: Sense = Sense {
    key: 0x05,
    code: 0x21,
    qualifier: 0,
};
const INVALID_FIELD: Sense = Sense {
    key: 0x05,
    code: 0x24,
    qualifier: 0,
};
const MEDIUM_ERROR: u8 = 0x03;

/// One disk's SCSI state.
pub struct Scsi {
    sector_count: u32,
    sense: Sense,
    /// The host ejected the disk (START STOP UNIT). Writes are flushed and
    /// it won't touch the disk again until it is reconnected.
    pub ejected: bool,
    /// The host asked that the disk not be removed (PREVENT ALLOW MEDIUM
    /// REMOVAL); it does this while it has the drive mounted.
    pub removal_prevented: bool,
    /// The host asked for its writes to be made durable (SYNCHRONIZE CACHE
    /// or an eject). The caller should flush and clear this.
    pub flush_requested: bool,
}

impl Scsi {
    pub fn new(sector_count: u32) -> Self {
        Self {
            sector_count,
            sense: NO_SENSE,
            ejected: false,
            removal_prevented: false,
            flush_requested: false,
        }
    }

    /// Call when a sector read or write fails during a [`Reply::Read`] or
    /// [`Reply::Write`], so the host learns why its command failed.
    pub fn set_medium_error(&mut self) {
        self.sense = Sense {
            key: MEDIUM_ERROR,
            code: 0x11, // unrecovered read error
            qualifier: 0,
        };
    }

    /// Handles a SCSI command block. Replies with data are written into
    /// `buf`. `Err` means the command failed; the host then asks why with
    /// REQUEST SENSE.
    pub fn command(&mut self, cb: &[u8; 16], buf: &mut [u8; REPLY_BUF_LEN]) -> Result<Reply, ()> {
        let result = self.run(cb, buf);
        if let Err(sense) = result {
            self.sense = sense;
        } else if cb[0] != REQUEST_SENSE {
            self.sense = NO_SENSE;
        }
        result.map_err(|_| ())
    }

    fn run(&mut self, cb: &[u8; 16], buf: &mut [u8; REPLY_BUF_LEN]) -> Result<Reply, Sense> {
        let be16 = |i: usize| u16::from_be_bytes([cb[i], cb[i + 1]]) as u32;
        let be32 = |i: usize| u32::from_be_bytes([cb[i], cb[i + 1], cb[i + 2], cb[i + 3]]);
        let medium = |this: &Self| {
            if this.ejected {
                Err(MEDIUM_NOT_PRESENT)
            } else {
                Ok(())
            }
        };
        buf.fill(0);
        let len = match cb[0] {
            TEST_UNIT_READY => {
                medium(self)?;
                0
            }
            REQUEST_SENSE => {
                buf[0] = 0x70; // current error, fixed format
                buf[2] = self.sense.key;
                buf[7] = 10; // additional length
                buf[12] = self.sense.code;
                buf[13] = self.sense.qualifier;
                self.sense = NO_SENSE;
                (cb[4] as usize).min(18)
            }
            INQUIRY => {
                if cb[1] & 0x01 != 0 {
                    // Vital product data pages aren't supported.
                    return Err(INVALID_FIELD);
                }
                buf[0] = 0x00; // direct access block device
                buf[1] = 0x80; // removable
                buf[2] = 0x04; // SPC-2
                buf[3] = 0x02; // response data format
                buf[4] = 31; // additional length
                buf[8..16].copy_from_slice(b"Incredi ");
                buf[16..32].copy_from_slice(b"Calculator      ");
                buf[32..36].copy_from_slice(b"1.0 ");
                (cb[4] as usize).min(36)
            }
            MODE_SENSE_6 => {
                // Header only: no pages, not write protected.
                buf[0] = 3;
                (cb[4] as usize).min(4)
            }
            MODE_SENSE_10 => {
                buf[1] = 6;
                (be16(7) as usize).min(8)
            }
            START_STOP_UNIT => {
                let load_eject = cb[4] & 0x02 != 0;
                let start = cb[4] & 0x01 != 0;
                if load_eject {
                    self.ejected = !start;
                }
                self.flush_requested = true;
                0
            }
            PREVENT_ALLOW_MEDIUM_REMOVAL => {
                self.removal_prevented = cb[4] & 0x01 != 0;
                0
            }
            READ_FORMAT_CAPACITIES => {
                buf[3] = 8; // capacity list length
                buf[4..8].copy_from_slice(&self.sector_count.to_be_bytes());
                buf[8] = 0x02; // formatted media
                buf[9..12].copy_from_slice(&(SECTOR_SIZE as u32).to_be_bytes()[1..]);
                (be16(7) as usize).min(12)
            }
            READ_CAPACITY_10 => {
                medium(self)?;
                buf[0..4].copy_from_slice(&(self.sector_count - 1).to_be_bytes());
                buf[4..8].copy_from_slice(&(SECTOR_SIZE as u32).to_be_bytes());
                8
            }
            READ_10 | WRITE_10 | VERIFY_10 => {
                medium(self)?;
                let lba = be32(2);
                let blocks = be16(7);
                if lba as u64 + blocks as u64 > self.sector_count as u64 {
                    return Err(LBA_OUT_OF_RANGE);
                }
                return Ok(match cb[0] {
                    READ_10 => Reply::Read { lba, blocks },
                    WRITE_10 => Reply::Write { lba, blocks },
                    _ => Reply::None,
                });
            }
            SYNCHRONIZE_CACHE_10 => {
                self.flush_requested = true;
                0
            }
            _ => return Err(INVALID_COMMAND),
        };
        Ok(if len == 0 && !matches!(cb[0], REQUEST_SENSE | INQUIRY) {
            Reply::None
        } else {
            Reply::Data { len }
        })
    }
}

const TEST_UNIT_READY: u8 = 0x00;
const REQUEST_SENSE: u8 = 0x03;
const INQUIRY: u8 = 0x12;
const MODE_SENSE_6: u8 = 0x1A;
const START_STOP_UNIT: u8 = 0x1B;
const PREVENT_ALLOW_MEDIUM_REMOVAL: u8 = 0x1E;
const READ_FORMAT_CAPACITIES: u8 = 0x23;
const READ_CAPACITY_10: u8 = 0x25;
const READ_10: u8 = 0x28;
const WRITE_10: u8 = 0x2A;
const VERIFY_10: u8 = 0x2F;
const SYNCHRONIZE_CACHE_10: u8 = 0x35;
const MODE_SENSE_10: u8 = 0x5A;

#[cfg(test)]
mod tests {
    use super::*;

    fn cbw_bytes(tag: u32, data_len: u32, data_in: bool, command: &[u8]) -> [u8; CBW_LEN] {
        let mut b = [0; CBW_LEN];
        b[0..4].copy_from_slice(&CBW_SIGNATURE.to_le_bytes());
        b[4..8].copy_from_slice(&tag.to_le_bytes());
        b[8..12].copy_from_slice(&data_len.to_le_bytes());
        b[12] = if data_in { 0x80 } else { 0 };
        b[14] = command.len() as u8;
        b[15..15 + command.len()].copy_from_slice(command);
        b
    }

    fn cb(bytes: &[u8]) -> [u8; 16] {
        let mut cb = [0; 16];
        cb[..bytes.len()].copy_from_slice(bytes);
        cb
    }

    fn run(scsi: &mut Scsi, bytes: &[u8]) -> (Result<Reply, ()>, [u8; REPLY_BUF_LEN]) {
        let mut buf = [0; REPLY_BUF_LEN];
        let reply = scsi.command(&cb(bytes), &mut buf);
        (reply, buf)
    }

    #[test]
    fn parses_cbws_and_builds_csws() {
        let bytes = cbw_bytes(0xABCD, 512, true, &[READ_10, 0, 0, 0, 0, 1, 0, 0, 1, 0]);
        let cbw = Cbw::parse(&bytes).unwrap();
        assert_eq!(cbw.tag, 0xABCD);
        assert_eq!(cbw.data_len, 512);
        assert!(cbw.data_in);
        assert_eq!(cbw.command[0], READ_10);
        let csw = cbw.status(12, CswStatus::Failed);
        assert_eq!(&csw[0..4], b"USBS");
        assert_eq!(&csw[4..8], &0xABCDu32.to_le_bytes());
        assert_eq!(&csw[8..12], &12u32.to_le_bytes());
        assert_eq!(csw[12], 1);

        assert_eq!(Cbw::parse(&bytes[..30]), None);
        let mut bad = bytes;
        bad[0] = 0;
        assert_eq!(Cbw::parse(&bad), None);
    }

    #[test]
    fn reports_capacity_and_identity() {
        let mut scsi = Scsi::new(1024);
        let (reply, buf) = run(&mut scsi, &[READ_CAPACITY_10]);
        assert_eq!(reply, Ok(Reply::Data { len: 8 }));
        assert_eq!(buf[0..8], [0, 0, 0x03, 0xFF, 0, 0, 0x02, 0]);
        let (reply, buf) = run(&mut scsi, &[INQUIRY, 0, 0, 0, 36, 0]);
        assert_eq!(reply, Ok(Reply::Data { len: 36 }));
        assert_eq!(buf[1], 0x80);
        assert_eq!(&buf[16..26], b"Calculator");
        let (reply, _) = run(&mut scsi, &[INQUIRY, 0, 0, 0, 5, 0]);
        assert_eq!(reply, Ok(Reply::Data { len: 5 }));
    }

    #[test]
    fn reads_and_writes_are_range_checked() {
        let mut scsi = Scsi::new(1024);
        let (reply, _) = run(&mut scsi, &[READ_10, 0, 0, 0, 0x03, 0xFE, 0, 0, 2, 0]);
        assert_eq!(
            reply,
            Ok(Reply::Read {
                lba: 1022,
                blocks: 2
            })
        );
        let (reply, _) = run(&mut scsi, &[WRITE_10, 0, 0, 0, 0x03, 0xFF, 0, 0, 2, 0]);
        assert_eq!(reply, Err(()));
        let (_, sense) = run(&mut scsi, &[REQUEST_SENSE, 0, 0, 0, 18, 0]);
        assert_eq!((sense[2], sense[12]), (0x05, 0x21));
        // Sense is cleared once read.
        let (_, sense) = run(&mut scsi, &[REQUEST_SENSE, 0, 0, 0, 18, 0]);
        assert_eq!((sense[2], sense[12]), (0, 0));
    }

    #[test]
    fn unknown_commands_fail_with_sense() {
        let mut scsi = Scsi::new(1024);
        let (reply, _) = run(&mut scsi, &[0xEE]);
        assert_eq!(reply, Err(()));
        let (_, sense) = run(&mut scsi, &[REQUEST_SENSE, 0, 0, 0, 18, 0]);
        assert_eq!((sense[2], sense[12]), (0x05, 0x20));
    }

    #[test]
    fn eject_flushes_and_removes_the_medium() {
        let mut scsi = Scsi::new(1024);
        let _ = run(&mut scsi, &[PREVENT_ALLOW_MEDIUM_REMOVAL, 0, 0, 0, 1, 0]);
        assert!(scsi.removal_prevented);
        let _ = run(&mut scsi, &[PREVENT_ALLOW_MEDIUM_REMOVAL, 0, 0, 0, 0, 0]);
        assert!(!scsi.removal_prevented);
        let (reply, _) = run(&mut scsi, &[START_STOP_UNIT, 0, 0, 0, 0x02, 0]);
        assert_eq!(reply, Ok(Reply::None));
        assert!(scsi.ejected && scsi.flush_requested);
        assert_eq!(run(&mut scsi, &[TEST_UNIT_READY]).0, Err(()));
        let (_, sense) = run(&mut scsi, &[REQUEST_SENSE, 0, 0, 0, 18, 0]);
        assert_eq!((sense[2], sense[12]), (0x02, 0x3A));
        // Loading it again brings it back.
        let _ = run(&mut scsi, &[START_STOP_UNIT, 0, 0, 0, 0x03, 0]);
        assert_eq!(run(&mut scsi, &[TEST_UNIT_READY]).0, Ok(Reply::None));
    }

    fn cbw(data_len: u32, data_in: bool) -> Cbw {
        Cbw {
            tag: 1,
            data_len,
            data_in,
            command: [0; 16],
        }
    }

    #[test]
    fn transfers_follow_the_hosts_expected_length() {
        // Exact read.
        let t = cbw(1024, true).transfer(Ok(Reply::Read { lba: 0, blocks: 2 }));
        assert_eq!(t.data, Data::Read { lba: 0, blocks: 2 });
        assert_eq!(
            (t.pad_in, t.residue, t.status),
            (false, 0, CswStatus::Passed)
        );
        // Host wants more than the reply: short transfer and residue.
        let t = cbw(255, true).transfer(Ok(Reply::Data { len: 36 }));
        assert_eq!(t.data, Data::Reply { len: 36 });
        assert_eq!(
            (t.pad_in, t.residue, t.status),
            (true, 219, CswStatus::Passed)
        );
        // Host wants less: truncate.
        let t = cbw(5, true).transfer(Ok(Reply::Data { len: 36 }));
        assert_eq!(t.data, Data::Reply { len: 5 });
        assert_eq!(t.residue, 0);
        // Failed command expecting data out: drain it.
        let t = cbw(512, false).transfer(Err(()));
        assert_eq!(
            (t.discard_out, t.residue, t.status),
            (512, 512, CswStatus::Failed)
        );
        // Write when the host means to read: phase error.
        let t = cbw(512, true).transfer(Ok(Reply::Write { lba: 0, blocks: 1 }));
        assert_eq!(t.status, CswStatus::PhaseError);
        assert!(t.pad_in);
        // More data than the host expects: phase error.
        let t = cbw(512, true).transfer(Ok(Reply::Read { lba: 0, blocks: 2 }));
        assert_eq!(t.status, CswStatus::PhaseError);
        // No-data command with no data phase.
        let t = cbw(0, false).transfer(Ok(Reply::None));
        assert_eq!(
            t,
            Transfer {
                data: Data::None,
                pad_in: false,
                discard_out: 0,
                residue: 0,
                status: CswStatus::Passed,
            }
        );
    }
}
