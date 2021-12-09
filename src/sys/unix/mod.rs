use cfg_if::cfg_if;
use std::io::{IoSlice, IoSliceMut};
use std::os::raw::c_int;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::time::Duration;

pub struct SerialPort {
	pub file: std::fs::File,
	pub read_timeout_ms: u32,
	pub write_timeout_ms: u32,
}

cfg_if! {
	if #[cfg(any(
			target_os = "dragonfly",
			target_os = "freebsd",
			target_os = "ios",
			target_os = "macos",
			target_os = "netbsd",
			target_os = "openbsd",
	))] {
		mod bsd;
		pub use bsd::*;

	} else if #[cfg(any(
		target_os = "linux",
		target_os = "android",
	))] {
		mod linux;
		pub use linux::*;

	} else if #[cfg(any(
		target_os = "illumos",
		target_os = "solaris",
	))] {
		mod solarish;
		pub use solarish::*;

	} else {
		mod other;
		pub use other::*;
	}
}

cfg_if! {
	if #[cfg(all(
		any(target_os = "android", target_os = "linux"),
		not(any(target_arch = "powerpc", target_arch = "powerpc64"))
	))]
	{
		#[derive(Clone)]
		pub struct Settings {
			pub termios: libc::termios2,
		}

		impl Settings {
			fn get_from_file(file: &std::fs::File) -> std::io::Result<Self> {
				unsafe {
					let mut termios = std::mem::zeroed();
					check(libc::ioctl(file.as_raw_fd(), libc::TCGETS2 as _, &mut termios))?;
					Ok(Settings { termios })
				}
			}

			fn set_on_file(&self, file: &mut std::fs::File) -> std::io::Result<()> {
				unsafe {
					check(libc::ioctl(file.as_raw_fd(), libc::TCSETSW2 as _, &self.termios))?;
					Ok(())
				}
			}
		}
	} else {
		#[derive(Clone)]
		pub struct Settings {
			pub termios: libc::termios,
		}

		impl Settings {
			fn get_from_file(file: &std::fs::File) -> std::io::Result<Self> {
				unsafe {
					let mut termios = std::mem::zeroed();
					check(libc::tcgetattr(file.as_raw_fd(), &mut termios))?;
					Ok(Settings { termios })
				}
			}

			fn set_on_file(&self, file: &mut std::fs::File) -> std::io::Result<()> {
				unsafe {
					check(libc::tcsetattr(file.as_raw_fd(), libc::TCSADRAIN, &self.termios))?;
					Ok(())
				}
			}
		}
	}
}

impl SerialPort {
	pub fn open(path: &Path) -> std::io::Result<Self> {
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
		Settings::get_from_file(&self.file)
	}

	pub fn set_configuration(&mut self, settings: &Settings) -> std::io::Result<()> {
		let mut settings = settings.clone();
		unsafe {
			libc::cfmakeraw(&mut settings.termios as *mut _ as *mut libc::termios);
		}
		settings.set_on_file(&mut self.file)?;
		let applied_settings = self.get_configuration()?;
		if applied_settings != settings {
			Err(other_error("failed to apply some or all settings"))
		} else {
			Ok(())
		}
	}

	pub fn set_read_timeout(&mut self, timeout: Duration) -> std::io::Result<()> {
		self.read_timeout_ms = timeout.as_millis().try_into().unwrap_or(u32::MAX);
		Ok(())
	}

	pub fn get_read_timeout(&self) -> std::io::Result<Duration> {
		Ok(Duration::from_millis(self.read_timeout_ms.into()))
	}

	pub fn set_write_timeout(&mut self, timeout: Duration) -> std::io::Result<()> {
		self.write_timeout_ms = timeout.as_millis().try_into().unwrap_or(u32::MAX);
		Ok(())
	}

	pub fn get_write_timeout(&self) -> std::io::Result<Duration> {
		Ok(Duration::from_millis(self.write_timeout_ms.into()))
	}

	pub fn read(&self, buf: &mut [u8]) -> std::io::Result<usize> {
		if !poll(&self.file, libc::POLLIN, self.read_timeout_ms)? {
			return Err(std::io::ErrorKind::TimedOut.into());
		}
		unsafe {
			loop {
				let result = check_isize(libc::read(
					self.file.as_raw_fd(),
					buf.as_mut_ptr().cast(),
					buf.len() as _,
				));
				match result {
					Err(ref e) if e.raw_os_error() == Some(libc::EINTR) => continue,
					x => return x,
				}
			}
		}
	}

	pub fn read_vectored(&self, buf: &mut [IoSliceMut<'_>]) -> std::io::Result<usize> {
		if !poll(&self.file, libc::POLLIN, self.read_timeout_ms)? {
			return Err(std::io::ErrorKind::TimedOut.into());
		}
		unsafe {
			loop {
				let result = check_isize(libc::readv(
					self.file.as_raw_fd(),
					buf.as_mut_ptr().cast(),
					buf.len() as _,
				));
				match result {
					Err(ref e) if e.raw_os_error() == Some(libc::EINTR) => continue,
					x => return x,
				}
			}
		}
	}

	pub fn is_read_vectored(&self) -> bool {
		true
	}

	pub fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
		if !poll(&self.file, libc::POLLOUT, self.read_timeout_ms)? {
			return Err(std::io::ErrorKind::TimedOut.into());
		}
		unsafe {
			loop {
				let result = check_isize(libc::write(self.file.as_raw_fd(), buf.as_ptr().cast(), buf.len() as _));
				match result {
					Err(ref e) if e.raw_os_error() == Some(libc::EINTR) => continue,
					x => return x,
				}
			}
		}
	}

	pub fn write_vectored(&self, buf: &[IoSlice<'_>]) -> std::io::Result<usize> {
		if !poll(&self.file, libc::POLLOUT, self.read_timeout_ms)? {
			return Err(std::io::ErrorKind::TimedOut.into());
		}
		unsafe {
			loop {
				let result = check_isize(libc::writev(self.file.as_raw_fd(), buf.as_ptr().cast(), buf.len() as _));
				match result {
					Err(ref e) if e.raw_os_error() == Some(libc::EINTR) => continue,
					x => return x,
				}
			}
		}
	}

	pub fn is_write_vectored(&self) -> bool {
		true
	}

	pub fn flush_output(&self) -> std::io::Result<()> {
		unsafe {
			check(libc::tcdrain(self.file.as_raw_fd()))?;
			Ok(())
		}
	}

	pub fn discard_buffers(&self, discard_input: bool, discard_output: bool) -> std::io::Result<()> {
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

	pub fn set_rts(&self, state: bool) -> std::io::Result<()> {
		set_pin(&self.file, libc::TIOCM_RTS, state)
	}

	pub fn read_cts(&self) -> std::io::Result<bool> {
		read_pin(&self.file, libc::TIOCM_CTS)
	}

	pub fn set_dtr(&self, state: bool) -> std::io::Result<()> {
		set_pin(&self.file, libc::TIOCM_DTR, state)
	}

	pub fn read_dsr(&self) -> std::io::Result<bool> {
		read_pin(&self.file, libc::TIOCM_DSR)
	}

	pub fn read_ri(&self) -> std::io::Result<bool> {
		read_pin(&self.file, libc::TIOCM_RI)
	}

	pub fn read_cd(&self) -> std::io::Result<bool> {
		read_pin(&self.file, libc::TIOCM_CD)
	}
}

/// Wait for a file to be readable or writable.
fn poll(file: &std::fs::File, events: std::os::raw::c_short, timeout_ms: u32) -> std::io::Result<bool> {
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

fn set_pin(file: &std::fs::File, pin: c_int, state: bool) -> std::io::Result<()> {
	unsafe {
		if state {
			check(libc::ioctl(file.as_raw_fd(), libc::TIOCMBIS as _, &pin))?;
		} else {
			check(libc::ioctl(file.as_raw_fd(), libc::TIOCMBIC as _, &pin))?;
		}
		Ok(())
	}
}

fn read_pin(file: &std::fs::File, pin: c_int) -> std::io::Result<bool> {
	unsafe {
		let mut bits: c_int = 0;
		check(libc::ioctl(file.as_raw_fd(), libc::TIOCMGET as _, &mut bits))?;
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

/// Check the return value of a syscall for errors.
fn check_isize(ret: isize) -> std::io::Result<usize> {
	if ret == -1 {
		Err(std::io::Error::last_os_error())
	} else {
		Ok(ret as usize)
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
		cfg_if! {
			if #[cfg(any(
				target_os = "dragonfly",
				target_os = "freebsd",
				target_os = "ios",
				target_os = "macos",
				target_os = "netbsd",
				target_os = "openbsd",
			))] {
				unsafe {
					check(libc::cfsetospeed(&mut self.termios, baud_rate as _))?;
					check(libc::cfsetispeed(&mut self.termios, baud_rate as _))?;
					Ok(())
				}
			} else if #[cfg(all(
				any(target_os = "android", target_os = "linux"),
				not(any(target_arch = "powerpc", target_arch = "powerpc64"))
			))]
			{
				// Always use `BOTHER` because we can't be bothered to use cfsetospeed/cfsetispeed for standard values.
				//
				// Also, we don't actually have a termios struct to pass to cfsetospeed or cfsetispeed.
				self.termios.c_cflag &= !(libc::CBAUD | libc::CIBAUD);
				self.termios.c_cflag |= libc::BOTHER;
				self.termios.c_cflag |= libc::BOTHER << libc::IBSHIFT;
				self.termios.c_ospeed = baud_rate;
				self.termios.c_ispeed = baud_rate;
				Ok(())
			} else {
				unsafe {
					let &(constant, _) = BAUD_RATES.iter()
						.find(|&&(_, bits_per_second)| bits_per_second == baud_rate)
						.ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "unsupported baud rate"))?;
					check(libc::cfsetospeed(&mut self.termios, constant))?;
					check(libc::cfsetispeed(&mut self.termios, constant))?;
					Ok(())
				}
			}
		}
	}

	pub fn get_baud_rate(&self) -> std::io::Result<u32> {
		cfg_if! {
			if #[cfg(any(
				target_os = "dragonfly",
				target_os = "freebsd",
				target_os = "ios",
				target_os = "macos",
				target_os = "netbsd",
				target_os = "openbsd",
			))]
			{
				unsafe {
					return Ok(libc::cfgetospeed(&self.termios).try_into().unwrap());
				}
			} else {
				#[cfg(all(
					any(target_os = "android", target_os = "linux"),
					not(any(target_arch = "powerpc", target_arch = "powerpc64"))
				))]
				if self.termios.c_cflag & libc::CBAUD == libc::BOTHER {
					return Ok(self.termios.c_ospeed);
				}

				unsafe {
					let termios_speed = libc::cfgetospeed(&self.termios as *const _ as *const _ );
					let &(_, bits_per_second) = BAUD_RATES.iter()
						.find(|&&(constant, _)| constant == termios_speed)
						.ok_or_else(|| other_error("unrecognized baud rate"))?;
					Ok(bits_per_second)
				}
			}
		}
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

		cfg_if::cfg_if! {
			if #[cfg(any(target_os = "android", target_os = "linux"))] {
				use libc::{CBAUD, CBAUDEX, IBSHIFT};
				let no_baud = !(CBAUD | CBAUDEX | (CBAUD | CBAUDEX) << IBSHIFT);
				let same = true;
				let same = same && a.c_cflag & no_baud == b.c_cflag & no_baud;
				let same = same && a.c_iflag == b.c_iflag;
				let same = same && a.c_oflag == b.c_oflag;
				let same = same && a.c_lflag == b.c_lflag;
				let same = same && a.c_cc == b.c_cc;
				let same = same && a.c_line == b.c_line;
				if !same {
					return false;
				}

				// TODO: Deal with input speed != output speed
				if let (Ok(speed_a), Ok(speed_b)) = (self.get_baud_rate(), other.get_baud_rate()) {
					speed_a == speed_b
				} else {
					false
				}
			} else {
				let same = true;
				let same = same && a.c_cflag == b.c_cflag;
				let same = same && a.c_iflag == b.c_iflag;
				let same = same && a.c_oflag == b.c_oflag;
				let same = same && a.c_lflag == b.c_lflag;
				let same = same && a.c_cc == b.c_cc;
				same
			}
		}
	}
}
