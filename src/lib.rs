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

#![warn(missing_docs)]

use std::ffi::OsStr;
use std::fs::File;
use std::io::{IoSlice, IoSliceMut};
use std::time::Duration;

mod sys;
mod settings;

pub use settings::*;

/// A serial port.
pub struct SerialPort {
	inner: sys::Inner,
}

impl SerialPort {
	/// Open and configure a serial port by path or name.
	///
	/// On Unix systems, you must pass the path to a TTY device.
	/// On Windows, you must pass the name of a COM device, such as COM1, COM2, etc.
	///
	/// This function also configures the serial port.
	/// Use [`Self::open_unconfigured()`] to open the serial port without explicitly configuring it.
	pub fn open(name: impl AsRef<OsStr>, settings: &SerialSettings) -> std::io::Result<Self> {
		let mut inner = sys::open(name.as_ref())?;
		sys::configure(&mut inner, settings)?;
		Ok(Self { inner })
	}

	/// Open a serial port by path or name without explicitly configuring it.
	///
	/// On Unix systems, you must pass the path to a TTY device.
	/// On Windows, you must pass the name of a COM device, such as COM1, COM2, etc.
	///
	/// On most platforms (but not Linux), serial ports revert to a default configuration when they are closed.
	/// For consistent cross-platform behaviour, make sure to configure the serial port before using it.
	/// You can open and configure a serial port in one go with [`Self::open()`].
	pub fn open_unconfigured(name: impl AsRef<OsStr>) -> std::io::Result<Self> {
		let inner = sys::open(name.as_ref())?;
		Ok(Self { inner })
	}

	/// Configure (or reconfigure) the serial port.
	pub fn configure(&mut self, settings: &SerialSettings) -> std::io::Result<()> {
		sys::configure(&mut self.inner, settings)
	}

	/// Get the current configuration of the serial port.
	///
	/// This function can fail if the underlying syscall fails,
	/// or if the serial port configuration can't be reported using [`SerialSettings`].
	pub fn get_configuration(&self) -> std::io::Result<SerialSettings> {
		sys::get_configuration(&self.inner)
	}

	/// Set the read timeout for the serial port.
	///
	/// The timeout set by this function is an upper bound on individual calls to [`std::io::Read::read()`].
	/// Other platform specific time-outs may trigger before this timeout does.
	pub fn set_read_timeout(&mut self, timeout: Duration) -> std::io::Result<()> {
		sys::set_read_timeout(&mut self.inner, timeout)
	}

	/// Get the read timeout of the serial port.
	pub fn get_read_timeout(&self) -> std::io::Result<Duration> {
		sys::get_read_timeout(&self.inner)
	}

	/// Set the write timeout for the serial port.
	///
	/// The timeout set by this function is an upper bound on individual calls to [`std::io::Write::write()`].
	/// Other platform specific time-outs may trigger before this timeout does.
	pub fn set_write_timeout(&mut self, timeout: Duration) -> std::io::Result<()> {
		sys::set_write_timeout(&mut self.inner, timeout)
	}

	/// Get the write timeout of the serial port.
	pub fn get_write_timeout(&self) -> std::io::Result<Duration> {
		sys::get_write_timeout(&self.inner)
	}

	/// Discard the kernel input and output buffers for the serial port.
	///
	/// When you write to a serial port, the data may be put in a buffer by the OS to be transmitted by the actual device later.
	/// Similarly, data received on the device can be put in a buffer by the OS untill you read it.
	/// This function clears both buffers: any untransmitted data and received but unread data is discarded by the OS.
	pub fn discard_buffers(&mut self) -> std::io::Result<()> {
		sys::discard_buffers(&mut self.inner, true, true)
	}

	/// Discard the kernel input buffers for the serial port.
	///
	/// Data received on the device can be put in a buffer by the OS untill you read it.
	/// This function clears that buffer: received but unread data is discarded by the OS.
	///
	/// This is particularly useful when communicating with a device that only responds to commands that you send to it.
	/// If you discard the input buffer before sending the command, you discard any noise that may have been received after the last command.
	pub fn discard_input_buffer(&mut self) -> std::io::Result<()> {
		sys::discard_buffers(&mut self.inner, true, false)
	}

	/// Discard the kernel output buffers for the serial port.
	///
	/// When you write to a serial port, the data is generally put in a buffer by the OS to be transmitted by the actual device later.
	/// This function clears that buffer: any untransmitted data is discarded by the OS.
	pub fn discard_output_buffer(&mut self) -> std::io::Result<()> {
		sys::discard_buffers(&mut self.inner, false, true)
	}
}

impl std::io::Read for SerialPort {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		sys::read(&mut self.inner, buf)
	}

	fn read_vectored(&mut self, buf: &mut [IoSliceMut<'_>]) -> std::io::Result<usize> {
		sys::read_vectored(&mut self.inner, buf)
	}
}

impl std::io::Write for SerialPort {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		sys::write(&mut self.inner, buf)
	}

	fn write_vectored(&mut self, buf: &[IoSlice<'_>]) -> std::io::Result<usize> {
		sys::write_vectored(&mut self.inner, buf)
	}

	fn flush(&mut self) -> std::io::Result<()> {
		sys::flush_output(&self.inner)
	}
}

#[cfg(unix)]
impl std::os::unix::io::AsRawFd for SerialPort {
	fn as_raw_fd(&self) -> std::os::unix::io::RawFd {
		self.inner.file.as_raw_fd()
	}
}

#[cfg(unix)]
impl std::os::unix::io::IntoRawFd for SerialPort {
	fn into_raw_fd(self) -> std::os::unix::io::RawFd {
		self.inner.file.into_raw_fd()
	}
}

#[cfg(unix)]
impl std::os::unix::io::FromRawFd for SerialPort {
	unsafe fn from_raw_fd(fd: std::os::unix::io::RawFd) -> Self {
		Self {
			inner: sys::from_file(File::from_raw_fd(fd)),
		}
	}
}

#[cfg(windows)]
impl std::os::windows::io::AsRawHandle for SerialPort {
	fn as_raw_handle(&self) -> std::os::windows::io::RawHandle {
		self.inner.file.as_raw_handle()
	}
}

#[cfg(windows)]
impl std::os::windows::io::IntoRawHandle for SerialPort {
	fn into_raw_handle(self) -> std::os::windows::io::RawHandle {
		self.inner.file.into_raw_handle()
	}
}

#[cfg(windows)]
impl std::os::windows::io::FromRawHandle for SerialPort {
	unsafe fn from_raw_handle(handle: std::os::windows::io::RawHandle) -> Self {
		Self {
			inner: sys::from_file(File::from_raw_handle(handle)),
		}
	}
}
