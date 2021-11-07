pub use libc::TIOCMBIS;
pub use libc::TIOCMBIC;
pub use libc::TIOCMGET;
pub use libc::TIOCM_RTS;
pub use libc::TIOCM_CTS;
pub use libc::TIOCM_DTR;
pub use libc::TIOCM_DSR;
pub use libc::TIOCM_RI;
pub use libc::TIOCM_CD;

pub const BAUD_RATES: [(u32, u32); 18] = [
	// POSIX 2017.1: https://pubs.opengroup.org/onlinepubs/9699919799
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
	// Not POSIX anymore, but we realllly want these.
	// Please file an issue if these don't exist for your platform.
	(libc::B57600, 57600),
	(libc::B115200, 115200),
	(libc::B230400, 230400),
];
