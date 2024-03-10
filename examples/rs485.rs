use std::{thread, time::Duration};
use serial2::{SerialPort, Settings};

fn main() -> std::io::Result<()> {
    let port_name = "/dev/ttyS5";
    let ser = SerialPort::open(port_name, |mut settings: Settings| {
        settings.set_raw();
        settings.set_baud_rate(115200)?;
        settings.enable_rs485();
        Ok(settings)
    })?;

    loop {
        ser.write(b"test").unwrap();
        thread::sleep(Duration::from_millis(500));
    }
}