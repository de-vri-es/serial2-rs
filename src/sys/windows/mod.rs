use std::ffi::OsString;
use std::io::{IoSlice, IoSliceMut};
use std::os::windows::io::AsRawHandle;
use std::path::{Path, PathBuf};
use std::time::Duration;

use winapi::um::{
	commapi,
	fileapi,
	ioapiset,
	minwinbase,
	synchapi,
	winbase,
	winnt,
	winreg,
};
use winapi::shared::minwindef::{BOOL, HKEY};
use winapi::shared::winerror;

pub struct SerialPort {
	pub file: std::fs::File,
}

#[derive(Clone)]
pub struct Settings {
	dcb: winbase::DCB,
}

impl SerialPort {
	pub fn open(name: &Path) -> std::io::Result<Self> {
		use std::os::windows::fs::OpenOptionsExt;

		// Use the win32 device namespace, otherwise we're limited to COM1-9.
		// This also works with higher numbers.
		// https://docs.microsoft.com/en-us/windows/win32/fileio/naming-a-file#win32-device-namespaces
		let mut path = OsString::from("\\\\.\\");
		path.push(name.as_os_str());

		let file = std::fs::OpenOptions::new()
			.read(true)
			.write(true)
			.create(false)
			.custom_flags(winbase::FILE_FLAG_OVERLAPPED)
			.open(path)?;

		unsafe {
			let mut timeouts: winbase::COMMTIMEOUTS = std::mem::zeroed();
			check_bool(commapi::GetCommTimeouts(file.as_raw_handle(), &mut timeouts))?;
			timeouts.ReadIntervalTimeout = 10;
			check_bool(commapi::SetCommTimeouts(file.as_raw_handle(), &mut timeouts))?;
		}
		Ok(Self::from_file(file))
	}

	pub fn from_file(file: std::fs::File) -> Self {
		Self { file }
	}

	pub fn get_configuration(&self) -> std::io::Result<Settings> {
		unsafe {
			let mut dcb: winbase::DCB = std::mem::zeroed();
			check_bool(commapi::GetCommState(self.file.as_raw_handle(), &mut dcb))?;
			Ok(Settings {
				dcb,
			})
		}
	}

	pub fn set_configuration(&mut self, settings: &Settings) -> std::io::Result<()> {
		unsafe {
			let mut settings = settings.clone();
			check_bool(commapi::SetCommState(self.file.as_raw_handle(), &mut settings.dcb))
		}
	}

	pub fn set_read_timeout(&mut self, timeout: Duration) -> std::io::Result<()> {
		unsafe {
			let mut timeouts = std::mem::zeroed();
			check_bool(commapi::GetCommTimeouts(self.file.as_raw_handle(), &mut timeouts))?;
			timeouts.ReadIntervalTimeout = 0;
			timeouts.ReadTotalTimeoutMultiplier = 0;
			timeouts.ReadTotalTimeoutConstant = timeout.as_millis().try_into().unwrap_or(u32::MAX);
			check_bool(commapi::SetCommTimeouts(self.file.as_raw_handle(), &mut timeouts))
		}
	}

	pub fn get_read_timeout(&self) -> std::io::Result<Duration> {
		unsafe {
			let mut timeouts = std::mem::zeroed();
			check_bool(commapi::GetCommTimeouts(self.file.as_raw_handle(), &mut timeouts))?;
			Ok(Duration::from_millis(timeouts.ReadTotalTimeoutConstant.into()))
		}
	}

	pub fn set_write_timeout(&mut self, timeout: Duration) -> std::io::Result<()> {
		unsafe {
			let mut timeouts = std::mem::zeroed();
			check_bool(commapi::GetCommTimeouts(self.file.as_raw_handle(), &mut timeouts))?;
			timeouts.WriteTotalTimeoutMultiplier = 0;
			timeouts.WriteTotalTimeoutConstant = timeout.as_millis().try_into().unwrap_or(u32::MAX);
			check_bool(commapi::SetCommTimeouts(self.file.as_raw_handle(), &mut timeouts))
		}
	}

	pub fn get_write_timeout(&self) -> std::io::Result<Duration> {
		unsafe {
			let mut timeouts = std::mem::zeroed();
			check_bool(commapi::GetCommTimeouts(self.file.as_raw_handle(), &mut timeouts))?;
			Ok(Duration::from_millis(timeouts.WriteTotalTimeoutConstant.into()))
		}
	}

	pub fn read(&self, buf: &mut [u8]) -> std::io::Result<usize> {
		unsafe {
			let len = buf.len().try_into().unwrap_or(u32::MAX);
			let mut read = 0;
			let mut overlapped: minwinbase::OVERLAPPED = std::mem::zeroed();
			overlapped.hEvent = check_handle(synchapi::CreateEventA(
				std::ptr::null_mut(), // security attributes
				0, // manual reset
				0, // initial state
				std::ptr::null(), // name
			))?;

			let ret = check_bool(fileapi::ReadFile(
				self.file.as_raw_handle(),
				buf.as_mut_ptr().cast(),
				len,
				&mut read,
				&mut overlapped,
			));
			match ret {
				// Windows reports timeouts as a succesfull transfer of 0 bytes.
				Ok(_) if read == 0 => return Err(std::io::ErrorKind::TimedOut.into()),
				Ok(_) => return Ok(read as usize),
				// BrokenPipe with reads means EOF on Windows.
				Err(ref e) if e.kind() == std::io::ErrorKind::BrokenPipe => return Ok(0),
				Err(ref e) if e.raw_os_error() == Some(winerror::ERROR_IO_PENDING as i32) => (),
				Err(e) => return Err(e),
			}

			wait_async_transfer(&self.file, &mut overlapped)
				.or_else(map_broken_pipe)
		}
	}

	pub fn read_vectored(&self, buf: &mut [IoSliceMut<'_>]) -> std::io::Result<usize> {
		if buf.is_empty() {
			self.read(&mut [])
		} else {
			self.read(&mut buf[0])
		}
	}

	pub fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
		unsafe {
			let len = buf.len().try_into().unwrap_or(u32::MAX);
			let mut written = 0;
			let mut overlapped: minwinbase::OVERLAPPED = std::mem::zeroed();
			overlapped.hEvent = check_handle(synchapi::CreateEventA(
				std::ptr::null_mut(), // security attributes
				0, // manual reset
				0, // initial state
				std::ptr::null(), // name
			))?;

			let ret = check_bool(fileapi::WriteFile(
				self.file.as_raw_handle(),
				buf.as_ptr().cast(),
				len,
				&mut written,
				&mut overlapped,
			));
			match ret {
				// Windows reports timeouts as a succesfull transfer of 0 bytes.
				Ok(_) if written == 0 => return Err(std::io::ErrorKind::TimedOut.into()),
				Ok(_) => return Ok(written as usize),
				Err(ref e) if e.raw_os_error() == Some(winerror::ERROR_IO_PENDING as i32) => (),
				Err(e) => return Err(e),
			}

			wait_async_transfer(&self.file, &mut overlapped)
		}
	}

	pub fn write_vectored(&self, buf: &[IoSlice<'_>]) -> std::io::Result<usize> {
		if buf.is_empty() {
			self.write(&[])
		} else {
			self.write(&buf[0])
		}
	}

	pub fn flush_output(&self) -> std::io::Result<()> {
		unsafe {
			check_bool(winapi::um::fileapi::FlushFileBuffers(self.file.as_raw_handle()))
		}
	}

	pub fn discard_buffers(&self, discard_input: bool, discard_output: bool) -> std::io::Result<()> {
		unsafe {
			let mut flags = 0;
			if discard_input {
				flags |= winbase::PURGE_RXCLEAR;
			}
			if discard_output {
				flags |= winbase::PURGE_TXCLEAR;
			}
			check_bool(commapi::PurgeComm(self.file.as_raw_handle(), flags))
		}
	}


	pub fn set_rts(&self, state: bool) -> std::io::Result<()> {
		if state {
			escape_comm_function(&self.file, winbase::SETRTS)
		} else {
			escape_comm_function(&self.file, winbase::CLRRTS)
		}
	}

	pub fn read_cts(&self) -> std::io::Result<bool> {
		read_pin(&self.file, winbase::MS_CTS_ON)
	}

	pub fn set_dtr(&self, state: bool) -> std::io::Result<()> {
		if state {
			escape_comm_function(&self.file, winbase::SETDTR)
		} else {
			escape_comm_function(&self.file, winbase::CLRDTR)
		}
	}

	pub fn read_dsr(&self) -> std::io::Result<bool> {
		read_pin(&self.file, winbase::MS_DSR_ON)
	}

	pub fn read_ri(&self) -> std::io::Result<bool> {
		read_pin(&self.file, winbase::MS_RING_ON)
	}

	pub fn read_cd(&self) -> std::io::Result<bool> {
		// RLSD or Receive Line Signal Detect is the same as Carrier Detect.
		//
		// I think.
		read_pin(&self.file, winbase::MS_RLSD_ON)
	}
}

fn map_broken_pipe(error: std::io::Error) -> std::io::Result<usize> {
	if error.kind() == std::io::ErrorKind::BrokenPipe {
		Ok(0)
	} else {
		Err(error)
	}
}

fn wait_async_transfer(file: &std::fs::File, overlapped: &mut minwinbase::OVERLAPPED) -> std::io::Result<usize> {
	unsafe {
		let mut transferred = 0;
		let ret = check_bool(ioapiset::GetOverlappedResult(
			file.as_raw_handle(),
			overlapped,
			&mut transferred,
			1,
		));
		match ret {
			// Windows reports timeouts as a succesfull transfer of 0 bytes.
			Ok(_) if transferred == 0 => Err(std::io::ErrorKind::TimedOut.into()),
			Ok(_) => Ok(transferred as usize),
			Err(e) => Err(e),
		}
	}
}

fn escape_comm_function(file: &std::fs::File, function: u32) -> std::io::Result<()> {
	unsafe {
		check_bool(commapi::EscapeCommFunction(file.as_raw_handle(), function))
	}
}

fn read_pin(file: &std::fs::File, pin: u32) -> std::io::Result<bool> {
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

/// Check the return value of a syscall for errors.
fn check_handle(ret: std::os::windows::io::RawHandle) -> std::io::Result<std::os::windows::io::RawHandle> {
	if ret.is_null() {
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

#[derive(Debug)]
struct RegKey {
	key: HKEY,
}

impl RegKey {
	fn open(parent: HKEY, subpath: &std::ffi::CStr, rights: winreg::REGSAM) -> std::io::Result<Self> {
		unsafe {
			let mut key: HKEY = std::ptr::null_mut();
			let status = winreg::RegOpenKeyExA(
				parent,
				subpath.as_ptr(),
				0,
				rights,
				&mut key,
			);
			if status != 0 {
				Err(std::io::Error::from_raw_os_error(status))
			} else {
				Ok(Self { key })
			}
		}
	}

	fn get_value_info(&self) -> std::io::Result<(u32, u32, u32)> {
		unsafe {
			let mut value_count: u32 = 0;
			let mut max_value_name_len: u32 = 0;
			let mut max_value_data_len: u32 = 0;
			let status = winreg::RegQueryInfoKeyA(
				self.key,
				std::ptr::null_mut(),
				std::ptr::null_mut(),
				std::ptr::null_mut(),
				std::ptr::null_mut(),
				std::ptr::null_mut(),
				std::ptr::null_mut(),
				&mut value_count,
				&mut max_value_name_len,
				&mut max_value_data_len,
				std::ptr::null_mut(),
				std::ptr::null_mut(),
			);
			if status != 0 {
				Err(std::io::Error::from_raw_os_error(status))
			} else {
				Ok((value_count, max_value_name_len, max_value_data_len))
			}
		}
	}

	fn get_string_value(&self, index: u32, max_name_len: u32, max_data_len: u32) -> std::io::Result<Option<(Vec<u8>, Vec<u8>)>> {
		unsafe {
			let mut name = vec![0u8; max_name_len as usize + 1];
			let mut data = vec![0u8; max_data_len as usize];
			let mut name_len = name.len() as u32;
			let mut data_len = data.len() as u32;
			let mut kind = 0;
			let status = winreg::RegEnumValueA(
				self.key,
				index,
				name.as_mut_ptr().cast(),
				&mut name_len,
				std::ptr::null_mut(),
				&mut kind,
				data.as_mut_ptr().cast(),
				&mut data_len,
			);
			if status == winerror::ERROR_NO_MORE_ITEMS as i32 {
				Ok(None)
			} else if status != 0 {
				Err(std::io::Error::from_raw_os_error(status))
			} else if kind != winnt::REG_SZ {
				Ok(None)
			} else {
				name.shrink_to(name_len as usize + 1);
				data.shrink_to(data_len as usize);
				Ok(Some((name, data)))
			}
		}
	}
}

impl Drop for RegKey {
	fn drop(&mut self) {
		unsafe {
			winreg::RegCloseKey(self.key);
		}
	}
}

pub fn enumerate() -> std::io::Result<Vec<PathBuf>> {
	let subkey = unsafe { std::ffi::CStr::from_bytes_with_nul_unchecked(b"Hardware\\DEVICEMAP\\SERIALCOMM\x00") };
	let device_map = RegKey::open(winreg::HKEY_LOCAL_MACHINE, subkey, winnt::KEY_READ)?;
	let (value_count, max_value_name_len, max_value_data_len) = device_map.get_value_info()?;

	let mut entries = Vec::with_capacity(16);
	for i in 0.. value_count {
		let name = match device_map.get_string_value(i, max_value_name_len, max_value_data_len) {
			Ok(Some((_name, data))) => data,
			Ok(None) => continue,
			Err(_) => continue,
		};
		if let Ok(name) = String::from_utf8(name) {
			entries.push(name.into());
		}
	}

	Ok(entries)
}
