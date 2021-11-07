//! Serial communication for Rust.
//!
//! The `serial2` crate provides a cross-platform way to use serial ports.
//! The API is inspired by the `serial` and `serialport` crates, and in some cases even borrows implementation details.
//!
//! This crate adds some missing functionality compared to the `serial` crate, and aims to have a simpler API than other alternatives.
//! This mostly means that there is a single [`SerialPort`] type rather than a trait and platform specific implementations.
//! Platform specific functionality is simply implemented on the same type, and removed on incompatible platforms with `#[cfg(...)]` attributes.
//!
//! You can open and configure a serial port in one go with [`SerialPort::open()`].
//! The returned [`SerialPort`] object implements the standard [`std::io::Read`] and [`std::io::Write`] traits,
//! as well as some serial port specific functions.
//!
//! It is also possible to clear the OS buffers for the serial port.
//! The kernel input buffer contains data that has been received by the kernel, but has not yet been returned by a `read()` call.
//! The kernel output buffer contains data that has been passed to the kernel with a `write()` call, but has not yet been transmitted by the hardware.
//! You can clear these buffers with one of the [`SerialPort::discard_input_buffer()`], [`SerialPort::discard_output_buffer()`] or [`SerialPort::discard_buffers()`] functions.
//!
//! The crate also supports read/write timeouts, which can be set using [`SerialPort::set_read_timeout`] and [`SerialPort::set_write_timeout`].
//! The exact timeout behaviour is platform specific, so be sure to read the documentation for more details.
//!
//! Finally, the library allows you to control or read the state of some individual signal lines using
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
	COMMON_BAUD_RATES,
	CharSize,
	StopBits,
	Parity,
	FlowControl,
};
