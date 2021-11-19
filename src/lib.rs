//! Serial port communication for Rust.
//!
//! The `serial2` crate provides a cross-platform interface to serial ports.
//! It aims to provide a simpler interface than other alternatives.
//!
//! Currently supported features:
//! * Simple interface: one [`SerialPort`] struct for all supported platforms.
//! * List available ports.
//! * Custom baud rates on all supported platforms except Solaris and Illumos.
//! * Concurrent reads and writes from multiple threads, even on Windows.
//! * Purge the OS buffers (useful to discard read noise when the line should have been silent, for example).
//! * Read and control individual modem status lines to use them as general purpose I/O.
//! * Cross platform configuration of serial port settings:
//!   * Baud rate
//!   * Character size
//!   * Stop bits
//!   * Parity checks
//!   * Flow control
//!   * Read/write timeouts
//!
//! You can open and configure a serial port in one go with [`SerialPort::open()`].
//! The second argument to `open()` must be a type that implements [`IntoSettings`].
//! In the simplest case, it is enough to pass a `u32` for the baud rate.
//! Doing that will also configure a character size of 8 bits with 1 stop bit and disables parity checks and flow control.
//! For full control over the applied settings, pass a closure that receives the the current [`Settings`] and return the desired settings.
//!
//! The [`SerialPort`] struct implements the standard [`std::io::Read`] and [`std::io::Write`] traits,
//! as well as [`read()`][SerialPort::read()] and [`write()`][SerialPort::write()] functions that take `&self` instead of `&mut self`.
//! This allows you to use the serial port concurrently from multiple threads.
//!
//! The [`SerialPort::available_ports()`] function can be used to get a list of available serial ports on supported platforms.
//!
//! # Example
//! This example opens a serial port and echoes back everything that is read.
//!
//! ```no_run
//! # fn example() -> std::io::Result<()> {
//! use serial2::SerialPort;
//!
//! // On Windows, use something like "COM1".
//! let port = SerialPort::open("/dev/ttyUSB0", 115200)?;
//! let mut buffer = [0; 256];
//! loop {
//!     let read = port.read(&mut buffer)?;
//!     port.write(&buffer[..read])?;
//! }
//! # }
//! ```

#![warn(missing_docs)]

mod sys;

mod into_settings;
pub use into_settings::{IntoSettings, KeepSettings};

mod serial_port;
pub use serial_port::SerialPort;

mod settings;
pub use settings::{CharSize, FlowControl, Parity, Settings, StopBits, COMMON_BAUD_RATES};
