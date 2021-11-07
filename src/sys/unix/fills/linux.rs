use cfg_if::cfg_if;

// IBSHIFT is 16 on all architectures.
//
// But we don't use it on the PowerPC architecture.
#[cfg(not(any(
	target_arch = "powerpc",
	target_arch = "powerpc64",
)))]
pub const IBSHIFT: libc::tcflag_t = 16;

// BOTHER is missing for musl/uclibc targets
//
// It's easier to just define it for all architectures though, also for GNU targets.
cfg_if! {
	// Generic
	if #[cfg(any(
		target_arch = "x86_64",
		target_arch = "x86",
		target_arch = "arm",
		target_arch = "aarch64",
		target_arch = "riscv32",
		target_arch = "riscv64",
		target_arch = "s390x",
	))]
	{
		pub const BOTHER: libc::speed_t = 0o010000;

	// MIPS
	} else if #[cfg(any(
		target_arch = "mips",
		target_arch = "mips64",
	))]
	{
		pub const BOTHER: libc::speed_t = 0o010000;

	// SPARC
	} else if #[cfg(any(
		target_arch = "sparc",
		target_arch = "sparc64",
	))]
	{
		pub const BOTHER: libc::speed_t = 0x1000;

	// PowerPC
	} else if #[cfg(any(
		target_arch = "powerpc",
		target_arch = "powerpc64",
	))]
	{
		// pub const BOTHER: libc::speed_t = 0o0037;
	}
}

// MIPS+musl/uclibc is missing TIOCM constants.
cfg_if! {
	if #[cfg(all(
		any(
			target_arch = "mips",
			target_arch = "mips64",
		),
		not(target_env = "gnu"),
	))]
	{
		pub const TIOCMBIS: u64 = 0x741B;
		pub const TIOCMBIC: u64 = 0x741C;
		pub const TIOCMGET: u64 = 0x741D;
		pub const TIOCM_RTS: libc::c_int = 0x004;
		pub const TIOCM_CTS: libc::c_int = 0x040;
		pub const TIOCM_DTR: libc::c_int = 0x002;
		pub const TIOCM_DSR: libc::c_int = 0x400;
		pub const TIOCM_RI: libc::c_int = 0x200;
		pub const TIOCM_CD: libc::c_int = 0x100;
	} else {
		pub use libc::TIOCMBIS;
		pub use libc::TIOCMBIC;
		pub use libc::TIOCMGET;
		pub use libc::TIOCM_RTS;
		pub use libc::TIOCM_CTS;
		pub use libc::TIOCM_DTR;
		pub use libc::TIOCM_DSR;
		pub use libc::TIOCM_RI;
		pub use libc::TIOCM_CD;
	}
}
