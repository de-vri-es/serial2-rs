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
//! * Full access to platform specific serial port settings using target specific feature flags (`"unix"` or `"windows"`).
//!
//! You can open and configure a serial port in one go with [`SerialPort::open()`].
//! The second argument to `open()` must be a type that implements [`IntoSettings`].
//! In the simplest case, it is enough to pass a `u32` for the baud rate.
//! Doing that will also configure a character size of 8 bits with 1 stop bit and disables parity checks and flow control.
//! For full control over the applied settings, pass a closure that receives the current [`Settings`] and return the desired settings.
//! If you do, you will almost always want to call [`Settings::set_raw()`] before changing any other settings.
//!
//! The standard [`std::io::Read`] and [`std::io::Write`] traits are implemented for [`SerialPort`] and  [`&SerialPort`][`SerialPort`].
//! This allows you to use the serial port concurrently from multiple threads through a non-mutable reference.
//!
//! There are also non-trait [`read()`][SerialPort::read()] and [`write()`][SerialPort::write()] functions,
//! so you can use the serial port without importing any traits.
//! These take `&self`, so they can also be used from multiple threads concurrently.
//!
//! The [`SerialPort::available_ports()`] function can be used to get a list of available serial ports on supported platforms.
//!
//! # Closing a port
//!
//! A `SerialPort` owns an open OS handle. Leaving its scope or calling `drop(port)`
//! closes that handle. Every handle created by `try_clone()` must also be dropped.
//! Dropping an `Arc<SerialPort>` only drops the port when the last strong owner is gone;
//! it does not disconnect other owners or cancel their I/O.
//!
//! For a connection that can be opened and closed, store an `Option<SerialPort>`.
//! With shared state such as `Arc<Mutex<Option<SerialPort>>>`, take the port out of
//! that state, release the mutex, and then drop the port:
//!
//! ```no_run
//! use serial2::SerialPort;
//! use std::sync::{Arc, Mutex};
//!
//! # fn example() -> std::io::Result<()> {
//! let connection = Arc::new(Mutex::new(Some(
//!     SerialPort::open("/dev/ttyUSB0", 115200)?,
//! )));
//! let other_owner = Arc::clone(&connection);
//!
//! let port = {
//!     let mut guard = connection.lock().unwrap();
//!     guard.take()
//! }; // Release the mutex before closing the handle.
//! drop(port);
//!
//! // Other owners now see that the connection is closed.
//! assert!(other_owner.lock().unwrap().is_none());
//! # Ok(())
//! # }
//! ```
//!
//! For a reader running in another thread, set a finite read timeout before starting
//! it, signal it to stop, and have it check that signal between reads (including
//! after timeouts). Join the thread and drop every port handle before reopening.
//! Do not hold a mutex across a blocking read if shutdown needs that same mutex,
//! or while joining a thread that needs to lock it.
//!
//! `flush()` waits for queued output to be transmitted and is not bounded by the
//! write timeout. It can block indefinitely if flow control stops transmission or
//! the device/driver stops making progress. Dropping a port does not call `flush()`.
//! If losing pending output is acceptable, `discard_output_buffer()` discards it
//! instead of waiting for transmission; this does not guarantee that a device
//! or driver will complete its own close operation promptly.
//!
//! On Linux, trace a hang with `strace -f -tt -T -o serial.trace ./your-program`.
//! A thread waiting in `futex` suggests synchronization to inspect; a wait in
//! `poll`, a drain/configuration `ioctl`, or `close` points to a different part of
//! the shutdown path. Include the trace, device driver, kernel and library versions
//! when reporting a hang, especially on embedded systems such as Yocto.
//!
//! # Example
//! This example opens a serial port and echoes back everything that is read.
//!
//! ```no_run
//! # fn example() -> std::io::Result<()> {
//! use serial2::SerialPort;
//!
//! // On Windows, use something like "COM1" or "COM15".
//! let port = SerialPort::open("/dev/ttyUSB0", 115200)?;
//! let mut buffer = [0; 256];
//! loop {
//!     let read = port.read(&mut buffer)?;
//!     port.write_all(&buffer[..read])?;
//! }
//! # }
//! ```

#![cfg_attr(feature = "doc-cfg", feature(doc_cfg))]

#![warn(missing_docs)]
#![warn(private_interfaces)]
#![warn(private_bounds)]

mod sys;

mod into_settings;
pub use into_settings::{IntoSettings, KeepSettings};

mod serial_port;
pub use serial_port::SerialPort;

mod settings;
pub use settings::{CharSize, FlowControl, Parity, Settings, StopBits, TryFromError, COMMON_BAUD_RATES};

pub mod os;

#[cfg(any(feature = "doc", feature = "rs4xx"))]
#[cfg_attr(feature = "doc-cfg", doc(cfg(feature = "rs4xx")))]
pub mod rs4xx;
