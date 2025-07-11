# 🚀 Instalador Universal - Stress

## 📋 Descripción

El **Instalador Universal** es una solución que detecta automáticamente tu sistema operativo y ejecuta todos los pasos de instalación sin intervención del usuario. Es la forma más sencilla de instalar Stress en cualquier plataforma.

## 🎯 Características

- ✅ **Detección automática** de sistema operativo
- ✅ **Instalación automática** de Rust (si no está instalado)
- ✅ **Compilación automática** desde código fuente
- ✅ **Configuración automática** de directorios
- ✅ **Verificación automática** de la instalación
- ✅ **Compatibilidad total** con Windows, macOS y Linux

## 🖥️ Sistemas Soportados

| Sistema Operativo | Arquitectura | Estado |
|-------------------|--------------|--------|
| Windows 10/11 | x64, x86 | ✅ Soportado |
| macOS 10.15+ | Intel, Apple Silicon | ✅ Soportado |
| Ubuntu 18.04+ | x64 | ✅ Soportado |
| CentOS 7+ | x64 | ✅ Soportado |
| WSL (Windows) | x64 | ✅ Soportado |

## 🚀 Instalación Rápida

### Opción 1: Una sola línea (Recomendado)

```bash
curl -fsSL https://raw.githubusercontent.com/Guntzx/stress/main/install_universal.sh | bash
```

### Opción 2: Windows PowerShell

```powershell
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/Guntzx/stress/main/install_universal.ps1" -OutFile "install.ps1"; .\install.ps1
```

### Opción 3: Windows CMD

```cmd
install_universal.bat
```

## 📋 Prerrequisitos

El instalador verificará y instalará automáticamente:

- ✅ **Git** (requerido)
- ✅ **Rust** (instalado automáticamente si no está presente)

## 🔧 Proceso de Instalación

El instalador ejecuta automáticamente estos pasos:

1. **Detección de sistema operativo**
2. **Detección de arquitectura**
3. **Verificación de prerrequisitos**
4. **Instalación de Rust** (si es necesario)
5. **Clonación del repositorio**
6. **Compilación del proyecto**
7. **Instalación del ejecutable**
8. **Configuración de directorios**
9. **Verificación de la instalación**

## 📁 Estructura de Archivos

Después de la instalación, se crean automáticamente:

```
~/.stress/
├── configs/          # Configuraciones guardadas
├── results/          # Resultados de pruebas
└── logs/            # Logs de la aplicación
```

## 🎯 Uso Inmediato

Una vez instalado, puedes usar Stress inmediatamente:

```bash
# Interfaz gráfica
stress --gui

# Línea de comandos
stress --help

# Ejemplo de prueba
stress single --base-url "http://localhost:8080" --iterations 100
```

## 🔍 Solución de Problemas

### Error: "Permission denied"
```bash
# En Linux/macOS
sudo chmod +x /usr/local/bin/stress

# En Windows
# El instalador maneja esto automáticamente
```

### Error: "Command not found"
```bash
# Reinicia tu terminal o ejecuta:
source ~/.bashrc  # Linux/macOS
# En Windows, reinicia CMD/PowerShell
```

### Error: "Rust installation failed"
```bash
# Instala Rust manualmente:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Error: "Git not found"
```bash
# Instala Git desde: https://git-scm.com/
```

## 📊 Comparación de Métodos

| Método | Facilidad | Velocidad | Control |
|--------|-----------|-----------|---------|
| **Instalador Universal** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| Descarga directa | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ |
| Homebrew | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| Compilación manual | ⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ |

## 🔄 Actualización

Para actualizar Stress:

```bash
# Ejecutar el instalador nuevamente
curl -fsSL https://raw.githubusercontent.com/Guntzx/stress/main/install_universal.sh | bash
```

## 🛠️ Desarrollo

### Estructura del Instalador

```
install_universal.sh      # Instalador principal (Bash)
install_universal.ps1     # Instalador PowerShell (Windows)
install_universal.bat     # Wrapper para Windows CMD
```

### Personalización

Puedes modificar el instalador para:
- Cambiar el directorio de instalación
- Agregar dependencias adicionales
- Personalizar la configuración
- Agregar validaciones específicas

## 📞 Soporte

Si encuentras problemas con el instalador:

1. **Revisa los logs** en `~/.stress/logs/`
2. **Ejecuta con debug**: `RUST_LOG=debug curl -fsSL ...`
3. **Abre un issue** en GitHub con:
   - Sistema operativo y versión
   - Arquitectura
   - Logs de error completos

## 🎉 ¡Listo!

El instalador universal hace que instalar Stress sea tan simple como ejecutar una línea de comando. ¡No más pasos manuales, no más configuraciones complejas!

---

**¿Necesitas ayuda?** Consulta la [documentación completa](README.md) o [abre un issue](https://github.com/Guntzx/stress/issues). 