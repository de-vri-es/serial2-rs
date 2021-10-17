use std::ffi::OsStr;
use std::fs::File;
use std::io::{IoSlice, IoSliceMut};
use std::time::Duration;

mod sys;
mod settings;

pub use settings::*;

pub struct SerialPort {
	inner: sys::Inner,
}

pub use DiscardBuffers::*;
pub enum DiscardBuffers {
	DiscardInput,
	DiscardOutput,
	DiscardBoth,
}

impl SerialPort {
	pub fn open(name: impl AsRef<OsStr>, settings: &SerialSettings) -> std::io::Result<Self> {
		let mut inner = sys::open(name.as_ref())?;
		sys::configure(&mut inner, settings)?;
		Ok(Self { inner })
	}

	pub fn open_unconfigured(name: impl AsRef<OsStr>) -> std::io::Result<Self> {
		let inner = sys::open(name.as_ref())?;
		Ok(Self { inner })
	}

	pub fn configure(&mut self, settings: &SerialSettings) -> std::io::Result<()> {
		sys::configure(&mut self.inner, settings)
	}

	pub fn get_configuration(&self) -> std::io::Result<SerialSettings> {
		sys::get_configuration(&self.inner)
	}

	pub fn set_read_timeout(&mut self, timeout: Duration) -> std::io::Result<()> {
		sys::set_read_timeout(&mut self.inner, timeout)
	}

	pub fn get_read_timeout(&self) -> std::io::Result<Duration> {
		sys::get_read_timeout(&self.inner)
	}

	pub fn set_write_timeout(&mut self, timeout: Duration) -> std::io::Result<()> {
		sys::set_write_timeout(&mut self.inner, timeout)
	}

	pub fn get_write_timeout(&self) -> std::io::Result<Duration> {
		sys::get_write_timeout(&self.inner)
	}

	pub fn discard_buffers(&mut self, buffers: DiscardBuffers) -> std::io::Result<()> {
		sys::discard_buffers(&mut self.inner, buffers)
	}

	pub fn discard_input_buffer(&mut self) -> std::io::Result<()> {
		self.discard_buffers(DiscardInput)
	}

	pub fn discard_output_buffer(&mut self) -> std::io::Result<()> {
		self.discard_buffers(DiscardOutput)
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
