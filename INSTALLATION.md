# Instalación de Test Stress

## 📦 Descarga de ejecutables precompilados

### Opción 1: Descargar desde releases
1. Ve a la sección [Releases](../../releases) de este repositorio
2. Descarga el ejecutable correspondiente a tu plataforma:
   - **macOS ARM64 (Apple Silicon)**: `test-stress-macos-arm64`
   - **macOS Intel**: `test-stress-macos-intel`
   - **Linux**: `test-stress-linux`
   - **Windows**: `test-stress-windows.exe`

### Opción 2: Compilar desde código fuente

#### Prerrequisitos
- [Rust](https://rustup.rs/) (versión 1.70 o superior)
- Git

#### Pasos de compilación
```bash
# Clonar el repositorio
git clone https://github.com/Guntzx/stress.git
cd stress

# Compilar para tu plataforma
cargo build --release

# El ejecutable estará en: target/release/stress
```

## 🚀 Instalación por plataforma

### macOS

#### Opción 1: Instalación manual
```bash
# Descargar el ejecutable
curl -L -o test-stress https://github.com/Guntzx/stress/releases/latest/download/test-stress-macos-arm64

# Hacer ejecutable
chmod +x stress

# Mover a /usr/local/bin (requiere sudo)
sudo mv test-stress /usr/local/bin/

# Verificar instalación
stress --help
```

#### Opción 2: Usar Homebrew (recomendado)
```bash
# Agregar el tap (si es necesario)
brew tap Guntzx/stress

# Instalar
brew install stress

# Verificar instalación
stress --help
```

### Linux

#### Opción 1: Instalación manual
```bash
# Descargar el ejecutable
wget https://github.com/Guntzx/stress/releases/latest/download/test-stress-linux

# Hacer ejecutable
chmod +x stress-linux

# Mover a /usr/local/bin (requiere sudo)
sudo mv test-stress-linux /usr/local/bin/test-stress

# Verificar instalación
stress --help
```

#### Opción 2: Usar Snap (si está disponible)
```bash
sudo snap install test-stress
```

### Windows

#### Opción 1: Instalación manual
1. Descarga `test-stress-windows.exe`
2. Colócalo en una carpeta (ej: `C:\Program Files\TestStress\`)
3. Agrega la carpeta al PATH del sistema
4. Abre Command Prompt y ejecuta: `test-stress-windows.exe --help`

#### Opción 2: Usar Chocolatey (si está disponible)
```cmd
choco install test-stress
```

#### Opción 3: Usar Scoop (si está disponible)
```cmd
scoop install test-stress
```

## 🎯 Uso básico

### Interfaz gráfica
```bash
stress --gui
```

### Línea de comandos
```bash
# Prueba individual
stress single --base-url "http://localhost:8080" --endpoint "/api/test" --iterations 100

# Suite de pruebas
stress suite --config-file "mi-suite.json"

# Ver ayuda
stress --help
```

## 📁 Estructura de archivos

Después de la instalación, la aplicación creará:
```
~/.stress/
├── configs/          # Configuraciones guardadas
├── results/          # Resultados de pruebas
└── logs/            # Logs de la aplicación
```

## 🔧 Configuración

### Variables de entorno (opcional)
```bash
# Directorio de configuración
export STRESS_CONFIG_DIR="$HOME/.stress"

# Directorio de resultados
export STRESS_RESULTS_DIR="$HOME/.stress/results"

# Nivel de logging
export RUST_LOG=info
```

## 🐛 Solución de problemas

### Error: "Permission denied"
```bash
chmod +x stress
```

### Error: "Command not found"
- Verifica que el ejecutable esté en el PATH
- En Windows, asegúrate de que la extensión `.exe` esté incluida

### Error: "Library not found" (Linux)
```bash
# Instalar dependencias del sistema
sudo apt-get update
sudo apt-get install libssl-dev pkg-config
```

### Error: "GUI not working" (Linux)
```bash
# Instalar dependencias de GUI
sudo apt-get install libgtk-3-dev libwebkit2gtk-4.0-dev
```

## 📞 Soporte

Si encuentras problemas:
1. Revisa los logs en `~/.stress/logs/`
2. Ejecuta con `RUST_LOG=debug` para más información
3. Abre un issue en GitHub con los detalles del error

## 🔄 Actualizaciones

Para actualizar la aplicación:
```bash
# Si instalaste con Homebrew
brew upgrade stress

# Si instalaste manualmente
# Descarga la nueva versión y reemplaza el ejecutable
``` 