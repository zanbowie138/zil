//! Gives zil.exe its icon and version info (from Cargo.toml), which Explorer and the installer show.
fn main() {
    println!("cargo:rerun-if-changed=zil.ico");
    println!("cargo:rerun-if-changed=../../Cargo.toml");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new().set_icon("zil.ico").compile().expect("embed the Windows icon and version info");
    }
}
