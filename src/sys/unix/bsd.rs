use cfg_if::cfg_if;
use std::path::PathBuf;

pub use libc::TIOCMBIC;
pub use libc::TIOCMBIS;
pub use libc::TIOCMGET;
pub use libc::TIOCM_CD;
pub use libc::TIOCM_CTS;
pub use libc::TIOCM_DSR;
pub use libc::TIOCM_DTR;
pub use libc::TIOCM_RI;
pub use libc::TIOCM_RTS;

pub fn enumerate() -> std::io::Result<Vec<PathBuf>> {
	use std::os::unix::ffi::OsStrExt;
	use std::os::unix::fs::FileTypeExt;

	let serial_ports = std::fs::read_dir("/dev")?
		.into_iter()
		.filter_map(|entry| {
			let entry = entry.ok()?;
			let kind = entry.metadata().ok()?.file_type();
			if kind.is_char_device() && is_tty_name(entry.file_name().as_bytes()) {
				Some(entry.path())
			} else {
				None
			}
		})
		.collect();
	Ok(serial_ports)
}

fn is_tty_name(name: &[u8]) -> bool {
	cfg_if! {
		if #[cfg(any(target_os = "ios", target_os = "macos"))] {
			// Sigh, closed source doesn't have to mean undocumented.
			// Anyway:
			// https://stackoverflow.com/questions/14074413/serial-port-names-on-mac-os-x
			// https://learn.adafruit.com/ftdi-friend/com-slash-serial-port-name
			name.starts_with(b"tty.") || name.starts_with(b"cu.")

		} else {
			// For BSD variants, we simply report all entries in /dev that look like a TTY.
			// This may contain a lot of false positives for pseudo-terminals or other fake terminals.
			// If anyone can improve this for a specific BSD they love, by all means send a PR.

			// https://www.dragonflybsd.org/docs/docs/newhandbook/serial_communications/
			#[cfg(target_os = "dragonfly")]
			const PREFIXES: [&[u8]; 2] = [b"ttyd", b"cuaa"];

			// https://www.freebsd.org/cgi/man.cgi?query=uart&sektion=4&apropos=0&manpath=FreeBSD+13.0-RELEASE+and+Ports
			// https://www.freebsd.org/cgi/man.cgi?query=ucom&sektion=4&apropos=0&manpath=FreeBSD+13.0-RELEASE+and+Ports
			#[cfg(target_os = "freebsd")]
			const PREFIXES: [&[u8]; 5] = [b"ttyu", b"ttyU", b"cuau", b"cuaU", b"cuad"];

			// https://man.netbsd.org/com.4
			// https://man.netbsd.org/ucom.4
			#[cfg(target_os = "netbsd")]
			const PREFIXES: [&[u8]; 4] = [b"tty", b"dty", b"ttyU", b"dtyU"];

			// https://man.openbsd.org/com
			// https://man.openbsd.org/ucom
			#[cfg(target_os = "openbsd")]
			const PREFIXES: [&[u8]; 4] = [b"tty", b"cua", b"ttyU", b"cuaU"];

			for prefix in PREFIXES {
				if let Some(suffix) = name.strip_prefix(prefix) {
					if !suffix.is_empty() && suffix.iter().all(|c| c.is_ascii_digit()) {
						return true;
					}
				}
			}

			false
		}
	}
}
