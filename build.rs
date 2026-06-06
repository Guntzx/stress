fn main() {
    slint_build::compile("ui/main.slint")
        .expect("Error compilando los archivos Slint de la interfaz");

    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icon.ico");
        res.compile().expect("Error embebiendo icono en el ejecutable");
    }
}
