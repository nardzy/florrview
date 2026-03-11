
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() != "windows" {
        return;
    }

    let mut res = winres::WindowsResource::new();
    res.set_icon("icon.ico");
    if cfg!(unix) {
        res.set_toolkit_path("/usr/x86_64-w64-mingw32/bin");
        res.set_ar_path("ar");
        res.set_windres_path("/usr/bin/x86_64-w64-mingw32-windres");
    }
    if let Err(e) = res.compile() {
        eprintln!("winres err: {e}");
        std::process::exit(1);
    }
}
