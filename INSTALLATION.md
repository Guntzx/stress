# Instalación detallada

Para la mayoría de los casos, el `README.md` es suficiente.  
Este documento cubre escenarios avanzados.

---

## Dependencias del sistema (Linux)

Si la compilación falla por dependencias faltantes:

```bash
# Ubuntu / Debian
sudo apt-get update
sudo apt-get install -y libssl-dev pkg-config libgtk-3-dev libwebkit2gtk-4.0-dev

# Fedora / RHEL
sudo dnf install -y openssl-devel gtk3-devel webkit2gtk3-devel
```

## Instalación manual del binario

Si prefieres no compilar, descarga el ejecutable de la página de [Releases](https://github.com/Guntzx/stress/releases) y sigue estos pasos:

### macOS / Linux

```bash
# Reemplaza <archivo> con el nombre descargado
chmod +x <archivo>
sudo mv <archivo> /usr/local/bin/stress
stress
```

### Windows

1. Descarga `stress-windows-x64.exe`
2. Renómbralo a `stress.exe`
3. Muévelo a una carpeta en tu PATH (ej: `C:\Windows\System32\` o una carpeta propia)
4. Ejecuta `stress` para abrir la app

## Creación de app nativa en macOS (opcional)

```bash
./create_app.sh
open "Stress.app"
```

Puedes arrastrar `Stress.app` a `/Applications` para acceso desde Launchpad.

## Actualización

```bash
# Desde el directorio del repositorio
git pull
cargo build --release
# Vuelve a copiar el binario al destino de instalación
sudo cp ./target/release/stress /usr/local/bin/stress  # macOS/Linux
```
