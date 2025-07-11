# Test Stress - Pruebas de Carga

Una aplicación completa de pruebas de carga escrita en Rust con interfaz gráfica y línea de comandos.

## 🚀 Características

- **Interfaz gráfica intuitiva** con pestañas para pruebas individuales y suites
- **Línea de comandos completa** para automatización
- **Múltiples métodos HTTP** (GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS)
- **Peticiones concurrentes** configurables
- **Guardado y carga de configuraciones** para reutilizar pruebas
- **Resultados detallados** con tiempos promedio, mínimo y máximo
- **Cancelación de pruebas** en tiempo real
- **Logs en tiempo real** con limpieza automática
- **Soporte multiplataforma** (macOS, Linux, Windows)

## 📦 Instalación

### Opción 1: Descarga directa (Recomendado)

Descarga el ejecutable para tu plataforma desde [Releases](../../releases):

- **macOS ARM64 (Apple Silicon)**: `test-stress-macos-arm64`
- **macOS Intel**: `test-stress-macos-intel`
- **Linux**: `test-stress-linux`
- **Windows**: `test-stress-windows.exe`

### Opción 2: Instalador Universal (Recomendado)

**Para cualquier sistema operativo (Windows, macOS, Linux):**

```bash
# Descargar y ejecutar instalador universal
curl -fsSL https://raw.githubusercontent.com/Guntzx/stress/main/install_universal.sh | bash
```

**Para Windows (PowerShell):**
```powershell
# Ejecutar en PowerShell
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/Guntzx/stress/main/install_universal.ps1" -OutFile "install.ps1"; .\install.ps1
```

**Para Windows (CMD):**
```cmd
# Ejecutar en CMD
install_universal.bat
```

### Opción 3: Instalación automática (legacy)

```bash
# Descargar e instalar automáticamente
curl -fsSL https://raw.githubusercontent.com/Guntzx/stress/main/install.sh | bash
```

### Opción 4: Homebrew (macOS)

```bash
# Agregar el tap
brew tap Guntzx/stress

# Instalar
brew install stress
```

### Opción 5: Compilar desde código fuente

```bash
# Prerrequisitos: Rust 1.70+
git clone https://github.com/Guntzx/stress.git
cd stress
cargo build --release
```

## 🎯 Uso

### Interfaz gráfica
```bash
stress --gui
```

### Línea de comandos
```bash
# Prueba individual
stress single \
  --base-url "http://localhost:8080" \
  --endpoint "/api/test" \
  --method POST \
  --iterations 100 \
  --concurrent 10

# Suite de pruebas
stress suite --config-file "mi-suite.json"

# Ver ayuda
stress --help
```

## 📁 Estructura del proyecto

```
test-stress/
├── src/
│   ├── main.rs          # Punto de entrada y CLI
│   ├── gui.rs           # Interfaz gráfica
│   ├── load_test.rs     # Motor de pruebas de carga
│   ├── models.rs        # Modelos de datos
│   ├── config.rs        # Gestión de configuraciones
│   └── report_generator.rs # Generación de reportes
├── configs/             # Configuraciones guardadas
├── results/             # Resultados de pruebas
├── build_release.sh     # Script de compilación multiplataforma
├── install.sh           # Script de instalación automática
└── INSTALLATION.md      # Instrucciones detalladas de instalación
```

## 🔧 Configuración

La aplicación crea automáticamente los directorios necesarios:

```
~/.stress/
├── configs/          # Configuraciones guardadas
├── results/          # Resultados de pruebas
└── logs/            # Logs de la aplicación
```

## 📊 Ejemplo de uso

### 1. Configurar una prueba individual
- Abre la aplicación: `stress --gui`
- Ve a la pestaña "Prueba Individual"
- Configura la URL base, endpoint, método HTTP
- Establece iteraciones y peticiones simultáneas
- Haz clic en "▶️ Ejecutar"

### 2. Crear una suite de pruebas
- Ve a la pestaña "Suite de Pruebas"
- Agrega múltiples peticiones a la suite
- Configura parámetros globales
- Ejecuta toda la suite

### 3. Guardar y cargar configuraciones
- Usa "💾 Guardar Configuración" para guardar
- Ve a "Configuraciones" para cargar configuraciones guardadas
- Reutiliza configuraciones en diferentes sesiones

## 🛠️ Desarrollo

### Compilar para múltiples plataformas

```bash
# Ejecutar script de compilación
./build_release.sh

# Los ejecutables estarán en releases/
```

### Crear un release

```bash
# Crear un tag
git tag v1.0.0
git push origin v1.0.0

# GitHub Actions compilará automáticamente para todas las plataformas
```

## 📋 Requisitos del sistema

- **macOS**: 10.15+ (Catalina o superior)
- **Linux**: Ubuntu 18.04+, CentOS 7+, o distribución equivalente
- **Windows**: Windows 10+ (64-bit)
- **Memoria**: 512MB RAM mínimo
- **Espacio**: 50MB de espacio libre

## 🐛 Solución de problemas

### Error: "Permission denied"
```bash
chmod +x stress
```

### Error: "GUI not working" (Linux)
```bash
sudo apt-get install libgtk-3-dev libwebkit2gtk-4.0-dev
```

### Error: "Library not found" (Linux)
```bash
sudo apt-get install libssl-dev pkg-config
```

## 🤝 Contribuir

1. Fork el proyecto
2. Crea una rama para tu feature (`git checkout -b feature/AmazingFeature`)
3. Commit tus cambios (`git commit -m 'Add some AmazingFeature'`)
4. Push a la rama (`git push origin feature/AmazingFeature`)
5. Abre un Pull Request

## 📄 Licencia

Este proyecto está bajo la Licencia MIT. Ver el archivo `LICENSE` para más detalles.

## 📞 Soporte

- 📧 Email: (pendiente)
- 🐛 Issues: [GitHub Issues](../../issues)
- 📖 Documentación: [INSTALLATION.md](INSTALLATION.md)

## 🙏 Agradecimientos

- [egui](https://github.com/emilk/egui) - Interfaz gráfica
- [reqwest](https://github.com/seanmonstar/reqwest) - Cliente HTTP
- [tokio](https://github.com/tokio-rs/tokio) - Runtime asíncrono
- [serde](https://github.com/serde-rs/serde) - Serialización

## 👨‍💻 Autor

**Yerko** - [@Guntzx](https://github.com/Guntzx) 