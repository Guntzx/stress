# Stress

Aplicación de pruebas de carga multiplataforma (CLI y GUI)

## Instalación por sistema operativo

### macOS

1. **Compilar y crear la app:**
   ```sh
   ./create_app.sh
   ```
   Esto generará `Stress.app` en el directorio actual.

2. **Abrir la app:**
   ```sh
   open "Stress.app"
   ```
   O arrástrala a la carpeta Aplicaciones para tener acceso desde Launchpad/Spotlight.

3. **(Opcional) Icono personalizado:**
   Coloca un archivo `icon.icns` en la raíz del proyecto antes de ejecutar el script para que la app tenga tu icono.

### Linux

1. **Compilar el binario:**
   ```sh
   cargo build --release
   ```
   El ejecutable estará en `target/release/stress`.

2. **Ejecutar la app:**
   ```sh
   ./target/release/stress
   ```
   Puedes crear un acceso directo o lanzador personalizado si lo deseas.

### Windows

1. **Compilar el binario:**
   ```sh
   cargo build --release
   ```
   El ejecutable estará en `target\release\stress.exe`.

2. **Ejecutar la app:**
   Haz doble clic en el ejecutable o ejecútalo desde la terminal:
   ```sh
   .\target\release\stress.exe
   ```

3. **(Opcional) Instalador:**
   Puedes usar el script NSIS o el instalador incluido para crear un instalador de Windows.

---

## Notas adicionales
- La app soporta tanto interfaz gráfica como línea de comandos.
- En macOS y Linux, puedes crear accesos directos personalizados para mayor comodidad.
- Consulta la pestaña "Opciones Generales" para personalizar la visualización de la terminal/logs.

Para más detalles, revisa los archivos `README_RUST.md`, `UNIVERSAL_INSTALLER.md`, `WINDOWS_INSTALLER.md` o la documentación específica de cada sistema operativo. 