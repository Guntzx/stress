#!/bin/bash

# Instalador de Test Stress para macOS
# Se puede ejecutar con doble clic

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

# Función principal
main() {
    echo "🚀 Instalador de Test Stress"
    echo "=========================="
    echo ""
    
    # Detectar arquitectura
    if [[ "$(uname -m)" == "arm64" ]]; then
        EXECUTABLE="test-stress-macos-arm64"
        print_status "Detectado: macOS ARM64 (Apple Silicon)"
    else
        EXECUTABLE="test-stress-macos-intel"
        print_status "Detectado: macOS Intel"
    fi
    
    # Obtener el directorio del script
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    RELEASES_DIR="$SCRIPT_DIR/releases"
    
    # Verificar que existe el ejecutable
    if [ ! -f "$RELEASES_DIR/$EXECUTABLE" ]; then
        print_error "No se encontró el ejecutable: $EXECUTABLE"
        print_error "Asegúrate de ejecutar primero: ./build_release.sh"
        read -p "Presiona Enter para salir..."
        exit 1
    fi
    
    print_status "Instalando Test Stress..."
    
    # Crear directorio de instalación
    INSTALL_DIR="/usr/local/bin"
    
    # Copiar ejecutable
    if sudo cp "$RELEASES_DIR/$EXECUTABLE" "$INSTALL_DIR/stress"; then
    sudo chmod +x "$INSTALL_DIR/stress"
    print_success "Ejecutable instalado en $INSTALL_DIR/stress"
    else
        print_error "Error instalando el ejecutable"
        read -p "Presiona Enter para salir..."
        exit 1
    fi
    
    # Crear directorios de configuración
    print_status "Configurando directorios..."
    mkdir -p "$HOME/.stress"/{configs,results,logs}
print_success "Directorios creados en $HOME/.stress/"
    
    # Verificar instalación
    print_status "Verificando instalación..."
    if command -v stress &> /dev/null; then
    print_success "Test Stress instalado correctamente"
else
    print_error "Error: stress no se encuentra en el PATH"
        read -p "Presiona Enter para salir..."
        exit 1
    fi
    
    echo ""
    print_success "¡Instalación completada!"
    echo ""
    echo "🎯 Para usar Test Stress:"
echo "  • Interfaz gráfica: stress --gui"
echo "  • Línea de comandos: stress --help"
echo ""
echo "📁 Configuraciones guardadas en: $HOME/.stress/"
    echo ""
    
    # Preguntar si quiere abrir la interfaz gráfica
    read -p "¿Quieres abrir la interfaz gráfica ahora? (s/n): " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Ss]$ ]]; then
        print_status "Abriendo interfaz gráfica..."
        stress --gui &
    fi
    
    echo ""
    print_success "¡Listo! Puedes cerrar esta ventana."
    echo ""
    read -p "Presiona Enter para cerrar..."
}

# Ejecutar función principal
main "$@" 