# Guía de Distribución - Test Stress

Esta guía explica todas las opciones disponibles para distribuir e instalar la aplicación Test Stress en diferentes plataformas.

## 📦 Opciones de Distribución

### 1. **Ejecutables Binarios** (Recomendado para usuarios finales)

#### Compilar para múltiples plataformas
```bash
# Ejecutar script de compilación
./build_release.sh

# Los ejecutables estarán en releases/
# - test-stress-macos-arm64
# - test-stress-macos-intel
# - test-stress-linux (si se compila en Linux)
# - test-stress-windows.exe (si se compila en Windows)
```

#### Instalación manual
```bash
# macOS
chmod +x test-stress-macos-arm64
sudo mv test-stress-macos-arm64 /usr/local/bin/test-stress

# Linux
chmod +x test-stress-linux
sudo mv test-stress-linux /usr/local/bin/test-stress

# Windows
# Copiar test-stress-windows.exe a una carpeta en el PATH
```

### 2. **Instaladores Automáticos**

#### Script de instalación automática
```bash
# Descargar e instalar automáticamente
curl -fsSL https://raw.githubusercontent.com/Guntzx/stress/main/install.sh | bash
```

#### Homebrew (macOS)
```bash
# Agregar el tap
brew tap Guntzx/stress

# Instalar
brew install test-stress
```

### 3. **Paquetes de Sistema**

#### macOS (.pkg)
```bash
# Crear paquete de instalación
./create_pkg.sh

# Instalar
sudo installer -pkg test-stress-1.0.0.pkg -target /
```

#### Linux (.deb)
```bash
# Crear paquete Debian
dpkg-buildpackage -b -us -uc

# Instalar
sudo dpkg -i test-stress_1.0.0_amd64.deb
```

#### Windows (.exe)
```bash
# Requiere NSIS instalado
makensis installer.nsi

# Instalar ejecutando test-stress-setup.exe
```

### 4. **Docker**
```bash
# Construir imagen
docker build -t test-stress .

# Ejecutar
docker run -it --rm test-stress test-stress --help
```

## 🚀 GitHub Actions (Automático)

El proyecto incluye un workflow de GitHub Actions que:

1. **Se activa automáticamente** cuando se crea un tag (ej: `v1.0.0`)
2. **Compila para todas las plataformas** en paralelo
3. **Crea un release** con todos los ejecutables
4. **Genera notas de release** automáticamente

### Crear un release
```bash
# Crear y subir tag
git tag v1.0.0
git push origin v1.0.0

# GitHub Actions compilará automáticamente y creará el release
```

## 📋 Comparación de Métodos

| Método | Facilidad | Tamaño | Instalación | Mantenimiento |
|--------|-----------|--------|-------------|---------------|
| **Binarios** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Script auto** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Homebrew** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Paquetes** | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ |
| **Docker** | ⭐⭐ | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ |

## 🛠️ Configuración por Plataforma

### macOS
- **Requisitos**: macOS 10.15+ (Catalina)
- **Dependencias**: Ninguna (binario estático)
- **Instalación preferida**: Homebrew o script automático

### Linux
- **Requisitos**: Ubuntu 18.04+, CentOS 7+
- **Dependencias**: libssl3, libgtk-3-0, libwebkit2gtk-4.0-37
- **Instalación preferida**: Paquete .deb o script automático

### Windows
- **Requisitos**: Windows 10+ (64-bit)
- **Dependencias**: Visual C++ Redistributable
- **Instalación preferida**: Instalador .exe o Chocolatey

## 📁 Estructura de Archivos de Distribución

```
test-stress/
├── releases/                    # Ejecutables compilados
│   ├── test-stress-macos-arm64
│   ├── test-stress-macos-intel
│   ├── test-stress-linux
│   └── test-stress-windows.exe
├── build_release.sh            # Script de compilación
├── install.sh                  # Script de instalación automática
├── create_pkg.sh              # Script para paquete macOS
├── installer.nsi              # Script para instalador Windows
├── debian/                    # Configuración paquete Debian
│   ├── control
│   └── rules
├── Dockerfile                 # Configuración Docker
├── .github/workflows/         # GitHub Actions
│   └── release.yml
└── Formula/                   # Fórmula Homebrew
    └── test-stress.rb
```

## 🔧 Personalización

### Cambiar información del proyecto
1. **Nombre y versión**: Editar `Cargo.toml`
2. **Descripción**: Actualizar `README.md` e `INSTALLATION.md`
3. **URLs**: Cambiar en `install.sh`, `Formula/test-stress.rb`
4. **Mantenedor**: Actualizar en `debian/control`, `installer.nsi`

### Agregar nuevas plataformas
1. Agregar target en `build_release.sh`
2. Actualizar GitHub Actions workflow
3. Agregar configuración en `install.sh`

## 📞 Soporte de Distribución

### Problemas comunes

#### Error de permisos
```bash
chmod +x test-stress
```

#### Error de dependencias (Linux)
```bash
sudo apt-get install libssl-dev pkg-config libgtk-3-dev libwebkit2gtk-4.0-dev
```

#### Error de certificados (Windows)
- Descargar desde GitHub Releases (certificado válido)
- Verificar firma digital si está disponible

### Verificación de instalación
```bash
# Verificar que funciona
test-stress --help

# Verificar ubicación
which test-stress

# Verificar version
test-stress --version
```

## 🎯 Recomendaciones

### Para usuarios finales
1. **Primera opción**: Descargar desde GitHub Releases
2. **Segunda opción**: Usar script de instalación automática
3. **Tercera opción**: Usar gestor de paquetes del sistema

### Para desarrolladores
1. **Primera opción**: Compilar desde código fuente
2. **Segunda opción**: Usar Docker
3. **Tercera opción**: Usar ejecutables precompilados

### Para distribución empresarial
1. **Primera opción**: Paquetes de sistema (.pkg, .deb, .exe)
2. **Segunda opción**: Docker containers
3. **Tercera opción**: Binarios con instalador personalizado 