# Stress

Aplicación de pruebas de carga multiplataforma (CLI y GUI)

---

## 🚀 Instalación y configuración para desarrolladores

### Requisitos

| Herramienta | Versión mínima | Enlace instalación           |
|-------------|----------------|-----------------------------|
| Rust        | 1.70           | https://rustup.rs/          |
| Git         | Cualquiera     | https://git-scm.com/        |

### 1. Clonar el repositorio
```sh
git clone https://github.com/Guntzx/stress.git
cd stress
```

### 2. Instalar dependencias y preparar entorno
```sh
# Instala Rust si no lo tienes
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Configura archivos de entorno
./setup.sh
# O manualmente:
cp env.qa.example .env.qa
cp env.prod.example .env.prod
# Edita las contraseñas en .env.qa y .env.prod
```

### 3. Compilar el proyecto
```sh
cargo build --release   # Para producción
# o
cargo build             # Para desarrollo
```

### 4. Ejecutar la aplicación

- **Modo desarrollo (hot reload, debug):**
  ```sh
  cargo run -- --gui
  # o solo CLI
  cargo run -- --help
  ```
- **Modo producción (binario optimizado):**
  ```sh
  ./target/release/stress --gui
  ./target/release/stress --help
  ```

---

## Instalación por sistema operativo (usuarios finales)

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