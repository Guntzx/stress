#!/bin/bash

# Script de instalación automática para stress
set -e

# Colores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Función para imprimir mensajes
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detectar sistema operativo
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    else
        echo "unknown"
    fi
}

# Detectar arquitectura
detect_arch() {
    if [[ "$(uname -m)" == "x86_64" ]]; then
        echo "x86_64"
    elif [[ "$(uname -m)" == "arm64" ]] || [[ "$(uname -m)" == "aarch64" ]]; then
        echo "arm64"
    else
        echo "unknown"
    fi
}

# Descargar ejecutable
download_executable() {
    local os=$1
    local arch=$2
    
    print_status "Detectado: $os $arch"
    
    case "$os" in
        "macos")
            if [[ "$arch" == "arm64" ]]; then
                filename="test-stress-macos-arm64"
            else
                filename="test-stress-macos-intel"
            fi
            ;;
        "linux")
            filename="test-stress-linux"
            ;;
        *)
            print_error "Sistema operativo no soportado: $os"
            exit 1
            ;;
    esac
    
    print_status "Descargando $filename..."
    
    # URL del release (ajustar según tu repositorio)
    # Por ahora, usar el archivo local
    local url="file://$(pwd)/releases/$filename"
    
    if command -v curl &> /dev/null; then
        curl -L -o "$filename" "$url"
    elif command -v wget &> /dev/null; then
        wget -O "$filename" "$url"
    else
        print_error "No se encontró curl ni wget. Instala uno de ellos."
        exit 1
    fi
    
    if [ ! -f "$filename" ]; then
        print_error "Error descargando el ejecutable"
        exit 1
    fi
    
    chmod +x "$filename"
    print_success "Ejecutable descargado: $filename"
}

# Instalar en el sistema
install_executable() {
    local filename=$1
    
    print_status "Instalando stress..."
    
    # Crear directorio de instalación
    local install_dir="/usr/local/bin"
    
    if [ ! -w "$install_dir" ]; then
        print_warning "No tienes permisos de escritura en $install_dir"
        print_status "Intentando con sudo..."
        
        if sudo mv "$filename" "$install_dir/stress"; then
    print_success "Instalado en $install_dir/stress"
        else
            print_error "Error instalando con sudo"
            exit 1
        fi
    else
        mv "$filename" "$install_dir/stress"
print_success "Instalado en $install_dir/stress"
    fi
}

# Crear directorios de configuración
setup_directories() {
    print_status "Configurando directorios..."
    
    local config_dir="$HOME/.stress"
    mkdir -p "$config_dir"/{configs,results,logs}
    
    print_success "Directorios creados en $config_dir"
}

# Verificar instalación
verify_installation() {
    print_status "Verificando instalación..."
    
    if command -v stress &> /dev/null; then
    print_success "stress instalado correctamente"
    stress --help | head -5
else
    print_error "stress no se encuentra en el PATH"
        exit 1
    fi
}

# Función principal
main() {
    echo "🚀 Instalador de Test Stress"
    echo "=========================="
    
    local os=$(detect_os)
    local arch=$(detect_arch)
    
    if [[ "$os" == "unknown" ]]; then
        print_error "Sistema operativo no soportado"
        exit 1
    fi
    
    if [[ "$arch" == "unknown" ]]; then
        print_error "Arquitectura no soportada"
        exit 1
    fi
    
    download_executable "$os" "$arch"
    install_executable "$filename"
    setup_directories
    verify_installation
    
    echo ""
    print_success "¡Instalación completada!"
    echo ""
    echo "🎯 Para usar stress:"
echo "  • Interfaz gráfica: stress --gui"
echo "  • Línea de comandos: stress --help"
echo ""
echo "📁 Configuraciones guardadas en: $HOME/.stress/"
}

# Ejecutar función principal
main "$@" 