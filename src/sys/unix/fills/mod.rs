use cfg_if::cfg_if;

cfg_if! {
	if #[cfg(any(target_os = "linux", target_os = "android"))] {
		mod linux;
		pub use linux::*;
	} else if #[cfg(target_os = "illumos")] {
		mod illumos;
		pub use illumos::*;
	} else {
		mod other;
		pub use other::*;
	}
}
