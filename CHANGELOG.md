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
