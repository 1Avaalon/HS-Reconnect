#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
#[cfg(target_os = "windows")] mod windows;
fn main() {
    #[cfg(target_os = "windows")]
    if let Err(e) = windows::run() { windows::error(&e); }
    #[cfg(not(target_os = "windows"))]
    eprintln!("This reconnect UI requires Windows x64. Parser tests are platform independent.");
}
