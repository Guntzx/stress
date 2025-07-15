#!/bin/bash

# =============================================================================
# INSTALADOR UNIVERSAL PARA STRESS
# =============================================================================
# Este script detecta automáticamente el sistema operativo y ejecuta
# la instalación correspondiente sin intervención del usuario.
# Compatible con: Windows (Git Bash/WSL), macOS, Linux
# =============================================================================

set -e

# Colores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Función para imprimir mensajes
print_header() {
    echo -e "${PURPLE}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${PURPLE}║                    INSTALADOR UNIVERSAL STRESS               ║${NC}"
    echo -e "${PURPLE}║              Detección automática de sistema operativo      ║${NC}"
    echo -e "${PURPLE}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

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

print_step() {
    echo -e "${CYAN}→${NC} $1"
}

# Función para detectar el sistema operativo
detect_os() {
    print_step "Detectando sistema operativo..."
    
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        OS="linux"
        print_success "Sistema detectado: Linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        OS="macos"
        print_success "Sistema detectado: macOS"
    elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "win32" ]]; then
        OS="windows"
        print_success "Sistema detectado: Windows"
    else
        print_error "Sistema operativo no soportado: $OSTYPE"
        exit 1
    fi
}

# Función para detectar la arquitectura
detect_architecture() {
    print_step "Detectando arquitectura..."
    
    if [[ "$OS" == "macos" ]]; then
        if [[ "$(uname -m)" == "arm64" ]]; then
            ARCH="arm64"
            EXECUTABLE="stress-macos-arm64"
        else
            ARCH="intel"
            EXECUTABLE="stress-macos-intel"
        fi
    elif [[ "$OS" == "linux" ]]; then
        ARCH="linux"
        EXECUTABLE="stress-linux"
    elif [[ "$OS" == "windows" ]]; then
        ARCH="windows"
        EXECUTABLE="stress-windows.exe"
    fi
    
    print_success "Arquitectura detectada: $ARCH"
}

# Función para verificar prerrequisitos
check_prerequisites() {
    print_step "Verificando prerrequisitos..."
    
    # Verificar si Rust está instalado
    if ! command -v cargo &> /dev/null; then
        print_warning "Rust no está instalado. Instalando Rust..."
        install_rust
    else
        print_success "Rust detectado: $(cargo --version)"
    fi
    
    # Verificar si Git está instalado
    if ! command -v git &> /dev/null; then
        print_error "Git no está instalado. Por favor, instala Git primero."
        exit 1
    else
        print_success "Git detectado: $(git --version)"
    fi
}

# Función para instalar Rust
install_rust() {
    print_step "Instalando Rust..."
    
    if [[ "$OS" == "windows" ]]; then
        # Windows - descargar rustup-init.exe
        print_status "Descargando instalador de Rust para Windows..."
        curl -fsSL https://win.rustup.rs/x86_64 -o rustup-init.exe
        ./rustup-init.exe -y
        rm rustup-init.exe
        source ~/.cargo/env
    else
        # macOS y Linux
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source ~/.cargo/env
    fi
    
    print_success "Rust instalado correctamente"
}

# Función para compilar desde código fuente
compile_from_source() {
    print_step "Compilando desde código fuente..."
    
    # Verificar si estamos en el directorio correcto
    if [ ! -f "Cargo.toml" ]; then
        print_error "No se encontró Cargo.toml"
        print_status "Clonando repositorio..."
        git clone https://github.com/Guntzx/stress.git
        cd stress
    fi
    
    # Compilar en modo release
    print_status "Compilando proyecto..."
    cargo build --release
    
    if [ $? -eq 0 ]; then
        print_success "Compilación exitosa"
        BINARY_PATH="./target/release/stress"
    else
        print_error "Error en la compilación"
        exit 1
    fi
}

# Función para instalar en Linux
install_linux() {
    print_step "Instalando en Linux..."
    
    # Crear directorio de instalación
    INSTALL_DIR="/usr/local/bin"
    
    # Copiar ejecutable
    if sudo cp "$BINARY_PATH" "$INSTALL_DIR/stress"; then
        sudo chmod +x "$INSTALL_DIR/stress"
        print_success "Instalado en $INSTALL_DIR/stress"
    else
        print_error "Error instalando el ejecutable"
        exit 1
    fi
}

# Función para instalar en macOS
install_macos() {
    print_step "Instalando en macOS..."
    
    # Crear directorio de instalación
    INSTALL_DIR="/usr/local/bin"
    
    # Copiar ejecutable
    if sudo cp "$BINARY_PATH" "$INSTALL_DIR/stress"; then
        sudo chmod +x "$INSTALL_DIR/stress"
        print_success "Instalado en $INSTALL_DIR/stress"
    else
        print_error "Error instalando el ejecutable"
        exit 1
    fi
}

# Función para instalar en Windows
install_windows() {
    print_step "Instalando en Windows..."
    
    # Crear directorio de instalación
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
    
    # Copiar ejecutable
    if cp "$BINARY_PATH" "$INSTALL_DIR/stress.exe"; then
        chmod +x "$INSTALL_DIR/stress.exe"
        print_success "Instalado en $INSTALL_DIR/stress.exe"
        
        # Agregar al PATH si no está
        if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
            print_status "Agregando al PATH..."
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
            export PATH="$HOME/.local/bin:$PATH"
        fi
    else
        print_error "Error instalando el ejecutable"
        exit 1
    fi
}

# Función para configurar directorios
setup_directories() {
    print_step "Configurando directorios..."
    
    local config_dir="$HOME/.stress"
    mkdir -p "$config_dir"/{configs,results,logs}
    print_success "Directorios creados en $config_dir/"
}

# Función para verificar instalación
verify_installation() {
    print_step "Verificando instalación..."
    
    if command -v stress &> /dev/null; then
        print_success "stress instalado correctamente"
        stress --help | head -5
    else
        print_error "stress no se encuentra en el PATH"
        exit 1
    fi
}

# Función para mostrar información final
show_final_info() {
    echo ""
    print_success "¡Instalación completada exitosamente!"
    echo ""
    echo -e "${GREEN}🎯 Para usar stress:${NC}"
    echo "  • Interfaz gráfica: stress --gui"
    echo "  • Línea de comandos: stress --help"
    echo ""
    echo -e "${GREEN}📁 Configuraciones guardadas en:${NC} $HOME/.stress/"
    echo ""
    echo -e "${GREEN}📖 Documentación:${NC}"
    echo "  • README.md - Guía completa"
    echo "  • QUICK_START.md - Inicio rápido"
    echo "  • INSTALLATION.md - Instrucciones detalladas"
    echo ""
    echo -e "${GREEN}🔗 Repositorio:${NC} https://github.com/Guntzx/stress"
    echo ""
}

# Función para preguntar si abrir la GUI
ask_open_gui() {
    echo -e "${YELLOW}¿Quieres abrir la interfaz gráfica ahora? (s/n):${NC} "
    read -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Ss]$ ]]; then
        print_status "Abriendo interfaz gráfica..."
        stress --gui &
    fi
}

# Función principal
main() {
    print_header
    
    # Detectar sistema operativo
    detect_os
    
    # Detectar arquitectura
    detect_architecture
    
    # Verificar prerrequisitos
    check_prerequisites
    
    # Compilar desde código fuente
    compile_from_source
    
    # Instalar según el sistema operativo
    case $OS in
        "linux")
            install_linux
            ;;
        "macos")
            install_macos
            ;;
        "windows")
            install_windows
            ;;
        *)
            print_error "Sistema operativo no soportado"
            exit 1
            ;;
    esac
    
    # Configurar directorios
    setup_directories
    
    # Verificar instalación
    verify_installation
    
    # Mostrar información final
    show_final_info
    
    # Preguntar si abrir la GUI
    ask_open_gui
    
    echo ""
    print_success "¡Listo! Puedes cerrar esta ventana."
    echo ""
}

# Ejecutar función principal
main "$@" 