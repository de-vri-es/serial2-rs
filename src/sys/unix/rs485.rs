/// RS485 support derived from the rs485 crate (https://github.com/mbr/rs485-rs)
/// Updated by Omelia Iliffe 2024, to include the `set_terminate_bus` and `set_mode_rs422` flags

use std::{mem, io};
use std::os::unix::io::{AsRawFd, RawFd};
use bitflags::bitflags;
use super::check;

// bitflags used by rs485 functionality
bitflags! {
    #[derive(Copy, Clone, Debug)]
    pub struct Rs485Flags: u32 {
        const SER_RS485_ENABLED        = (1 << 0);
        const SER_RS485_RTS_ON_SEND    = (1 << 1);
        const SER_RS485_RTS_AFTER_SEND = (1 << 2);

        const SER_RS485_RX_DURING_TX   = (1 << 4);
        const SER_RS485_TERMINATE_BUS  = (1 << 5);
        const SER_RS485_MODE_RS422     = (1 << 9);
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
/// RS485 serial configuration
///
/// Internally, this structure is the same as a [`struct serial_rs485`]
///(https://github.com/torvalds/linux/blob/e8f897f4afef0031fe618a8e94127a0934896aba/include/uapi/linux/serial.h#L143).
pub struct SerialRs485 {
    flags: Rs485Flags,
    delay_rts_before_send: u32,
    delay_rts_after_send: u32,
    _padding: [u32; 5],
}


impl SerialRs485 {
    /// Create a new, empty set of serial settings
    ///
    /// All flags will default to "off", delays will be set to 0 ms.
    #[inline]
    pub fn new() -> SerialRs485 {
        unsafe { mem::zeroed() }
    }

    /// Load settings from file descriptor
    ///
    /// Settings will be loaded from the file descriptor, which must be a
    /// valid serial device support RS485 extensions
    #[inline]
    pub fn from_fd(fd: RawFd) -> io::Result<SerialRs485> {
        let mut conf = SerialRs485::new();

        unsafe { check(libc::ioctl(fd, libc::TIOCGRS485, &mut conf as *mut SerialRs485))? };

        Ok(conf)

    }

    /// Enable RS485 support
    ///
    /// Unless enabled, none of the settings set take effect.
    #[inline]
    pub fn set_enabled(&mut self, enabled: bool) -> &mut Self {
        if enabled {
            self.flags |= Rs485Flags::SER_RS485_ENABLED;
        } else {
            self.flags &= !Rs485Flags::SER_RS485_ENABLED;
        }

        self
    }

    /// Set RTS high or low before sending
    ///
    /// RTS will be set before sending, this setting controls whether
    /// it will be set high (`true`) or low (`false`).
    #[inline]
    pub fn set_rts_on_send(&mut self, rts_on_send: bool) -> &mut Self {
        if rts_on_send {
            self.flags |= Rs485Flags::SER_RS485_RTS_ON_SEND;
        } else {
            self.flags &= !Rs485Flags::SER_RS485_RTS_ON_SEND;
        }

        self
    }

    /// Set RTS high or low after sending
    ///
    /// RTS will be set after sending, this setting contrls whether
    /// it will be set high (`true`) or low (`false`).
    #[inline]
    pub fn set_rts_after_send(&mut self, rts_after_send: bool) -> &mut Self {
        if rts_after_send {
            self.flags |= Rs485Flags::SER_RS485_RTS_AFTER_SEND;
        } else {
            self.flags &= !Rs485Flags::SER_RS485_RTS_AFTER_SEND;
        }

        self
    }

    /// Delay before sending in ms
    ///
    /// If set to non-zero, transmission will not start until
    /// `delays_rts_before_send` milliseconds after RTS has been set
    #[inline]
    pub fn delay_rts_before_send_ms(&mut self, delay_rts_before_send: u32) -> &mut Self {
        self.delay_rts_before_send = delay_rts_before_send;
        self
    }

    /// Hold RTS after sending, in ms
    ///
    /// If set to non-zero, RTS will be kept high/low for
    /// `delays_rts_after_send` ms after the transmission is complete
    #[inline]
    pub fn delay_rts_after_send_ms(&mut self, delay_rts_after_send: u32) -> &mut Self {
        self.delay_rts_after_send = delay_rts_after_send;
        self
    }

    /// Allow receiving whilst transmitting
    ///
    /// Note that turning off this option sometimes seems to make the UART
    /// misbehave and cut off transmission. For this reason, it is best left on
    /// even when using half-duplex.
    pub fn set_rx_during_tx(&mut self, set_rx_during_tx: bool) -> &mut Self {
        if set_rx_during_tx {
            self.flags |= Rs485Flags::SER_RS485_RX_DURING_TX
        } else {
            self.flags &= !Rs485Flags::SER_RS485_RX_DURING_TX;
        }
        self
    }

    pub fn set_terminate_bus(&mut self, terminate_bus: bool) -> &mut Self {
        if terminate_bus {
            self.flags |= Rs485Flags::SER_RS485_TERMINATE_BUS
        } else {
            self.flags &= !Rs485Flags::SER_RS485_TERMINATE_BUS;
        }
        self
    }

    pub fn set_mode_rs422(&mut self, mode_rs422: bool) -> &mut Self {
        if mode_rs422 {
            self.flags |= Rs485Flags::SER_RS485_MODE_RS422
        } else {
            self.flags &= !Rs485Flags::SER_RS485_MODE_RS422;
        }
        self
    }

    /// Apply settings to file descriptor
    ///
    /// Applies the constructed configuration a raw filedescriptor using
    /// `ioctl`.
    #[inline]
    pub fn set_on_fd(&self, fd: RawFd) -> io::Result<()> {
        unsafe { check(libc::ioctl(fd, libc::TIOCSRS485, self as *const SerialRs485))? };
        Ok(())
    }
}

// not sure if we want these traits
//
//
// /// Rs485 controls
// ///
// /// A convenient trait for controlling Rs485 parameters.
// pub trait Rs485 {
//     /// Retrieves RS485 parameters from target
//     fn get_rs485_conf(&self) -> io::Result<SerialRs485>;
//
//     /// Sets RS485 parameters on target
//     fn set_rs485_conf(&self, conf: &SerialRs485) -> io::Result<()>;
//
//     /// Update RS485 configuration
//     ///
//     /// Combines `get_rs485_conf` and `set_rs485_conf` through a closure
//     fn update_rs485_conf<F: FnOnce(&mut SerialRs485) -> ()>(&self, f: F) -> io::Result<()>;
// }
//
// impl<T: AsRawFd> Rs485 for T {
//     #[inline]
//     fn get_rs485_conf(&self) -> io::Result<SerialRs485> {
//         SerialRs485::from_fd(self.as_raw_fd())
//     }
//
//     #[inline]
//     fn set_rs485_conf(&self, conf: &SerialRs485) -> io::Result<()> {
//         conf.set_on_fd(self.as_raw_fd())
//     }
//
//     #[inline]
//     fn update_rs485_conf<F: FnOnce(&mut SerialRs485) -> ()>(&self, f: F) -> io::Result<()> {
//         let mut conf = self.get_rs485_conf()?;
//         f(&mut conf);
//         self.set_rs485_conf(&conf)
//     }
// }

