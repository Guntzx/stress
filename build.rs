// build.rs — compila los archivos Slint antes de construir el binario
fn main() {
    slint_build::compile("ui/main.slint")
        .expect("Error compilando los archivos Slint de la interfaz");
}
