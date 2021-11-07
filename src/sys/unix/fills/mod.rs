use cfg_if::cfg_if;

cfg_if! {
	if #[cfg(any(target_os = "linux", target_os = "android"))] {
		mod linux;
		pub use linux::*;
	} else {
		mod other;
		pub use other::*;
	}
}
