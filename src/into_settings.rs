use crate::Settings;

/// Trait for objects that can configure a serial port.
///
/// This trait is also implemented for `u32`.
/// That implementation also configures a char size of 8 bits,
/// one stop bit, no parity checks and disables flow control.
///
/// If you need more control, you can use a `Fn(Settings) -> std::io::Result<Settings>`.
pub trait IntoSettings {
	/// Apply the configuration to an existing [`Settings`] struct.
	fn apply_to_settings(self, settings: &mut Settings) -> std::io::Result<()>;
}

impl<F> IntoSettings for F
where
	F: FnOnce(Settings) -> std::io::Result<Settings>,
{
	fn apply_to_settings(self, settings: &mut Settings) -> std::io::Result<()> {
		*settings = (self)(settings.clone())?;
		Ok(())
	}
}

impl IntoSettings for u32 {
	fn apply_to_settings(self, settings: &mut Settings) -> std::io::Result<()> {
		settings.set_baud_rate(self)?;
		settings.set_char_size(crate::CharSize::Bits8);
		settings.set_stop_bits(crate::StopBits::One);
		settings.set_parity(crate::Parity::None);
		settings.set_flow_control(crate::FlowControl::None);
		Ok(())
	}
}

/// A serial port "configuration" that simply keeps all existing settings.
///
/// You can pass this to [`SerialPort::open()`] to prevent it from changing any port settings.
///
/// However, be aware that on many platforms, configuration of serial ports resets to a default when they are closed.
/// Usually, you should explicitly configure all important settings.
pub struct KeepSettings;

impl IntoSettings for KeepSettings {
	fn apply_to_settings(self, _settings: &mut Settings) -> std::io::Result<()> {
		Ok(())
	}
}
