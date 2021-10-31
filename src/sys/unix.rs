use std::ffi::OsStr;
use std::io::{IoSlice, IoSliceMut};
use std::os::raw::c_int;
use std::os::unix::io::AsRawFd;
use std::time::Duration;

pub struct SerialPort {
	pub file: std::fs::File,
	pub read_timeout_ms: u32,
	pub write_timeout_ms: u32,
}

#[derive(Clone)]
pub struct Settings {
	pub termios: libc::termios,
}

impl SerialPort {
	pub fn open(path: &OsStr) -> std::io::Result<Self> {
		let file = std::fs::OpenOptions::new()
			.read(true)
			.write(true)
			.create(false)
			.open(path)?;

		Ok(Self::from_file(file))
	}

	pub fn from_file(file: std::fs::File) -> Self {
		Self {
			file,
			read_timeout_ms: 100,
			write_timeout_ms: 100,
		}
	}

	pub fn get_configuration(&self) -> std::io::Result<Settings> {
		unsafe {
			let mut termios = std::mem::zeroed();
			check(libc::tcgetattr(self.file.as_raw_fd(), &mut termios))?;
			Ok(Settings { termios })
		}
	}

	pub fn set_configuration(&mut self, settings: &Settings) -> std::io::Result<()> {
		unsafe {
			check(libc::tcsetattr(self.file.as_raw_fd(), libc::TCSADRAIN, &settings.termios))?;
			let applied_settings = self.get_configuration()?;
			if applied_settings != *settings {
				Err(other_error("failed to apply some or all settings"))
			} else {
				Ok(())
			}
		}
	}

	pub fn set_read_timeout(&mut self, timeout: Duration) -> std::io::Result<()> {
		use std::convert::TryInto;
		self.read_timeout_ms = timeout.as_millis().try_into().unwrap_or(u32::MAX);
		Ok(())
	}

	pub fn get_read_timeout(&self) -> std::io::Result<Duration> {
		Ok(Duration::from_millis(self.read_timeout_ms.into()))
	}

	pub fn set_write_timeout(&mut self, timeout: Duration) -> std::io::Result<()> {
		use std::convert::TryInto;
		self.write_timeout_ms = timeout.as_millis().try_into().unwrap_or(u32::MAX);
		Ok(())
	}

	pub fn get_write_timeout(&self) -> std::io::Result<Duration> {
		Ok(Duration::from_millis(self.write_timeout_ms.into()))
	}

	pub fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		use std::io::Read;
		if !poll(&mut self.file, libc::POLLIN, self.read_timeout_ms)? {
			Err(std::io::ErrorKind::TimedOut.into())
		} else {
			self.file.read(buf)
		}
	}

	pub fn read_vectored(&mut self, buf: &mut [IoSliceMut<'_>]) -> std::io::Result<usize> {
		use std::io::Read;
		if !poll(&mut self.file, libc::POLLIN, self.read_timeout_ms)? {
			Err(std::io::ErrorKind::TimedOut.into())
		} else {
			self.file.read_vectored(buf)
		}
	}

	pub fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		use std::io::Write;
		if !poll(&mut self.file, libc::POLLOUT, self.read_timeout_ms)? {
			Err(std::io::ErrorKind::TimedOut.into())
		} else {
			self.file.write(buf)
		}
	}

	pub fn write_vectored(&mut self, buf: &[IoSlice<'_>]) -> std::io::Result<usize> {
		use std::io::Write;
		if !poll(&mut self.file, libc::POLLOUT, self.read_timeout_ms)? {
			Err(std::io::ErrorKind::TimedOut.into())
		} else {
			self.file.write_vectored(buf)
		}
	}

	pub fn flush_output(&self) -> std::io::Result<()> {
		unsafe {
			check(libc::tcdrain(self.file.as_raw_fd()))?;
			Ok(())
		}
	}

	pub fn discard_buffers(&mut self, discard_input: bool, discard_output: bool) -> std::io::Result<()> {
		unsafe {
			let mut flags = 0;
			if discard_input {
				flags |= libc::TCIFLUSH;
			}
			if discard_output {
				flags |= libc::TCOFLUSH;
			}
			check(libc::tcflush(self.file.as_raw_fd(), flags))?;
			Ok(())
		}
	}

	pub fn set_rts(&mut self, state: bool) -> std::io::Result<()> {
		set_pin(&mut self.file, libc::TIOCM_RTS, state)
	}

	pub fn read_cts(&mut self) -> std::io::Result<bool> {
		read_pin(&mut self.file, libc::TIOCM_CTS)
	}

	pub fn set_dtr(&mut self, state: bool) -> std::io::Result<()> {
		set_pin(&mut self.file, libc::TIOCM_DTR, state)
	}

	pub fn read_dsr(&mut self) -> std::io::Result<bool> {
		read_pin(&mut self.file, libc::TIOCM_DSR)
	}

	pub fn read_ri(&mut self) -> std::io::Result<bool> {
		read_pin(&mut self.file, libc::TIOCM_RI)
	}

	pub fn read_cd(&mut self) -> std::io::Result<bool> {
		read_pin(&mut self.file, libc::TIOCM_CD)
	}
}

/// Wait for a file to be readable or writable.
fn poll(file: &mut std::fs::File, events: std::os::raw::c_short, timeout_ms: u32) -> std::io::Result<bool> {
	unsafe {
		let mut poll_fd = libc::pollfd {
			fd: file.as_raw_fd(),
			events,
			revents: 0,
		};
		check(libc::poll(&mut poll_fd, 1, timeout_ms as i32))?;
		Ok(poll_fd.revents != 0)
	}
}

fn set_pin(file: &mut std::fs::File, pin: c_int, state: bool) -> std::io::Result<()> {
	unsafe {
		if state {
			check(libc::ioctl(file.as_raw_fd(), libc::TIOCMBIS, &pin))?;
		} else {
			check(libc::ioctl(file.as_raw_fd(), libc::TIOCMBIC, &pin))?;
		}
		Ok(())
	}
}

fn read_pin(file: &mut std::fs::File, pin: c_int) -> std::io::Result<bool> {
	unsafe {
		let mut bits: c_int = 0;
		check(libc::ioctl(file.as_raw_fd(), libc::TIOCMGET, &mut bits))?;
		Ok(bits & pin != 0)
	}
}

/// Check the return value of a syscall for errors.
fn check(ret: i32) -> std::io::Result<i32> {
	if ret == -1 {
		Err(std::io::Error::last_os_error())
	} else {
		Ok(ret)
	}
}

/// Create an std::io::Error with custom message.
fn other_error<E>(msg: E) -> std::io::Error
where
	E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
	std::io::Error::new(std::io::ErrorKind::Other, msg)
}

impl Settings {
	pub fn set_baud_rate(&mut self, baud_rate: u32) -> std::io::Result<()> {
		let speed = match baud_rate {
			50 => libc::B50,
			75 => libc::B75,
			110 => libc::B110,
			134 => libc::B134,
			150 => libc::B150,
			200 => libc::B200,
			300 => libc::B300,
			600 => libc::B600,
			1200 => libc::B1200,
			1800 => libc::B1800,
			2400 => libc::B2400,
			4800 => libc::B4800,
			9600 => libc::B9600,
			19200 => libc::B19200,
			38400 => libc::B38400,
			57600 => libc::B57600,
			115200 => libc::B115200,
			230400 => libc::B230400,
			_ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "unsupported baud rate")),
		};
		unsafe {
			check(libc::cfsetospeed(&mut self.termios, speed))?;
			check(libc::cfsetispeed(&mut self.termios, speed))?;
		}
		Ok(())
	}

	pub fn get_baud_rate(&self) -> std::io::Result<u32> {
		let speed = unsafe { libc::cfgetospeed(&self.termios) };
		let speed = match speed {
			libc::B50 => 50,
			libc::B75 => 75,
			libc::B110 => 110,
			libc::B134 => 134,
			libc::B150 => 150,
			libc::B200 => 200,
			libc::B300 => 300,
			libc::B600 => 600,
			libc::B1200 => 1200,
			libc::B1800 => 1800,
			libc::B2400 => 2400,
			libc::B4800 => 4800,
			libc::B9600 => 9600,
			libc::B19200 => 19200,
			libc::B38400 => 38400,
			libc::B57600 => 57600,
			libc::B115200 => 115200,
			libc::B230400 => 230400,
			_ => return Err(other_error("unrecognized baud rate")),
		};
		Ok(speed)
	}

	pub fn set_char_size(&mut self, char_size: crate::CharSize) {
		let bits = match char_size {
			crate::CharSize::Bits5 => libc::CS5,
			crate::CharSize::Bits6 => libc::CS6,
			crate::CharSize::Bits7 => libc::CS7,
			crate::CharSize::Bits8 => libc::CS8,
		};
		self.termios.c_cflag = (self.termios.c_cflag & !libc::CSIZE) | bits;
	}

	pub fn get_char_size(&self) -> std::io::Result<crate::CharSize> {
		let bits = self.termios.c_cflag & libc::CSIZE;
		match bits {
			libc::CS5 => Ok(crate::CharSize::Bits5),
			libc::CS6 => Ok(crate::CharSize::Bits6),
			libc::CS7 => Ok(crate::CharSize::Bits7),
			libc::CS8 => Ok(crate::CharSize::Bits8),
			_ => Err(other_error("unrecognized char size")),
		}
	}

	pub fn set_stop_bits(&mut self, stop_bits: crate::StopBits) {
		match stop_bits {
			crate::StopBits::One => self.termios.c_cflag &= !libc::CSTOPB,
			crate::StopBits::Two => self.termios.c_cflag |= libc::CSTOPB,
		};
	}

	pub fn get_stop_bits(&self) -> std::io::Result<crate::StopBits> {
		if self.termios.c_cflag & libc::CSTOPB == 0 {
			Ok(crate::StopBits::One)
		} else {
			Ok(crate::StopBits::Two)
		}
	}

	pub fn set_parity(&mut self, parity: crate::Parity) {
		match parity {
			crate::Parity::None => self.termios.c_cflag = self.termios.c_cflag & !libc::PARODD & !libc::PARENB,
			crate::Parity::Even => self.termios.c_cflag = self.termios.c_cflag & !libc::PARODD | libc::PARENB,
			crate::Parity::Odd => self.termios.c_cflag = self.termios.c_cflag | libc::PARODD | libc::PARENB,
		};
	}

	pub fn get_parity(&self) -> std::io::Result<crate::Parity> {
		if self.termios.c_cflag & libc::PARENB == 0 {
			Ok(crate::Parity::None)
		} else if self.termios.c_cflag & libc::PARODD == 0 {
			Ok(crate::Parity::Odd)
		} else {
			Ok(crate::Parity::Even)
		}
	}

	pub fn set_flow_control(&mut self, flow_control: crate::FlowControl) {
		match flow_control {
			crate::FlowControl::None => {
				self.termios.c_iflag &= !(libc::IXON | libc::IXOFF);
				self.termios.c_cflag &= !libc::CRTSCTS;
			},
			crate::FlowControl::XonXoff => {
				self.termios.c_iflag |= libc::IXON | libc::IXOFF;
				self.termios.c_cflag &= !libc::CRTSCTS;
			},
			crate::FlowControl::RtsCts => {
				self.termios.c_iflag &= !(libc::IXON | libc::IXOFF);
				self.termios.c_cflag |= libc::CRTSCTS;
			},
		};
	}

	pub fn get_flow_control(&self) -> std::io::Result<crate::FlowControl> {
		let ixon = self.termios.c_iflag & libc::IXON != 0;
		let ixoff = self.termios.c_iflag & libc::IXOFF != 0;
		let crtscts = self.termios.c_cflag & libc::CRTSCTS != 0;

		if !crtscts && !ixon && !ixoff {
			Ok(crate::FlowControl::None)
		} else if crtscts && !ixon && !ixoff {
			Ok(crate::FlowControl::XonXoff)
		} else if !crtscts && ixon && ixoff {
			Ok(crate::FlowControl::RtsCts)
		} else {
			Err(other_error("unknown flow control configuration"))
		}
	}
}

impl PartialEq for Settings {
	fn eq(&self, other: &Self) -> bool {
		let a = &self.termios;
		let b = &other.termios;
		a.c_cflag == b.c_cflag
			&& a.c_iflag == b.c_iflag
			&& a.c_oflag == b.c_oflag
			&& a.c_lflag == b.c_lflag
			&& a.c_line == b.c_line
			&& a.c_ispeed == b.c_ispeed
			&& a.c_ospeed == b.c_ospeed
	}
}
