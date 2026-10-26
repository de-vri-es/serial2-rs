#![cfg(unix)]

use assert2::assert;
use serial2::SerialPort;
use std::time::Duration;

#[test]
fn close_releases_the_terminal() {
	assert!(let Ok((a, mut b)) = SerialPort::pair());
	assert!(let Ok(()) = b.set_read_timeout(Duration::from_millis(100)));
	assert!(let Ok(()) = a.close());
	// Closing the last handle for a produces end-of-file for reads on b.
	assert!(let Ok(0) = b.read(&mut [0]));
}

#[test]
fn close_preserves_other_handles() {
	assert!(let Ok((a, mut b)) = SerialPort::pair());
	assert!(let Ok(()) = b.set_read_timeout(Duration::from_millis(100)));
	assert!(let Ok(clone) = a.try_clone());
	assert!(let Ok(()) = a.close());

	assert!(let Ok(()) = clone.write_all(b"still open"));
	let mut buffer = [0; 10];
	assert!(let Ok(()) = b.read_exact(&mut buffer));
	assert_eq!(&buffer, b"still open");

	assert!(let Ok(()) = clone.close());
	assert!(let Ok(0) = b.read(&mut [0]));
}
