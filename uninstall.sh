#!/bin/bash
# Desinstalador de Stress - macOS, Linux y Windows (Git Bash / WSL)
set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

info()    { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC}   $1"; }
warn()    { echo -e "${YELLOW}[WARN]${NC} $1"; }
error()   { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# ── Detectar SO ────────────────────────────────────────────────────
case "$OSTYPE" in
  linux-gnu*) OS="linux"   ;;
  darwin*)    OS="macos"   ;;
  msys*|cygwin*|win32*) OS="windows" ;;
  *) error "Sistema operativo no soportado: $OSTYPE" ;;
esac

# ── Desinstalar ────────────────────────────────────────────────────
if [[ "$OS" == "linux" || "$OS" == "macos" ]]; then
  INSTALL_PATH="/usr/local/bin/stress"

  if [ ! -f "$INSTALL_PATH" ]; then
    warn "No se encontró stress en $INSTALL_PATH. Puede que ya esté desinstalado."
    exit 0
  fi

  info "Eliminando $INSTALL_PATH..."
  if [ -w "/usr/local/bin" ]; then
    rm -f "$INSTALL_PATH"
  else
    sudo rm -f "$INSTALL_PATH"
  fi
  success "stress eliminado de $INSTALL_PATH"

elif [[ "$OS" == "windows" ]]; then
  INSTALL_PATH="$HOME/.local/bin/stress.exe"

  if [ ! -f "$INSTALL_PATH" ]; then
    warn "No se encontró stress en $INSTALL_PATH. Puede que ya esté desinstalado."
    exit 0
  fi

  info "Eliminando $INSTALL_PATH..."
  rm -f "$INSTALL_PATH"
  success "stress.exe eliminado de $HOME/.local/bin"

  warn "La línea de PATH en ~/.bashrc no se eliminó automáticamente."
  warn "Si deseas limpiarla, edita ~/.bashrc y elimina la línea con '.local/bin'."
fi

echo ""
success "Desinstalación completada."
echo ""
