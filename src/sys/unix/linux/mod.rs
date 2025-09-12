use std::path::{Path, PathBuf};

#[cfg(feature = "rs4xx")]
mod rs4xx;

#[cfg(feature = "rs4xx")]
pub use rs4xx::*;

pub fn enumerate() -> std::io::Result<Vec<PathBuf>> {
	use std::os::unix::ffi::OsStrExt;
	use std::os::unix::fs::FileTypeExt;

	let dir = std::fs::read_dir("/sys/class/tty")?;
	let mut entries = Vec::with_capacity(32);

	for entry in dir {
		// Skip entries we can't stat.
		let entry = match entry {
			Ok(x) => x,
			Err(_) => continue,
		};

		let name = entry.file_name();

		// Skip everything that doesn't have a matching device node in /dev
		let dev_path = Path::new("/dev").join(&name);
		match dev_path.metadata() {
			Err(_) => continue,
			Ok(metadata) => {
				if !metadata.file_type().is_char_device() {
					continue;
				}
			},
		}

		match name.as_bytes().strip_prefix(b"tty") {
			// Skip entries called "tty";
			Some(b"") => continue,
			// Skip "tty1", "tty2", etc (they are virtual terminals, not serial ports).
			Some(&[c, ..]) if c.is_ascii_digit() => continue,
			// Skip everything that doesn't start with "tty", they are almost certainly not serial ports.
			None => continue,
			// Accept the rest.
			Some(_) => (),
		};

		// There's a bunch of ttyS* ports that are not really serial ports.
		//
		// They have a file called `device/driver_override` set to "(null)".
		if let Ok(driver_override) = std::fs::read(entry.path().join("device/driver_override")) {
			if driver_override == b"(null)\n" {
				continue;
			}
		}

		entries.push(dev_path);
	}

	Ok(entries)
}
