#[cfg(target_os = "linux")]
fn main() {
    println!("cargo:rustc-link-lib=X11");
}

#[cfg(target_os = "windows")]
fn main() {}

#[cfg(target_os = "macos")]
fn main() {}
