use cfg_if::cfg_if;

cfg_if! {
	if #[cfg(any(
			target_os = "dragonfly",
			target_os = "freebsd",
			target_os = "ios",
			target_os = "macos",
			target_os = "netbsd",
			target_os = "openbsd",
	))] {
		mod bsd;
		pub use bsd::*;

	} else if #[cfg(any(
		target_os = "linux",
		target_os = "android",
	))] {
		mod linux;
		pub use linux::*;

	} else if #[cfg(any(
		target_os = "illumos",
		target_os = "solaris",
	))] {
		mod solarish;
		pub use solarish::*;

	} else {
		mod other;
		pub use other::*;
	}
}
