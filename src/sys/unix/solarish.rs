use std::path::PathBuf;

// All values taken from:
// https://github.com/illumos/illumos-gate/blob/252adeb303174e992b64771bf9639e63a4d55418/usr/src/uts/common/sys/termios.h

pub use libc::TIOCMBIS;
pub use libc::TIOCMBIC;
pub use libc::TIOCMGET;
//pub const TIOCM_LE: i32 = 0o0001;
pub const TIOCM_DTR: i32 = 0o0002;
pub const TIOCM_RTS: i32 = 0o0004;
//pub const TIOCM_ST: i32 = 0o0010;
//pub const TIOCM_SR: i32 = 0020;
pub const TIOCM_CTS: i32 = 0o0040;
pub const TIOCM_CAR: i32 = 0o0100;
pub const TIOCM_CD: i32 = TIOCM_CAR;
pub const TIOCM_RNG: i32 = 0o0200;
pub const TIOCM_RI: i32 = TIOCM_RNG;
pub const TIOCM_DSR: i32 = 0o0400;

pub const BAUD_RATES: [(u32, u32); 23] = [
	(libc::B50, 50),
	(libc::B75, 75),
	(libc::B110, 110),
	(libc::B134, 134),
	(libc::B150, 150),
	(libc::B200, 200),
	(libc::B300, 300),
	(libc::B600, 600),
	(libc::B1200, 1200),
	(libc::B1800, 1800),
	(libc::B2400, 2400),
	(libc::B4800, 4800),
	(libc::B9600, 9600),
	(libc::B19200, 19200),
	(libc::B38400, 38400),
	(libc::B57600, 57600),
	(libc::B76800, 76800),
	(libc::B115200, 115200),
	(libc::B153600, 153600),
	(libc::B230400, 230400),
	(libc::B307200, 307200),
	(libc::B460800, 460800),
	(libc::B921600, 921600),
];

pub fn enumerate() -> std::io::Result<Vec<PathBuf>> {
	Err(std::io::Error::new(std::io::ErrorKind::Other, "port enumeration is not implemented for this platform"))
}
