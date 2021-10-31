use std::ffi::OsStr;
use std::ffi::OsString;
use std::io::{IoSlice, IoSliceMut};
use std::os::windows::io::AsRawHandle;
use std::time::Duration;

use winapi::um::commapi;
use winapi::um::winbase;
use winapi::shared::minwindef::BOOL;

pub struct SerialPort {
	pub file: std::fs::File,
}

#[derive(Clone)]
pub struct Settings {
	dcb: winbase::DCB,
}

pub fn open(name: &OsStr) -> std::io::Result<SerialPort> {
	// Use the win32 device namespace, otherwise we're limited to COM1-9.
	// This also works with higher numbers.
	// https://docs.microsoft.com/en-us/windows/win32/fileio/naming-a-file#win32-device-namespaces
	let mut path = OsString::from("\\\\.\\");
	path.push(name);

	let file = std::fs::OpenOptions::new()
		.read(true)
		.write(true)
		.create(false)
		.open(path)?;
	Ok(from_file(file))
}

pub fn from_file(file: std::fs::File) -> SerialPort {
	SerialPort { file }
}

pub fn get_configuration(inner: &SerialPort) -> std::io::Result<Settings> {
	unsafe {
		let mut dcb: winbase::DCB = std::mem::zeroed();
		check_bool(commapi::GetCommState(inner.file.as_raw_handle(), &mut dcb))?;
		Ok(Settings {
			dcb,
		})
	}
}

pub fn set_configuration(inner: &mut SerialPort, settings: &Settings) -> std::io::Result<()> {
	unsafe {
		let mut settings = settings.clone();
		check_bool(commapi::SetCommState(inner.file.as_raw_handle(), &mut settings.dcb))
	}
}

pub fn set_read_timeout(inner: &mut SerialPort, timeout: Duration) -> std::io::Result<()> {
	use std::convert::TryInto;
	unsafe {
		let mut timeouts = std::mem::zeroed();
		check_bool(commapi::GetCommTimeouts(inner.file.as_raw_handle(), &mut timeouts))?;
		timeouts.ReadIntervalTimeout = 0;
		timeouts.ReadTotalTimeoutMultiplier = 0;
		timeouts.ReadTotalTimeoutConstant = timeout.as_millis().try_into().unwrap_or(u32::MAX);
		check_bool(commapi::SetCommTimeouts(inner.file.as_raw_handle(), &mut timeouts))
	}
}

pub fn get_read_timeout(inner: &SerialPort) -> std::io::Result<Duration> {
	unsafe {
		let mut timeouts = std::mem::zeroed();
		check_bool(commapi::GetCommTimeouts(inner.file.as_raw_handle(), &mut timeouts))?;
		Ok(Duration::from_millis(timeouts.ReadTotalTimeoutConstant.into()))
	}
}

pub fn set_write_timeout(inner: &mut SerialPort, timeout: Duration) -> std::io::Result<()> {
	use std::convert::TryInto;
	unsafe {
		let mut timeouts = std::mem::zeroed();
		check_bool(commapi::GetCommTimeouts(inner.file.as_raw_handle(), &mut timeouts))?;
		timeouts.WriteTotalTimeoutMultiplier = 0;
		timeouts.WriteTotalTimeoutConstant = timeout.as_millis().try_into().unwrap_or(u32::MAX);
		check_bool(commapi::SetCommTimeouts(inner.file.as_raw_handle(), &mut timeouts))
	}
}

pub fn get_write_timeout(inner: &SerialPort) -> std::io::Result<Duration> {
	unsafe {
		let mut timeouts = std::mem::zeroed();
		check_bool(commapi::GetCommTimeouts(inner.file.as_raw_handle(), &mut timeouts))?;
		Ok(Duration::from_millis(timeouts.WriteTotalTimeoutConstant.into()))
	}
}

pub fn read(inner: &mut SerialPort, buf: &mut [u8]) -> std::io::Result<usize> {
	use std::io::Read;
	inner.file.read(buf)
}

pub fn read_vectored(inner: &mut SerialPort, buf: &mut [IoSliceMut<'_>]) -> std::io::Result<usize> {
	// TODO: Use read timeout
	use std::io::Read;
	inner.file.read_vectored(buf)
}

pub fn write(inner: &mut SerialPort, buf: &[u8]) -> std::io::Result<usize> {
	use std::io::Write;
	inner.file.write(buf)
}

pub fn write_vectored(inner: &mut SerialPort, buf: &[IoSlice<'_>]) -> std::io::Result<usize> {
	use std::io::Write;
	inner.file.write_vectored(buf)
}

pub fn flush_output(inner: &SerialPort) -> std::io::Result<()> {
	unsafe {
		check_bool(winapi::um::fileapi::FlushFileBuffers(inner.file.as_raw_handle()))
	}
}

pub fn discard_buffers(inner: &mut SerialPort, discard_input: bool, discard_output: bool) -> std::io::Result<()> {
	unsafe {
		let mut flags = 0;
		if discard_input {
			flags |= winbase::PURGE_RXCLEAR;
		}
		if discard_output {
			flags |= winbase::PURGE_TXCLEAR;
		}
		check_bool(commapi::PurgeComm(inner.file.as_raw_handle(), flags))
	}
}


pub fn set_rts(inner: &mut SerialPort, state: bool) -> std::io::Result<()> {
	if state {
		escape_comm_function(&mut inner.file, winbase::SETRTS)
	} else {
		escape_comm_function(&mut inner.file, winbase::CLRRTS)
	}
}

pub fn read_cts(inner: &mut SerialPort) -> std::io::Result<bool> {
	read_pin(&mut inner.file, winbase::MS_CTS_ON)
}

pub fn set_dtr(inner: &mut SerialPort, state: bool) -> std::io::Result<()> {
	if state {
		escape_comm_function(&mut inner.file, winbase::SETDTR)
	} else {
		escape_comm_function(&mut inner.file, winbase::CLRDTR)
	}
}

pub fn read_dsr(inner: &mut SerialPort) -> std::io::Result<bool> {
	read_pin(&mut inner.file, winbase::MS_DSR_ON)
}

pub fn read_ri(inner: &mut SerialPort) -> std::io::Result<bool> {
	read_pin(&mut inner.file, winbase::MS_RING_ON)
}

pub fn read_cd(inner: &mut SerialPort) -> std::io::Result<bool> {
	// RLSD or Receive Line Signal Detect is the same as Carrier Detect.
	//
	// I think.
	read_pin(&mut inner.file, winbase::MS_RLSD_ON)
}

fn escape_comm_function(file: &mut std::fs::File, function: u32) -> std::io::Result<()> {
	unsafe {
		check_bool(commapi::EscapeCommFunction(file.as_raw_handle(), function))
	}
}

fn read_pin(file: &mut std::fs::File, pin: u32) -> std::io::Result<bool> {
	unsafe {
		let mut bits: u32 = 0;
		check_bool(commapi::GetCommModemStatus(file.as_raw_handle(), &mut bits))?;
		Ok(bits & pin != 0)
	}
}

/// Check the return value of a syscall for errors.
fn check_bool(ret: BOOL) -> std::io::Result<()> {
	if ret == 0 {
		Err(std::io::Error::last_os_error())
	} else {
		Ok(())
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
		self.dcb.BaudRate = baud_rate;
		Ok(())
	}

	pub fn get_baud_rate(&self) -> std::io::Result<u32> {
		Ok(self.dcb.BaudRate)
	}

	pub fn set_char_size(&mut self, char_size: crate::CharSize) {
		self.dcb.ByteSize = match char_size {
			crate::CharSize::Bits5 => 5,
			crate::CharSize::Bits6 => 6,
			crate::CharSize::Bits7 => 7,
			crate::CharSize::Bits8 => 8,
		};
	}

	pub fn get_char_size(&self) -> std::io::Result<crate::CharSize> {
		match self.dcb.ByteSize {
			5 => Ok(crate::CharSize::Bits5),
			6 => Ok(crate::CharSize::Bits6),
			7 => Ok(crate::CharSize::Bits7),
			8 => Ok(crate::CharSize::Bits8),
			_ => Err(other_error("unsupported char size")),
		}
	}

	pub fn set_stop_bits(&mut self, stop_bits: crate::StopBits) {
		self.dcb.StopBits = match stop_bits {
			crate::StopBits::One => winbase::ONESTOPBIT,
			crate::StopBits::Two => winbase::TWOSTOPBITS,
		};
	}

	pub fn get_stop_bits(&self) -> std::io::Result<crate::StopBits> {
		match self.dcb.StopBits {
			winbase::ONESTOPBIT => Ok(crate::StopBits::One),
			winbase::TWOSTOPBITS => Ok(crate::StopBits::Two),
			_ => Err(other_error("unsupported stop bits")),
		}
	}

	pub fn set_parity(&mut self, parity: crate::Parity) {
		match parity {
			crate::Parity::None => {
				self.dcb.set_fParity(0);
				self.dcb.Parity = winbase::NOPARITY;
			},
			crate::Parity::Odd => {
				self.dcb.set_fParity(1);
				self.dcb.Parity = winbase::ODDPARITY;
			},
			crate::Parity::Even => {
				self.dcb.set_fParity(1);
				self.dcb.Parity = winbase::EVENPARITY;
			},
		}
	}

	pub fn get_parity(&self) -> std::io::Result<crate::Parity> {
		let parity_enabled = self.dcb.fParity() != 0;
		match self.dcb.Parity {
			winbase::NOPARITY => Ok(crate::Parity::None),
			winbase::ODDPARITY if parity_enabled => Ok(crate::Parity::Odd),
			winbase::EVENPARITY if parity_enabled => Ok(crate::Parity::Even),
			_ => Err(other_error("unsupported parity configuration")),
		}
	}

	pub fn set_flow_control(&mut self, flow_control: crate::FlowControl) {
		match flow_control {
			crate::FlowControl::None => {
				self.dcb.set_fInX(0);
				self.dcb.set_fOutX(0);
				self.dcb.set_fDtrControl(winbase::DTR_CONTROL_DISABLE);
				self.dcb.set_fRtsControl(winbase::RTS_CONTROL_DISABLE);
				self.dcb.set_fOutxCtsFlow(0);
				self.dcb.set_fOutxDsrFlow(0);
			},
			crate::FlowControl::XonXoff => {
				self.dcb.set_fInX(1);
				self.dcb.set_fOutX(1);
				self.dcb.set_fDtrControl(winbase::DTR_CONTROL_DISABLE);
				self.dcb.set_fRtsControl(winbase::RTS_CONTROL_DISABLE);
				self.dcb.set_fOutxCtsFlow(0);
				self.dcb.set_fOutxDsrFlow(0);
			},
			crate::FlowControl::RtsCts => {
				self.dcb.set_fInX(0);
				self.dcb.set_fOutX(0);
				self.dcb.set_fDtrControl(winbase::DTR_CONTROL_DISABLE);
				self.dcb.set_fRtsControl(winbase::RTS_CONTROL_TOGGLE);
				self.dcb.set_fOutxCtsFlow(1);
				self.dcb.set_fOutxDsrFlow(0);
			},
		}
	}

	pub fn get_flow_control(&self) -> std::io::Result<crate::FlowControl> {
		let in_x = self.dcb.fInX() != 0;
		let out_x = self.dcb.fOutX() != 0;
		let out_cts = self.dcb.fOutxCtsFlow() != 0;
		let out_dsr = self.dcb.fOutxDsrFlow() != 0;

		match (in_x, out_x, out_cts, out_dsr, self.dcb.fDtrControl(), self.dcb.fRtsControl()) {
			(false, false, false, false, winbase::DTR_CONTROL_DISABLE, winbase::RTS_CONTROL_DISABLE) => Ok(crate::FlowControl::None),
			(true, true, false, false, winbase::DTR_CONTROL_DISABLE, winbase::RTS_CONTROL_DISABLE) => Ok(crate::FlowControl::XonXoff),
			(false, false, true, false, winbase::DTR_CONTROL_DISABLE, winbase::RTS_CONTROL_TOGGLE) => Ok(crate::FlowControl::RtsCts),
			_ => Err(other_error("unsupported flow control configuration")),
		}
	}
}
