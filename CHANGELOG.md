# Version 0.1.9 - 2023-05-28
- Disable `fDsrSensitivity` on Windows when configuring flow control.

# Version 0.1.8 - 2023-05-28
- Mention the win32 device namespace in the documentation.
- Implement `Debug` for `Settings`.
- Fix setting of character size, stop bits and parity on Unix platforms.

# Version 0.1.7 - 2022-10-12
- Add `write_all()` function that takes `&self`.

# Version 0.1.6 - 2021-12-09
- Remove fills for libc constants that are no longer needed.

# Version 0.1.5 - 2021-11-20
- Dual-license under BSD-2-Clause and Apache-2.0.

# Version 0.1.4 - 2021-11-19
- Detect `/dev/ttyU*` and `/dev/cuaU*` devices when listing serial ports on FreeBSD and DragonFlyBSD.

# Version 0.1.3 - 2021-11-13
- Handle non-existing `HKLM\Hardware\DEVICEMAP\SERIAL` registry key for port enumeration on Windows.

# Version 0.1.2 - 2021-11-13
- Fix link to documentation in `Cargo.toml`.

# Version 0.1.1 - 2021-11-13
- Fix leaking event objects on Windows.

# Version 0.1.0 - 2021-11-12
- Add non-trait `is_read_vectored()` and `is_write_vectored()` functions.
- Improve documentation.

# Version 0.1.0-alpha5 - 2021-11-11
- Implement port enumeration for BSD and Apple platforms.
- Implement port enumeration for Illumos and Solaris.

# Version 0.1.0-alpha4 - 2021-11-09
- Fix buffer truncation when enumerating ports on Windows.
- Switch to overlapped IO on windows to allow concurrent reads and writes.

# Version 0.1.0-alpha3 - 2021-11-08
- Implement port enumeration for Windows.
- Fix comparing `termios` struct on Linux, when using `BOTHER` with a standard baud rate.
- Always set the TTY to raw mode on Unix.
- Add versions of `read()`, `write()` and friends that take a const `&self`.
- Make `KeepSettings` public as intended.
- Make more functions take `&self` instead of `&mut self`.

# Version 0.1.0-alpha2 - 2021-11-07
- Add `SerialPort::available_ports()`, which for now only works on Linux.

# Version 0.1.0-alpha1 - 2021-11-07
- First alpha release.
