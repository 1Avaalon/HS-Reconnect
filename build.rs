use std::{env, fs, path::PathBuf, process::Command};

fn find_resource_compiler() -> PathBuf {
    if Command::new("rc.exe").arg("/?").output().is_ok() {
        return PathBuf::from("rc.exe");
    }
    for key in ["WindowsSdkVerBinPath", "WindowsSdkBinPath"] {
        if let Some(base) = env::var_os(key) {
            for tail in ["x64/rc.exe", "rc.exe"] {
                let p = PathBuf::from(&base).join(tail);
                if p.is_file() { return p; }
            }
        }
    }
    let base = env::var_os("ProgramFiles(x86)")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Program Files (x86)"))
        .join("Windows Kits/10/bin");
    let mut versions: Vec<PathBuf> = fs::read_dir(&base)
        .into_iter().flatten().filter_map(Result::ok)
        .map(|d| d.path().join("x64/rc.exe")).filter(|p| p.is_file()).collect();
    versions.sort();
    versions.pop().expect("Windows SDK rc.exe not found. Install the Windows SDK with C++ Build Tools.")
}

fn main() {
    println!("cargo:rerun-if-changed=app.manifest");
    println!("cargo:rerun-if-changed=assets/app.ico");
    println!("cargo:rerun-if-changed=assets/overlay-logo.ico");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") { return; }
    if env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("msvc") {
        panic!("Use x86_64-pc-windows-msvc");
    }
    let root = env::current_dir().expect("Project directory");
    let manifest = root.join("app.manifest");
    let icon = root.join("assets/app.ico");
    assert!(icon.is_file(), "assets/app.ico is missing: upload the assets folder too");
    let overlay_icon = root.join("assets/overlay-logo.ico");
    assert!(overlay_icon.is_file(), "assets/overlay-logo.ico is missing: upload the whole assets folder");
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    let resource_script = out.join("app-icon.rc");
    let resource = out.join("app-icon.res");
    // Forward slashes avoid RC string escape ambiguity in Windows paths.
    let icon_path = icon.to_string_lossy().replace('\\', "/");
    let overlay_path = overlay_icon.to_string_lossy().replace('\\', "/");
    fs::write(&resource_script, format!("101 ICON \"{}\"\n102 ICON \"{}\"\n", icon_path, overlay_path)).expect("Write icon resources");
    let status = Command::new(find_resource_compiler())
        .arg("/nologo").arg("/fo").arg(&resource).arg(&resource_script)
        .status().expect("Start Windows resource compiler");
    assert!(status.success(), "Windows icon resource compilation failed");
    println!("cargo:rustc-link-arg-bin=hs-rust-reconnect={}", resource.display());
    println!("cargo:rustc-link-arg-bin=hs-rust-reconnect=/MANIFESTUAC:NO");
    println!("cargo:rustc-link-arg-bin=hs-rust-reconnect=/MANIFEST:EMBED");
    println!("cargo:rustc-link-arg-bin=hs-rust-reconnect=/MANIFESTINPUT:{}", manifest.display());
}
