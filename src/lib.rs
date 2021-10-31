//! Serial ports for Rust.
//!
//! The `serial2` crate provides a cross-platform way to use serial ports.
//! The API is heavily inspired by the `serial` crate, which now seems to be unmaintained.
//! This crate adds some missing functionality and has a simplified interface compared to it's predecessor.
//!
//! You can open a serial port and configure it in one go with [`SerialPort::open()`].
//! The returned [`SerialPort`] object implements the standard [`std::io::Read`] and [`std::io::Write`] traits.
//!
//! You can also clear the OS buffer for received but not-yet read data,
//! and the OS buffer for written but not-yet transmitted data.
//! This is done with one of the [`SerialPort::discard_input_buffer()`], [`SerialPort::discard_output_buffer()`] or [`SerialPort::discard_buffers()`] functions.
//!
//! You can set read/write timeouts using [`SerialPort::set_read_timeout`] and [`SerialPort::set_write_timeout`].
//! The exact timeout behaviour is platform specific, so be sure to read the documentation for more details.
//!
//! You can also control or read some individual signal lines using
//! [`SerialPort::set_rts()`], [`SerialPort::read_cts()`], [`SerialPort::set_dtr()`], [`SerialPort::read_dsr()`],
//! [`SerialPort::read_ri()`] and [`SerialPort::read_cd()`].

#![warn(missing_docs)]

mod sys;

mod into_settings;
pub use into_settings::IntoSettings;

mod serial_port;
pub use serial_port::SerialPort;

mod settings;
pub use settings::{
	Settings,
	CharSize,
	StopBits,
	Parity,
	FlowControl,
};
