mod windows;

#[cfg(target_os = "linux")]
pub use self::linux::VolumeProvider;

#[cfg(target_os = "windows")]
pub use self::windows::VolumeProvider;
