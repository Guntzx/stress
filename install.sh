#!/bin/bash
# Instalador de Stress - macOS, Linux y Windows (Git Bash / WSL)
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

# ── Instalar Rust si falta ─────────────────────────────────────────
if ! command -v cargo &>/dev/null; then
  warn "Rust no encontrado. Instalando..."
  if [[ "$OS" == "windows" ]]; then
    curl -fsSL https://win.rustup.rs/x86_64 -o rustup-init.exe
    ./rustup-init.exe -y --no-modify-path
    rm -f rustup-init.exe
  else
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
  fi
  source "$HOME/.cargo/env" 2>/dev/null || export PATH="$HOME/.cargo/bin:$PATH"
  success "Rust instalado: $(cargo --version)"
else
  success "Rust: $(cargo --version)"
fi

# ── Obtener el código fuente ───────────────────────────────────────
if [ ! -f "Cargo.toml" ]; then
  info "Clonando repositorio..."
  git clone https://github.com/Guntzx/stress.git
  cd stress
fi

# ── Compilar ───────────────────────────────────────────────────────
info "Compilando (esto puede tardar unos minutos la primera vez)..."
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
  success "Instalado en $INSTALL_DIR/stress"

elif [[ "$OS" == "windows" ]]; then
  INSTALL_DIR="$HOME/.local/bin"
  mkdir -p "$INSTALL_DIR"
  cp "$BINARY" "$INSTALL_DIR/stress.exe"
  # Agregar al PATH del perfil si aún no está
  if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "export PATH=\"\$HOME/.local/bin:\$PATH\"" >> "$HOME/.bashrc"
    export PATH="$INSTALL_DIR:$PATH"
    warn "PATH actualizado en ~/.bashrc — reinicia tu terminal o ejecuta: source ~/.bashrc"
  fi
  success "Instalado en $INSTALL_DIR/stress.exe"
fi

# ── Verificar ──────────────────────────────────────────────────────
if command -v stress &>/dev/null || [ -f "$INSTALL_DIR/stress.exe" ]; then
  echo ""
  success "Instalación completada."
  echo ""
  echo "  Ejecuta 'stress' para abrir la interfaz gráfica."
  echo ""
else
  error "No se encontró 'stress' en el PATH tras la instalación."
fi
