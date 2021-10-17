pub struct SerialSettings {
	pub baud_rate: u32,
	pub char_size: CharSize,
	pub stop_bits: StopBits,
	pub parity: Parity,
	pub flow_control: FlowControl,
}

pub const BAUD_110: u32 = 110;
pub const BAUD_300: u32 = 300;
pub const BAUD_600: u32 = 600;
pub const BAUD_1200: u32 = 1200;
pub const BAUD_2400: u32 = 2400;
pub const BAUD_4800: u32 = 4800;
pub const BAUD_9600: u32 = 9600;
pub const BAUD_19200: u32 = 19200;
pub const BAUD_38400: u32 = 38400;
pub const BAUD_57600: u32 = 57600;
pub const BAUD_115200: u32 = 115200;

pub use CharSize::*;
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum CharSize {
	Bits5,
	Bits6,
	Bits7,
	Bits8,
}

pub use StopBits::*;
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum StopBits {
	Stop1,
	Stop2,
}

pub use Parity::*;
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Parity {
	ParityNone,
	ParityOdd,
	ParityEven,
}

pub use FlowControl::*;
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum FlowControl {
	FlowControlNone,
	FlowControlSoftware,
	FlowControlHardware,
}
