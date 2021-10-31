/// Settings for a serial port.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SerialSettings {
	/// The baud rate.
	///
	/// You can use one of the `BAUD_*` constants for portable baud rates.
	/// Alternatively, you can try to set a custom baud rate.
	/// It is platform and device dependent if custom baud-rates are supported.
	pub baud_rate: u32,

	/// The number of data bits per character.
	pub char_size: CharSize,

	/// The number of stop bits per character.
	pub stop_bits: StopBits,

	/// The parity check per character.
	pub parity: Parity,

	/// The type of flow control for the serial port.
	pub flow_control: FlowControl,
}

/// A baud rate of 110.
pub const BAUD_110: u32 = 110;

/// A baud rate of 300.
pub const BAUD_300: u32 = 300;

/// A baud rate of 600.
pub const BAUD_600: u32 = 600;

/// A baud rate of 1200.
pub const BAUD_1200: u32 = 1200;

/// A baud rate of 2400.
pub const BAUD_2400: u32 = 2400;

/// A baud rate of 4800.
pub const BAUD_4800: u32 = 4800;

/// A baud rate of 9600.
pub const BAUD_9600: u32 = 9600;

/// A baud rate of 129200.
pub const BAUD_19200: u32 = 19200;

/// A baud rate of 38400.
pub const BAUD_38400: u32 = 38400;

/// A baud rate of 57600.
pub const BAUD_57600: u32 = 57600;

/// A baud rate of 115200.
pub const BAUD_115200: u32 = 115200;

/// The number of bits per character for a serial port.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum CharSize {
	/// Characters of 5 bits.
	Bits5,

	/// Characters of 6 bits.
	Bits6,

	/// Characters of 7 bits.
	Bits7,

	/// Characters of 8 bits.
	Bits8,
}

/// The number of stop bits per character for a serial port.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum StopBits {
	/// One stop bit.
	One,

	/// Two stop bit.
	Two,
}

/// The type of parity check for a serial port.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Parity {
	/// Do not add a parity bit and do not check for parity.
	None,

	/// Add a bit to ensure odd parity of all characters send over the serial port.
	///
	/// Received characters are also expected to have a parity bit and odd parity.
	/// What happens with received characters that have invalid parity is platform and device specific.
	Odd,

	/// Add a bit to ensure even parity of all characters send over the serial port.
	///
	/// Received characters are also expected to have a parity bit and even parity.
	/// What happens with received characters that have invalid parity is platform and device specific.
	Even,
}

/// The type of flow control for a serial port.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum FlowControl {
	/// Do not perform any automatic flow control.
	None,

	/// Perform XON/XOFF flow control.
	///
	/// This is also sometimes referred to as "software flow control".
	XonXoff,

	/// Perform RTS/CTS flow control.
	///
	/// This is also sometimes referred to as "hardware flow control".
	RtsCts,
}
