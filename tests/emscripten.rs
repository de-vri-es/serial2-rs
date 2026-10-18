#![cfg(target_os = "emscripten")]

#[test]
fn set_break_is_unsupported() -> std::io::Result<()> {
	use std::io::{Read, Seek, SeekFrom, Write};
	use std::os::fd::OwnedFd;

	let nonce = std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.unwrap()
		.as_nanos();
	let path = std::env::temp_dir().join(format!("serial2-set-break-{}-{nonce}", std::process::id()));
	let file = std::fs::OpenOptions::new()
		.read(true)
		.write(true)
		.create_new(true)
		.open(&path)?;
	let port = serial2::SerialPort::from(OwnedFd::from(file));
	let result = (|| {
		for enable in [true, false] {
			let error = port.set_break(enable).unwrap_err();
			assert_eq!(error.kind(), std::io::ErrorKind::Unsupported);
			assert_eq!(error.raw_os_error(), None);
		}
		let mut file = std::fs::File::from(OwnedFd::from(port));
		file.write_all(b"still open")?;
		file.seek(SeekFrom::Start(0))?;
		let mut contents = String::new();
		file.read_to_string(&mut contents)?;
		assert_eq!(contents, "still open");
		Ok(())
	})();
	std::fs::remove_file(path)?;
	result
}
