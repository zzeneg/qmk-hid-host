#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use self::linux::LayoutProvider;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use self::windows::LayoutProvider;
