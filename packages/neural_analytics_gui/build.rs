fn main() {
    // Compilar el archivo Slint principal
    if let Err(e) = slint_build::compile("src/frames/index.slint") {
        eprintln!("Error al compilar Slint: {:?}", e);
        // Usar panic con mensaje descriptivo en lugar de unwrap()
        panic!("Falló la compilación de Slint: {:?}", e);
    }
}