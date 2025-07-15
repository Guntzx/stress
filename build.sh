#!/bin/bash

# Script de construcción para Test Stress
# Este script facilita la compilación y configuración de la aplicación

set -e

echo "🚀 Test Stress - Script de Construcción"
echo "======================================"

# Verificar si Rust está instalado
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Rust no está instalado"
    echo "Por favor, instala Rust desde https://rustup.rs/"
    exit 1
fi

echo "✅ Rust detectado: $(cargo --version)"

# Verificar si estamos en el directorio correcto
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: No se encontró Cargo.toml"
    echo "Asegúrate de estar en el directorio raíz del proyecto"
    exit 1
fi

# Limpiar compilaciones anteriores
echo "🧹 Limpiando compilaciones anteriores..."
cargo clean

# Compilar en modo release
echo "🔨 Compilando en modo release..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Compilación exitosa!"
    echo ""
    echo "📁 Ejecutable creado en: target/release/test-stress"
    echo ""
    echo "🚀 Para ejecutar la aplicación:"
    echo "   ./target/release/test-stress --gui"
    echo ""
    echo "📖 Para ver la ayuda:"
    echo "   ./target/release/test-stress --help"
    echo ""
    echo "⚙️  Configuración:"
    echo "   1. Copia env.qa.example a .env.qa"
    echo "   2. Copia env.prod.example a .env.prod"
    echo "   3. Actualiza las contraseñas en los archivos .env"
    echo ""
    echo "🎯 Ejemplos de uso:"
    echo "   ./target/release/test-stress login -i 10 -e qa -c 5 -w 1"
    echo "   ./target/release/test-stress full -i 100 -e qa -o stgo -d vina --date 31-12-2023 --service-id 22337601 -c 10 -w 2"
else
    echo "❌ Error en la compilación"
    exit 1
fi 