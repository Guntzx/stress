#!/bin/bash
# Actualizador de Stress - macOS, Linux y Windows (Git Bash / WSL)
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

# ── Verificar dependencias ─────────────────────────────────────────
command -v git   &>/dev/null || error "Git no encontrado. Instálalo y vuelve a intentarlo."
command -v cargo &>/dev/null || error "Rust/Cargo no encontrado. Instálalo desde https://rustup.rs y vuelve a intentarlo."

# ── Obtener o actualizar el repositorio ───────────────────────────
REPO_URL="https://github.com/Guntzx/stress.git"
REPO_DIR="$HOME/.local/share/stress"

if [ -d "$REPO_DIR/.git" ]; then
  info "Actualizando repositorio en $REPO_DIR..."
  git -C "$REPO_DIR" pull --ff-only
elif [ -f "Cargo.toml" ]; then
  info "Repositorio local detectado. Actualizando..."
  git pull --ff-only
  REPO_DIR="."
else
  info "Clonando repositorio en $REPO_DIR..."
  mkdir -p "$(dirname "$REPO_DIR")"
  git clone "$REPO_URL" "$REPO_DIR"
fi

cd "$REPO_DIR"

# ── Compilar ───────────────────────────────────────────────────────
info "Compilando nueva versión..."
cargo build --release
success "Compilación completada"

BINARY="./target/release/stress"
[[ "$OS" == "windows" ]] && BINARY="./target/release/stress.exe"

# ── Instalar ───────────────────────────────────────────────────────
if [[ "$OS" == "linux" || "$OS" == "macos" ]]; then
  INSTALL_DIR="/usr/local/bin"
  if [ -w "$INSTALL_DIR" ]; then
    cp "$BINARY" "$INSTALL_DIR/stress"
  else
    sudo cp "$BINARY" "$INSTALL_DIR/stress"
  fi
  chmod +x "$INSTALL_DIR/stress" 2>/dev/null || sudo chmod +x "$INSTALL_DIR/stress"
  success "Actualizado en $INSTALL_DIR/stress"

elif [[ "$OS" == "windows" ]]; then
  INSTALL_DIR="$HOME/.local/bin"
  mkdir -p "$INSTALL_DIR"
  cp "$BINARY" "$INSTALL_DIR/stress.exe"
  success "Actualizado en $INSTALL_DIR/stress.exe"
fi

echo ""
success "Actualización completada."
echo ""
echo "  Ejecuta 'stress' para abrir la interfaz gráfica."
echo ""
