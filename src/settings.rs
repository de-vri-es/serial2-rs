use std::fmt::{Display, Formatter};

/// The settings of a serial port.
#[derive(Clone)]
pub struct Settings {
	pub(crate) inner: crate::sys::Settings,
}

/// Common baud rates used by many applications and devices.
///
/// Note that Linux, *BSD, Windows and Apple platforms all support custom baud rates, so you are not limited to these values.
/// It is also not guaranteed that all devices support these speeds.
///
/// These speeds can be useful to populate a user interface with some common options though.
pub const COMMON_BAUD_RATES: &[u32] = &[
	4800, 9600, 19200, 38400, 57600, 115200, 230400, 460800, 500000, 576000, 921600, 1000000, 1500000, 2000000,
];

/// The number of bits per character for a serial port.
///
/// <div>
/// <span class="stab portability" style="display: inline">Available on <strong>crate feature <code>serde</code></strong> only:</span>
/// This type supports (de)serialization as a number.
/// </div>
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u8)]
pub enum CharSize {
	/// Characters of 5 bits.
	///
	/// <div class="item-info" style="margin-left: 0">
	/// <span class="stab portability" style="display: inline">Available on <strong>crate feature <code>serde</code></strong> only:</span>
	/// This variant is (de)serialized as the number <code>5</code>.
	/// </div>
	Bits5 = 5,

	/// Characters of 6 bits.
	///
	/// <div class="item-info" style="margin-left: 0">
	/// <span class="stab portability" style="display: inline">Available on <strong>crate feature <code>serde</code></strong> only:</span>
	/// This variant is (de)serialized as the number <code>7</code>.
	/// </div>
	Bits6 = 6,

	/// Characters of 7 bits.
	///
	/// <div class="item-info" style="margin-left: 0">
	/// <span class="stab portability" style="display: inline">Available on <strong>crate feature <code>serde</code></strong> only:</span>
	/// This variant is (de)serialized as the number <code>7</code>.
	/// </div>
	Bits7 = 7,

	/// Characters of 8 bits.
	///
	/// <div class="item-info" style="margin-left: 0">
	/// <span class="stab portability" style="display: inline">Available on <strong>crate feature <code>serde</code></strong> only:</span>
	/// This variant is (de)serialized as the number <code>8</code>.
	/// </div>
	Bits8 = 8,
}

impl Display for CharSize {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			CharSize::Bits5 => write!(f, "Five"),
			CharSize::Bits6 => write!(f, "Six"),
			CharSize::Bits7 => write!(f, "Seven"),
			CharSize::Bits8 => write!(f, "Eight"),
		}
	}
}

/// The number of stop bits per character for a serial port.
///
/// <div>
/// <span class="stab portability" style="display: inline">Available on <strong>crate feature <code>serde</code></strong> only:</span>
/// This type supports (de)serialization as a number.
/// </div>
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u8)]
pub enum StopBits {
	/// One stop bit.
	///
	/// <div class="item-info" style="margin-left: 0">
	/// <span class="stab portability" style="display: inline">Available on <strong>crate feature <code>serde</code></strong> only:</span>
	/// This variant is (de)serialized as the number <code>1</code>.
	/// </div>
	One = 1,

	/// Two stop bit.
	///
	/// <div class="item-info" style="margin-left: 0">
	/// <span class="stab portability" style="display: inline">Available on <strong>crate feature <code>serde</code></strong> only:</span>
	/// This variant is (de)serialized as the number <code>2</code>.
	/// </div>
	Two = 2,
}

impl Display for StopBits {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			StopBits::One => write!(f, "One"),
			StopBits::Two => write!(f, "Two"),
		}
	}
}

/// The type of parity check for a serial port.
///
/// <div>
/// <span class="stab portability" style="display: inline">Available on <strong>crate feature <code>serde</code></strong> only:</span>
/// This type supports (de)serialization as a string.
/// </div>
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Parity {
	/// Do not add a parity bit and do not check for parity.
	///
	/// <div class="item-info" style="margin-left: 0">
	/// <span class="stab portability" style="display: inline">Available on <strong>crate feature <code>serde</code></strong> only:</span>
	/// This variant is (de)serialized as the string <code>"none"</code>.
	/// </div>
	None,

	/// Add a bit to ensure odd parity of all characters send over the serial port.
	///
	/// Received characters are also expected to have a parity bit and odd parity.
	/// What happens with received characters that have invalid parity is platform and device specific.
	///
	///
	/// <div class="item-info" style="margin-left: 0">
	/// <span class="stab portability" style="display: inline">Available on <strong>crate feature <code>serde</code></strong> only:</span>
	/// This variant is (de)serialized as the string <code>"odd"</code>.
	/// </div>
	Odd,

	/// Add a bit to ensure even parity of all characters send over the serial port.
	///
	/// Received characters are also expected to have a parity bit and even parity.
	/// What happens with received characters that have invalid parity is platform and device specific.
	///
	///
	/// <div class="item-info" style="margin-left: 0">
	/// <span class="stab portability" style="display: inline">Available on <strong>crate feature <code>serde</code></strong> only:</span>
	/// This variant is (de)serialized as the string <code>"even"</code>.
	/// </div>
	Even,
}

impl Display for Parity {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Parity::None => write!(f, "None"),
			Parity::Odd => write!(f, "Odd"),
			Parity::Even => write!(f, "Even"),
		}
	}
}

/// The type of flow control for a serial port.
///
/// <div>
/// <span class="stab portability" style="display: inline">Available on <strong>crate feature <code>serde</code></strong> only:</span>
/// This type supports (de)serialization as a string.
/// </div>
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum FlowControl {
	/// Do not perform any automatic flow control.
	///
	/// <div class="item-info" style="margin-left: 0">
	/// <span class="stab portability" style="display: inline">Available on <strong>crate feature <code>serde</code></strong> only:</span>
	/// This variant is (de)serialized as the string <code>"none"</code>.
	/// </div>
	None,

	/// Perform XON/XOFF flow control.
	///
	/// This is also sometimes referred to as "software flow control".
	///
	/// <div class="item-info" style="margin-left: 0">
	/// <span class="stab portability" style="display: inline">Available on <strong>crate feature <code>serde</code></strong> only:</span>
	/// This variant is (de)serialized as the string <code>"xon/xoff"</code>.
	/// </div>
	XonXoff,

	/// Perform RTS/CTS flow control.
	///
	/// This is also sometimes referred to as "hardware flow control".
	///
	/// <div class="item-info" style="margin-left: 0">
	/// <span class="stab portability" style="display: inline">Available on <strong>crate feature <code>serde</code></strong> only:</span>
	/// This variant is (de)serialized as the string <code>"rts/cts"</code>.
	/// </div>
	RtsCts,
}

impl Display for FlowControl {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			FlowControl::None => write!(f, "None"),
			FlowControl::XonXoff => write!(f, "Software"),
			FlowControl::RtsCts => write!(f, "Hardware"),
		}
	}
}

impl Settings {
	/// Disable all OS level input and output processing.
	///
	/// All input and output processing will be disabled,
	/// and the configuration will be set for 8 bit binary communication,
	/// one stop bit, no parity checks and no flow control.
	///
	/// This is usually a good starting point for manual configuration.
	pub fn set_raw(&mut self) {
		self.inner.set_raw();
	}

	/// Set the baud rate to be configured.
	///
	/// This function returns an error if the platform does not support the requested band-width.
	/// Note that the device itself may also not support the requested baud rate, even if the platform does.
	/// In that case [`SerialPort::set_configuration()`][crate::SerialPort::set_configuration] will return an error.
	pub fn set_baud_rate(&mut self, baud_rate: u32) -> std::io::Result<()> {
		self.inner.set_baud_rate(baud_rate)
	}

	/// Get the baud rate from the configuration.
	pub fn get_baud_rate(&self) -> std::io::Result<u32> {
		self.inner.get_baud_rate()
	}

	/// Set the number of bits in a character.
	pub fn set_char_size(&mut self, char_size: CharSize) {
		self.inner.set_char_size(char_size)
	}

	/// Get the number of bits in a character.
	pub fn get_char_size(&self) -> std::io::Result<CharSize> {
		self.inner.get_char_size()
	}

	/// Set the number of stop bits following each character.
	pub fn set_stop_bits(&mut self, stop_bits: StopBits) {
		self.inner.set_stop_bits(stop_bits)
	}

	/// Get the number of stop bits following each character.
	pub fn get_stop_bits(&self) -> std::io::Result<StopBits> {
		self.inner.get_stop_bits()
	}

	/// Set the partity check.
	pub fn set_parity(&mut self, parity: Parity) {
		self.inner.set_parity(parity)
	}

	/// Get the partity check.
	pub fn get_parity(&self) -> std::io::Result<Parity> {
		self.inner.get_parity()
	}

	/// Set the flow control mechanism.
	///
	/// See the individual documentation of the [`FlowControl`] variants for more information.
	pub fn set_flow_control(&mut self, flow_control: FlowControl) {
		self.inner.set_flow_control(flow_control)
	}

	/// Get the flow control mechanism
	pub fn get_flow_control(&self) -> std::io::Result<FlowControl> {
		self.inner.get_flow_control()
	}

	/// Get a reference to the raw `termios` struct.
	///
	/// On Linux and Android this is actually a `termios2` struct.
	/// On other Unix platforms, this is a `termios` struct.
	///
	/// You can use this function to access Unix specific features of the serial port.
	/// You code will not be cross platform anymore if you use this.
	#[cfg(any(doc, all(unix, feature = "unix")))]
	#[cfg_attr(feature = "doc-cfg", doc(cfg(feature = "unix")))]
	pub fn as_termios(&self) -> &crate::os::unix::RawTermios {
		&self.inner.termios
	}

	/// Get a mutable reference to the raw `termios` struct.
	///
	/// On Linux and Android this is actually a `termios2` struct.
	/// On other Unix platforms, this is a `termios` struct.
	///
	/// You can use this function to access Unix specific features of the serial port.
	/// You code will not be cross platform anymore if you use this.
	#[cfg(any(doc, all(unix, feature = "unix")))]
	#[cfg_attr(feature = "doc-cfg", doc(cfg(feature = "unix")))]
	pub fn as_termios_mut(&mut self) -> &mut crate::os::unix::RawTermios {
		&mut self.inner.termios
	}

	/// Get a reference to the raw `DCB` struct.
	///
	/// You can use this function to access Windows specific features of the serial port.
	/// You code will not be cross platform anymore if you use this.
	#[cfg(any(doc, all(windows, feature = "windows")))]
	#[cfg_attr(feature = "doc-cfg", doc(cfg(feature = "windows")))]
	pub fn as_raw_dbc(&self) -> &crate::os::windows::DCB {
		&self.inner.dcb
	}

	/// Get a mutable reference to the raw  `DCB` struct.
	///
	/// You can use this function to access Windows specific features of the serial port.
	/// You code will not be cross platform anymore if you use this.
	#[cfg(any(doc, all(windows, feature = "windows")))]
	#[cfg_attr(feature = "doc-cfg", doc(cfg(feature = "windows")))]
	pub fn as_raw_dbc_mut(&mut self) -> &mut crate::os::windows::DCB {
		&mut self.inner.dcb
	}
}

impl std::fmt::Debug for Settings {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Settings")
			.field("baud_rate", &self.get_baud_rate())
			.field("char_size", &self.get_char_size())
			.field("stop_bits", &self.get_stop_bits())
			.field("parity", &self.get_parity())
			.field("flow_control", &self.get_flow_control())
			.finish()
	}
}

#[cfg(feature = "serde")]
impl serde::Serialize for CharSize {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_u8(*self as u8)
	}
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for CharSize {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		struct Visitor;
		impl<'de> serde::de::Visitor<'de> for Visitor {
			type Value = CharSize;
			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("the number 5, 6, 7 or 8")
			}

			fn visit_u64<E: serde::de::Error>(self, data: u64) -> Result<Self::Value, E> {
				match data {
					5 => Ok(CharSize::Bits5),
					6 => Ok(CharSize::Bits6),
					7 => Ok(CharSize::Bits7),
					8 => Ok(CharSize::Bits8),
					x => Err(E::invalid_value(
						serde::de::Unexpected::Unsigned(x),
						&"the number 5, 6, 7 or 8",
					)),
				}
			}
		}

		deserializer.deserialize_u8(Visitor)
	}
}

#[cfg(feature = "serde")]
impl serde::Serialize for StopBits {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_u8(*self as u8)
	}
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for StopBits {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		struct Visitor;
		impl<'de> serde::de::Visitor<'de> for Visitor {
			type Value = StopBits;
			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("the number 1 or 2")
			}

			fn visit_u64<E: serde::de::Error>(self, data: u64) -> Result<Self::Value, E> {
				match data {
					1 => Ok(StopBits::One),
					2 => Ok(StopBits::Two),
					x => Err(E::invalid_value(
						serde::de::Unexpected::Unsigned(x),
						&"the number 1 or 2",
					)),
				}
			}
		}

		deserializer.deserialize_u8(Visitor)
	}
}

#[cfg(feature = "serde")]
impl serde::Serialize for Parity {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		match self {
			Self::None => serializer.serialize_str("none"),
			Self::Even => serializer.serialize_str("even"),
			Self::Odd => serializer.serialize_str("odd"),
		}
	}
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Parity {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		struct Visitor;
		impl<'de> serde::de::Visitor<'de> for Visitor {
			type Value = Parity;
			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("the string \"none\", \"even\" or \"odd\"")
			}

			fn visit_str<E: serde::de::Error>(self, data: &str) -> Result<Self::Value, E> {
				match data {
					"none" => Ok(Parity::None),
					"even" => Ok(Parity::Even),
					"odd" => Ok(Parity::Odd),
					x => Err(E::invalid_value(
						serde::de::Unexpected::Str(x),
						&"the string \"none\", \"even\" or \"odd\"",
					)),
				}
			}
		}

		deserializer.deserialize_str(Visitor)
	}
}

#[cfg(feature = "serde")]
impl serde::Serialize for FlowControl {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		match self {
			Self::None => serializer.serialize_str("none"),
			Self::XonXoff => serializer.serialize_str("xon/xoff"),
			Self::RtsCts => serializer.serialize_str("rts/cts"),
		}
	}
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for FlowControl {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		struct Visitor;
		impl<'de> serde::de::Visitor<'de> for Visitor {
			type Value = FlowControl;
			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("the string \"none\", \"xon/xoff\" or \"rts/cts\"")
			}

			fn visit_str<E: serde::de::Error>(self, data: &str) -> Result<Self::Value, E> {
				match data {
					"none" => Ok(FlowControl::None),
					"xon/xoff" => Ok(FlowControl::XonXoff),
					"rts/cts" => Ok(FlowControl::RtsCts),
					x => Err(E::invalid_value(
						serde::de::Unexpected::Str(x),
						&"the string \"none\", \"xon/xoff\" or \"rts/cts\"",
					)),
				}
			}
		}

		deserializer.deserialize_str(Visitor)
	}
}
