use std::ffi::OsStr;
use std::io::{IoSlice, IoSliceMut};
use std::os::raw::c_int;
use std::os::unix::io::AsRawFd;
use std::time::Duration;

pub struct Inner {
	pub file: std::fs::File,
	pub read_timeout_ms: u32,
	pub write_timeout_ms: u32,
}

pub fn open(path: &OsStr) -> std::io::Result<Inner> {
	let file = std::fs::OpenOptions::new()
		.read(true)
		.write(true)
		.create(false)
		.open(path)?;

	Ok(from_file(file))
}

pub fn from_file(file: std::fs::File) -> Inner {
	Inner {
		file,
		read_timeout_ms: 100,
		write_timeout_ms: 100,
	}
}

pub fn configure(inner: &mut Inner, settings: &crate::SerialSettings) -> std::io::Result<()> {
	unsafe {
		let mut termios: libc::termios = std::mem::zeroed();
		check(libc::tcgetattr(inner.file.as_raw_fd(), &mut termios))?;
		libc::cfmakeraw(&mut termios);

		termios::set_baud_rate(&mut termios, settings.baud_rate)?;
		termios::set_char_size(&mut termios, settings.char_size);
		termios::set_stop_bits(&mut termios, settings.stop_bits);
		termios::set_parity(&mut termios, settings.parity);
		termios::set_flow_control(&mut termios, settings.flow_control);
		check(libc::tcsetattr(inner.file.as_raw_fd(), libc::TCSADRAIN, &termios))?;

		let mut real_termios: libc::termios = std::mem::zeroed();
		check(libc::tcgetattr(inner.file.as_raw_fd(), &mut real_termios))?;
		if !termios::is_same(&real_termios, &termios) {
			Err(other_error("failed to apply some or all settings"))
		} else {
			Ok(())
		}
	}
}

pub fn get_configuration(inner: &Inner) -> std::io::Result<crate::SerialSettings> {
	let mut termios;
	unsafe {
		termios = std::mem::zeroed();
		check(libc::tcgetattr(inner.file.as_raw_fd(), &mut termios))?;
	}

	Ok(crate::SerialSettings {
		baud_rate: termios::get_baud_rate(&termios)?,
		char_size: termios::get_char_size(&termios)?,
		stop_bits: termios::get_stop_bits(&termios),
		parity: termios::get_parity(&termios),
		flow_control: termios::get_flow_control(&termios)?,
	})
}

pub fn set_read_timeout(inner: &mut Inner, timeout: Duration) -> std::io::Result<()> {
	use std::convert::TryInto;
	inner.read_timeout_ms = timeout.as_millis().try_into().unwrap_or(u32::MAX);
	Ok(())
}

pub fn get_read_timeout(inner: &Inner) -> std::io::Result<Duration> {
	Ok(Duration::from_millis(inner.read_timeout_ms.into()))
}

pub fn set_write_timeout(inner: &mut Inner, timeout: Duration) -> std::io::Result<()> {
	use std::convert::TryInto;
	inner.write_timeout_ms = timeout.as_millis().try_into().unwrap_or(u32::MAX);
	Ok(())
}

pub fn get_write_timeout(inner: &Inner) -> std::io::Result<Duration> {
	Ok(Duration::from_millis(inner.write_timeout_ms.into()))
}

pub fn read(inner: &mut Inner, buf: &mut [u8]) -> std::io::Result<usize> {
	use std::io::Read;
	if !poll(&mut inner.file, libc::POLLIN, inner.read_timeout_ms)? {
		Err(std::io::ErrorKind::TimedOut.into())
	} else {
		inner.file.read(buf)
	}
}

pub fn read_vectored(inner: &mut Inner, buf: &mut [IoSliceMut<'_>]) -> std::io::Result<usize> {
	use std::io::Read;
	if !poll(&mut inner.file, libc::POLLIN, inner.read_timeout_ms)? {
		Err(std::io::ErrorKind::TimedOut.into())
	} else {
		inner.file.read_vectored(buf)
	}
}

pub fn write(inner: &mut Inner, buf: &[u8]) -> std::io::Result<usize> {
	use std::io::Write;
	if !poll(&mut inner.file, libc::POLLOUT, inner.read_timeout_ms)? {
		Err(std::io::ErrorKind::TimedOut.into())
	} else {
		inner.file.write(buf)
	}
}

pub fn write_vectored(inner: &mut Inner, buf: &[IoSlice<'_>]) -> std::io::Result<usize> {
	use std::io::Write;
	if !poll(&mut inner.file, libc::POLLOUT, inner.read_timeout_ms)? {
		Err(std::io::ErrorKind::TimedOut.into())
	} else {
		inner.file.write_vectored(buf)
	}
}

pub fn flush_output(inner: &Inner) -> std::io::Result<()> {
	unsafe {
		check(libc::tcdrain(inner.file.as_raw_fd()))?;
		Ok(())
	}
}

pub fn discard_buffers(inner: &mut Inner, discard_input: bool, discard_output: bool) -> std::io::Result<()> {
	unsafe {
		let mut flags = 0;
		if discard_input {
			flags |= libc::TCIFLUSH;
		}
		if discard_output {
			flags |= libc::TCOFLUSH;
		}
		check(libc::tcflush(inner.file.as_raw_fd(), flags))?;
		Ok(())
	}
}

pub fn set_rts(inner: &mut Inner, state: bool) -> std::io::Result<()> {
	set_pin(&mut inner.file, libc::TIOCM_RTS, state)
}

pub fn read_cts(inner: &mut Inner) -> std::io::Result<bool> {
	read_pin(&mut inner.file, libc::TIOCM_CTS)
}

pub fn set_dtr(inner: &mut Inner, state: bool) -> std::io::Result<()> {
	set_pin(&mut inner.file, libc::TIOCM_DTR, state)
}

pub fn read_dsr(inner: &mut Inner) -> std::io::Result<bool> {
	read_pin(&mut inner.file, libc::TIOCM_DSR)
}

pub fn read_ri(inner: &mut Inner) -> std::io::Result<bool> {
	read_pin(&mut inner.file, libc::TIOCM_RI)
}

pub fn read_cd(inner: &mut Inner) -> std::io::Result<bool> {
	read_pin(&mut inner.file, libc::TIOCM_CD)
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

/// Functions to manipulate a termios structure.
mod termios {
	use super::{check, other_error};

	pub fn is_same(a: &libc::termios, b: &libc::termios) -> bool {
		unsafe {
			a.c_cflag == b.c_cflag
				&& a.c_iflag == b.c_iflag
				&& a.c_oflag == b.c_oflag
				&& a.c_lflag == b.c_lflag
				&& libc::cfgetispeed(a) == libc::cfgetispeed(b)
				&& libc::cfgetospeed(a) == libc::cfgetospeed(b)
		}
	}

	pub fn set_baud_rate(termios: &mut libc::termios, baud_rate: u32) -> std::io::Result<()> {
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
			check(libc::cfsetospeed(termios, speed))?;
			check(libc::cfsetispeed(termios, speed))?;
		}
		Ok(())
	}

	pub fn get_baud_rate(termios: &libc::termios) -> std::io::Result<u32> {
		let speed = unsafe { libc::cfgetospeed(termios) };
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

	pub fn set_char_size(termios: &mut libc::termios, char_size: crate::CharSize) {
		let bits = match char_size {
			crate::Bits5 => libc::CS5,
			crate::Bits6 => libc::CS6,
			crate::Bits7 => libc::CS7,
			crate::Bits8 => libc::CS8,
		};
		termios.c_cflag = (termios.c_cflag & !libc::CSIZE) | bits;
	}

	pub fn get_char_size(termios: &libc::termios) -> std::io::Result<crate::CharSize> {
		let bits = termios.c_cflag & libc::CSIZE;
		match bits {
			libc::CS5 => Ok(crate::Bits5),
			libc::CS6 => Ok(crate::Bits6),
			libc::CS7 => Ok(crate::Bits7),
			libc::CS8 => Ok(crate::Bits8),
			_ => Err(other_error("unrecognized char size")),
		}
	}

	pub fn set_stop_bits(termios: &mut libc::termios, stop_bits: crate::StopBits) {
		match stop_bits {
			crate::Stop1 => termios.c_cflag &= !libc::CSTOPB,
			crate::Stop2 => termios.c_cflag |= libc::CSTOPB,
		};
	}

	pub fn get_stop_bits(termios: &libc::termios) -> crate::StopBits {
		if termios.c_cflag & libc::CSTOPB == 0 {
			crate::Stop1
		} else {
			crate::Stop2
		}
	}

	pub fn set_parity(termios: &mut libc::termios, parity: crate::Parity) {
		match parity {
			crate::ParityNone => termios.c_cflag = termios.c_cflag & !libc::PARODD & !libc::PARENB,
			crate::ParityEven => termios.c_cflag = termios.c_cflag & !libc::PARODD | libc::PARENB,
			crate::ParityOdd => termios.c_cflag = termios.c_cflag | libc::PARODD | libc::PARENB,
		};
	}

	pub fn get_parity(termios: &libc::termios) -> crate::Parity {
		if termios.c_cflag & libc::PARENB == 0 {
			crate::ParityNone
		} else if termios.c_cflag & libc::PARODD == 0 {
			crate::ParityOdd
		} else {
			crate::ParityEven
		}
	}

	pub fn set_flow_control(termios: &mut libc::termios, flow_control: crate::FlowControl) {
		match flow_control {
			crate::FlowControlNone => {
				termios.c_iflag &= !(libc::IXON | libc::IXOFF);
				termios.c_cflag &= !libc::CRTSCTS;
			},
			crate::FlowControlXonXoff => {
				termios.c_iflag |= libc::IXON | libc::IXOFF;
				termios.c_cflag &= !libc::CRTSCTS;
			},
			crate::FlowControlRtsCts => {
				termios.c_iflag &= !(libc::IXON | libc::IXOFF);
				termios.c_cflag |= libc::CRTSCTS;
			},
		};
	}

	pub fn get_flow_control(termios: &libc::termios) -> std::io::Result<crate::FlowControl> {
		let ixon = termios.c_iflag & libc::IXON != 0;
		let ixoff = termios.c_iflag & libc::IXOFF != 0;
		let crtscts = termios.c_cflag & libc::CRTSCTS != 0;

		if !crtscts && !ixon && !ixoff {
			Ok(crate::FlowControlNone)
		} else if crtscts && !ixon && !ixoff {
			Ok(crate::FlowControlXonXoff)
		} else if !crtscts && ixon && ixoff {
			Ok(crate::FlowControlRtsCts)
		} else {
			Err(other_error("unknown flow control configuration"))
		}
	}
}
