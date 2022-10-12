use std::io::{IoSlice, IoSliceMut};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::{sys, IntoSettings, Settings};

/// A serial port.
pub struct SerialPort {
	inner: sys::SerialPort,
}

impl SerialPort {
	/// Open and configure a serial port by path or name.
	///
	/// On Unix systems, the `name` parameter must be a path to a TTY device.
	/// On Windows, it must be the name of a COM device, such as COM1, COM2, etc.
	///
	/// The second argument is used to configure the serial port.
	/// For simple cases, you pass a `u32` for the baud rate.
	/// See [`IntoSettings`] for more information.
	///
	/// # Example
	/// ```no_run
	/// # use serial2::SerialPort;
	/// # fn main() -> std::io::Result<()> {
	/// SerialPort::open("/dev/ttyUSB0", 115200)?;
	/// #   Ok(())
	/// # }
	/// ```
	pub fn open(name: impl AsRef<Path>, settings: impl IntoSettings) -> std::io::Result<Self> {
		let mut serial_port = Self {
			inner: sys::SerialPort::open(name.as_ref())?,
		};
		let mut port_settings = serial_port.get_configuration()?;
		settings.apply_to_settings(&mut port_settings)?;
		serial_port.set_configuration(&port_settings)?;
		Ok(serial_port)
	}

	/// Get a list of available serial ports.
	///
	/// Not currently supported on all platforms.
	/// On unsupported platforms, this function always returns an error.
	pub fn available_ports() -> std::io::Result<Vec<PathBuf>> {
		sys::enumerate()
	}

	/// Configure (or reconfigure) the serial port.
	pub fn set_configuration(&mut self, settings: &Settings) -> std::io::Result<()> {
		self.inner.set_configuration(&settings.inner)
	}

	/// Get the current configuration of the serial port.
	///
	/// This function can fail if the underlying syscall fails,
	/// or if the serial port configuration can't be reported using [`Settings`].
	pub fn get_configuration(&self) -> std::io::Result<Settings> {
		Ok(Settings {
			inner: self.inner.get_configuration()?,
		})
	}

	/// Read bytes from the serial port.
	///
	/// This is identical to [`std::io::Read::read()`], except that this function takes a const reference `&self`.
	/// This allows you to use the serial port concurrently from multiple threads.
	///
	/// Note that there are no guarantees on which thread receives what data when multiple threads are reading from the serial port.
	/// You should normally limit yourself to a single reading thread and a single writing thread.
	pub fn read(&self, buf: &mut [u8]) -> std::io::Result<usize> {
		self.inner.read(buf)
	}

	/// Read bytes from the serial port into a slice of buffers.
	///
	/// This is identical to [`std::io::Read::read_vectored()`], except that this function takes a const reference `&self`.
	/// This allows you to use the serial port concurrently from multiple threads.
	///
	/// Note that there are no guarantees on which thread receives what data when multiple threads are reading from the serial port.
	/// You should normally limit yourself to a single reading thread and a single writing thread.
	pub fn read_vectored(&self, buf: &mut [IoSliceMut<'_>]) -> std::io::Result<usize> {
		self.inner.read_vectored(buf)
	}

	/// Check if the implementation supports vectored reads.
	///
	/// If this returns false, then [`Self::read_vectored()`] will only use the first buffer of the given slice.
	/// All platforms except for Windows support vectored reads.
	pub fn is_read_vectored(&self) -> bool {
		self.inner.is_read_vectored()
	}

	/// Write bytes to the serial port.
	///
	/// This is identical to [`std::io::Write::write()`], except that this function takes a const reference `&self`.
	/// This allows you to use the serial port concurrently from multiple threads.
	///
	/// Note that data written to the same serial port from multiple threads may end up interleaved at the receiving side.
	/// You should normally limit yourself to a single reading thread and a single writing thread.
	pub fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
		self.inner.write(buf)
	}

	/// Write all bytes to the serial port.
	///
	/// This will continue to call [`Self::write()`] until the entire buffer has been written,
	/// or an I/O error occurs.
	///
	/// This is identical to [`std::io::Write::write_all()`], except that this function takes a const reference `&self`.
	/// This allows you to use the serial port concurrently from multiple threads.
	///
	/// Note that data written to the same serial port from multiple threads may end up interleaved at the receiving side.
	/// You should normally limit yourself to a single reading thread and a single writing thread.
	pub fn write_all(&self, buf: &[u8]) -> std::io::Result<()> {
		let mut written = 0;
		while written < buf.len() {
			written += self.write(&buf[written..])?;
		}
		Ok(())
	}

	/// Write bytes to the serial port from a slice of buffers.
	///
	/// This is identical to [`std::io::Write::write_vectored()`], except that this function takes a const reference `&self`.
	/// This allows you to use the serial port concurrently from multiple threads.
	///
	/// Note that data written to the same serial port from multiple threads may end up interleaved at the receiving side.
	/// You should normally limit yourself to a single reading thread and a single writing thread.
	pub fn write_vectored(&self, buf: &[IoSlice<'_>]) -> std::io::Result<usize> {
		self.inner.write_vectored(buf)
	}

	/// Check if the implementation supports vectored writes.
	///
	/// If this returns false, then [`Self::write_vectored()`] will only use the first buffer of the given slice.
	/// All platforms except for Windows support vectored writes.
	pub fn is_write_vectored(&self) -> bool {
		self.inner.is_write_vectored()
	}

	/// Flush all data queued to be written.
	///
	/// This will block until the OS buffer has been fully transmitted.
	///
	/// This is identical to [`std::io::Write::flush()`], except that this function takes a const reference `&self`.
	pub fn flush(&self) -> std::io::Result<()> {
		self.inner.flush_output()
	}

	/// Set the read timeout for the serial port.
	///
	/// The timeout set by this function is an upper bound on individual calls to [`std::io::Read::read()`].
	/// Other platform specific time-outs may trigger before this timeout does.
	pub fn set_read_timeout(&mut self, timeout: Duration) -> std::io::Result<()> {
		self.inner.set_read_timeout(timeout)
	}

	/// Get the read timeout of the serial port.
	pub fn get_read_timeout(&self) -> std::io::Result<Duration> {
		self.inner.get_read_timeout()
	}

	/// Set the write timeout for the serial port.
	///
	/// The timeout set by this function is an upper bound on individual calls to [`std::io::Write::write()`].
	/// Other platform specific time-outs may trigger before this timeout does.
	pub fn set_write_timeout(&mut self, timeout: Duration) -> std::io::Result<()> {
		self.inner.set_write_timeout(timeout)
	}

	/// Get the write timeout of the serial port.
	pub fn get_write_timeout(&self) -> std::io::Result<Duration> {
		self.inner.get_write_timeout()
	}

	/// Discard the kernel input and output buffers for the serial port.
	///
	/// When you write to a serial port, the data may be put in a buffer by the OS to be transmitted by the actual device later.
	/// Similarly, data received on the device can be put in a buffer by the OS untill you read it.
	/// This function clears both buffers: any untransmitted data and received but unread data is discarded by the OS.
	pub fn discard_buffers(&self) -> std::io::Result<()> {
		self.inner.discard_buffers(true, true)
	}

	/// Discard the kernel input buffers for the serial port.
	///
	/// Data received on the device can be put in a buffer by the OS untill you read it.
	/// This function clears that buffer: received but unread data is discarded by the OS.
	///
	/// This is particularly useful when communicating with a device that only responds to commands that you send to it.
	/// If you discard the input buffer before sending the command, you discard any noise that may have been received after the last command.
	pub fn discard_input_buffer(&self) -> std::io::Result<()> {
		self.inner.discard_buffers(true, false)
	}

	/// Discard the kernel output buffers for the serial port.
	///
	/// When you write to a serial port, the data is generally put in a buffer by the OS to be transmitted by the actual device later.
	/// This function clears that buffer: any untransmitted data is discarded by the OS.
	pub fn discard_output_buffer(&self) -> std::io::Result<()> {
		self.inner.discard_buffers(false, true)
	}

	/// Set the state of the Ready To Send line.
	///
	/// If hardware flow control is enabled on the serial port, it is platform specific what will happen.
	/// The function may fail with an error or it may silently be ignored.
	/// It may even succeed and interfere with the flow control.
	pub fn set_rts(&self, state: bool) -> std::io::Result<()> {
		self.inner.set_rts(state)
	}

	/// Read the state of the Clear To Send line.
	///
	/// If hardware flow control is enabled on the serial port, it is platform specific what will happen.
	/// The function may fail with an error, it may return a bogus value, or it may return the actual state of the CTS line.
	pub fn read_cts(&self) -> std::io::Result<bool> {
		self.inner.read_cts()
	}

	/// Set the state of the Data Terminal Ready line.
	///
	/// If hardware flow control is enabled on the serial port, it is platform specific what will happen.
	/// The function may fail with an error or it may silently be ignored.
	pub fn set_dtr(&self, state: bool) -> std::io::Result<()> {
		self.inner.set_dtr(state)
	}

	/// Read the state of the Data Set Ready line.
	///
	/// If hardware flow control is enabled on the serial port, it is platform specific what will happen.
	/// The function may fail with an error, it may return a bogus value, or it may return the actual state of the DSR line.
	pub fn read_dsr(&self) -> std::io::Result<bool> {
		self.inner.read_dsr()
	}

	/// Read the state of the Ring Indicator line.
	///
	/// This line is also sometimes also called the RNG or RING line.
	pub fn read_ri(&self) -> std::io::Result<bool> {
		self.inner.read_ri()
	}

	/// Read the state of the Carrier Detect (CD) line.
	///
	/// This line is also called the Data Carrier Detect (DCD) line
	/// or the Receive Line Signal Detect (RLSD) line.
	pub fn read_cd(&self) -> std::io::Result<bool> {
		self.inner.read_cd()
	}
}

impl std::io::Read for SerialPort {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		self.inner.read(buf)
	}

	fn read_vectored(&mut self, buf: &mut [IoSliceMut<'_>]) -> std::io::Result<usize> {
		self.inner.read_vectored(buf)
	}
}

impl std::io::Write for SerialPort {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		self.inner.write(buf)
	}

	fn write_vectored(&mut self, buf: &[IoSlice<'_>]) -> std::io::Result<usize> {
		self.inner.write_vectored(buf)
	}

	fn flush(&mut self) -> std::io::Result<()> {
		self.inner.flush_output()
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
		use std::fs::File;
		Self {
			inner: sys::SerialPort::from_file(File::from_raw_fd(fd)),
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
		use std::fs::File;
		Self {
			inner: sys::SerialPort::from_file(File::from_raw_handle(handle)),
		}
	}
}
