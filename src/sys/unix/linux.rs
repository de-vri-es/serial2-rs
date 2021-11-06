use cfg_if::cfg_if;

cfg_if! {
	if #[cfg(any(
		target_arch = "x86_64",
		target_arch = "x86",
		target_arch = "arm",
		target_arch = "aarch64",
		target_arch = "riscv32",
		target_arch = "riscv64",
	))]
	{
		pub const BOTHER: libc::speed_t = 0o010000;

		pub const TCGETS2: u64 = 0x802c542a;
		pub const TCSETSW2: u64 = 0x402c542c;

		#[repr(C)]
		#[derive(Clone)]
		#[allow(incorrect_ident_case)]
		pub struct termios2 {
			pub c_iflag: libc::tcflag_t,
			pub c_oflag: libc::tcflag_t,
			pub c_cflag: libc::tcflag_t,
			pub c_lflag: libc::tcflag_t,
			pub c_line: libc::cc_t,
			pub c_cc: [libc::cc_t; 19],
			pub c_ispeed: libc::speed_t,
			pub c_ospeed: libc::speed_t,
		}
	} else if #[cfg(any(
		target_arch = "mips",
		target_arch = "mips64",
	))]
	{
		pub const BOTHER: libc::speed_t = 0o010000;

		pub const TCGETS2: u64 = 0x4030542a;
		pub const TCSETSW2: u64 = 0x8030542c;

		#[repr(C)]
		#[derive(Clone)]
		#[allow(incorrect_ident_case)]
		pub struct termios2 {
			pub c_iflag: libc::tcflag_t,
			pub c_oflag: libc::tcflag_t,
			pub c_cflag: libc::tcflag_t,
			pub c_lflag: libc::tcflag_t,
			pub c_line: libc::cc_t,
			pub c_cc: [libc::cc_t; 23],
			pub c_ispeed: libc::speed_t,
			pub c_ospeed: libc::speed_t,
		}
	} else if #[cfg(any(
		target_arch = "sparc",
		target_arch = "sparc64",
	))]
	{
		pub const BOTHER: libc::speed_t = 0x1000;

		pub const TCGETS2: u64 = 0x402c540c;
		pub const TCSETSW2: u64 = 0x802c540e;

		#[repr(C)]
		#[derive(Clone)]
		#[allow(incorrect_ident_case)]
		pub struct termios2 {
			pub c_iflag: libc::tcflag_t,
			pub c_oflag: libc::tcflag_t,
			pub c_cflag: libc::tcflag_t,
			pub c_lflag: libc::tcflag_t,
			pub c_line: libc::cc_t,
			pub c_cc: [libc::cc_t; 19],
			pub c_ispeed: libc::speed_t,
			pub c_ospeed: libc::speed_t,
		}
	}
}
