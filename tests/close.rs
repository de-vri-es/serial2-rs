#![cfg(target_os = "linux")]

use serial2::SerialPort;
use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[test]
fn close_releases_the_terminal() -> io::Result<()> {
	let (a, mut b) = SerialPort::pair()?;
	b.set_read_timeout(Duration::from_secs(1))?;
	a.close()?;
	// Closing the last handle for a produces EOF on b.
	assert_eq!(b.read(&mut [0])?, 0);
	Ok(())
}

#[test]
fn close_preserves_other_handles() -> io::Result<()> {
	let (a, mut b) = SerialPort::pair()?;
	b.set_read_timeout(Duration::from_secs(1))?;
	let clone = a.try_clone()?;
	a.close()?;

	clone.write_all(b"still open")?;
	let mut buffer = [0; 10];
	b.read_exact(&mut buffer)?;
	assert_eq!(&buffer, b"still open");

	clone.close()?;
	assert_eq!(b.read(&mut [0])?, 0);
	Ok(())
}

#[test]
fn reopen_after_close() -> io::Result<()> {
	use std::ffi::{CStr, OsStr};
	use std::os::unix::{ffi::OsStrExt, io::AsRawFd};

	let (mut a, b) = SerialPort::pair()?;
	a.set_read_timeout(Duration::from_secs(1))?;
	let mut name = [0; 256];
	// SAFETY: b owns a valid descriptor and name is a writable buffer.
	let result = unsafe { libc::ttyname_r(b.as_raw_fd(), name.as_mut_ptr(), name.len()) };
	if result != 0 {
		return Err(io::Error::from_raw_os_error(result));
	}
	// ttyname_r writes a NUL-terminated name on success.
	let name = unsafe { CStr::from_ptr(name.as_ptr()) };
	let path = OsStr::from_bytes(name.to_bytes());
	b.close()?;

	for _ in 0..16 {
		let port = SerialPort::open(path, 115200)?;
		port.write_all(b"reopened")?;
		let mut buffer = [0; 8];
		a.read_exact(&mut buffer)?;
		assert_eq!(&buffer, b"reopened");
		port.close()?;
	}
	a.close()
}

#[test]
fn close_shared_connection_while_other_state_owners_remain() -> io::Result<()> {
	let (a, mut b) = SerialPort::pair()?;
	b.set_read_timeout(Duration::from_secs(1))?;
	let connection = Arc::new(Mutex::new(Some(a)));
	let other_owner = Arc::clone(&connection);
	let port = {
		let mut guard = connection.lock().unwrap();
		guard.take()
	};

	// The state remains shared, but shutdown does not hold the mutex.
	assert!(other_owner.try_lock().unwrap().is_none());
	port.unwrap().close()?;
	assert_eq!(b.read(&mut [0])?, 0);
	Ok(())
}

#[test]
fn stop_reader_before_closing_connection() -> io::Result<()> {
	use std::sync::atomic::{AtomicBool, Ordering};
	use std::sync::mpsc;

	let (a, mut b) = SerialPort::pair()?;
	b.set_read_timeout(Duration::from_millis(20))?;
	let stop = Arc::new(AtomicBool::new(false));
	let reader_stop = Arc::clone(&stop);
	let (ready_tx, ready_rx) = mpsc::channel();
	let (done_tx, done_rx) = mpsc::channel();
	let thread = std::thread::spawn(move || {
		ready_tx.send(()).unwrap();
		let result = (|| {
			// Always attempt one read so the idle read/timeout path is exercised.
			loop {
				match b.read(&mut [0]) {
					Ok(0) => break,
					Ok(_) => (),
					Err(e) if e.kind() == io::ErrorKind::TimedOut => (),
					Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
					Err(e) => return Err(e),
				}
				if reader_stop.load(Ordering::Acquire) {
					break;
				}
			}
			b.close()
		})();
		done_tx.send(result).unwrap();
	});

	ready_rx.recv_timeout(Duration::from_secs(5)).unwrap();
	stop.store(true, Ordering::Release);
	// A broken timeout/shutdown path fails instead of hanging the test on join.
	done_rx.recv_timeout(Duration::from_secs(5)).unwrap()?;
	thread.join().unwrap();
	a.close()
}
