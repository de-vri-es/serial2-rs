// All values taken from:
// https://github.com/illumos/illumos-gate/blob/252adeb303174e992b64771bf9639e63a4d55418/usr/src/uts/common/sys/termios.h

pub use libc::TIOCMBIS;
pub use libc::TIOCMBIC;
pub use libc::TIOCMGET;
//pub const TIOCM_LE: i32 = 0o0001;
pub const TIOCM_DTR: i32 = 0o0002;
pub const TIOCM_RTS: i32 = 0o0004;
//pub const TIOCM_ST: i32 = 0o0010;
//pub const TIOCM_SR: i32 = 0020;
pub const TIOCM_CTS: i32 = 0o0040;
pub const TIOCM_CAR: i32 = 0o0100;
pub const TIOCM_CD: i32 = TIOCM_CAR;
pub const TIOCM_RNG: i32 = 0o0200;
pub const TIOCM_RI: i32 = TIOCM_RNG;
pub const TIOCM_DSR: i32 = 0o0400;
