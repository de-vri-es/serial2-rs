use crate::Settings;

/// Trait for objects that can configure a serial port.
///
/// Usually, you can simply pass a `u32` when an implementor of this trait is required.
/// That uses the `u32` to configure the baud rate,
/// sets the char size to 8 bits, sets one stop bit, disables parity checks and flow control.
///
/// If you need more control, you can use a `Fn(Settings) -> std::io::Result<Settings>`.
///
/// If you want to open the serial port without modifying any settings, you can use [`KeepSettings`].
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
/// Note: many platforms reset the configuration of a serial port when it is no longer in use.
/// You should normally explicitly configure the settings that you care about.
pub struct KeepSettings;

impl IntoSettings for KeepSettings {
	fn apply_to_settings(self, _settings: &mut Settings) -> std::io::Result<()> {
		Ok(())
	}
}
