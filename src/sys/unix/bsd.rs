use std::path::PathBuf;

pub use libc::TIOCMBIS;
pub use libc::TIOCMBIC;
pub use libc::TIOCMGET;
pub use libc::TIOCM_RTS;
pub use libc::TIOCM_CTS;
pub use libc::TIOCM_DTR;
pub use libc::TIOCM_DSR;
pub use libc::TIOCM_RI;
pub use libc::TIOCM_CD;

pub fn enumerate() -> std::io::Result<Vec<PathBuf>> {
	Err(std::io::Error::new(std::io::ErrorKind::Other, "port enumeration is not implemented for this platform"))
}
