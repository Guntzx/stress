#!/bin/bash

# Script de configuración inicial para Test Stress
# Este script configura el entorno para usar la aplicación

set -e

echo "⚙️  Test Stress - Configuración Inicial"
echo "======================================"

# Verificar si estamos en el directorio correcto
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: No se encontró Cargo.toml"
    echo "Asegúrate de estar en el directorio raíz del proyecto"
    exit 1
fi

# Crear archivos de entorno si no existen
echo "📝 Configurando archivos de entorno..."

if [ ! -f ".env.qa" ]; then
    if [ -f "env.qa.example" ]; then
        cp env.qa.example .env.qa
        echo "✅ Archivo .env.qa creado desde env.qa.example"
        echo "⚠️  IMPORTANTE: Actualiza la contraseña en .env.qa"
    else
        echo "❌ No se encontró env.qa.example"
        exit 1
    fi
else
    echo "✅ Archivo .env.qa ya existe"
fi

if [ ! -f ".env.prod" ]; then
    if [ -f "env.prod.example" ]; then
        cp env.prod.example .env.prod
        echo "✅ Archivo .env.prod creado desde env.prod.example"
        echo "⚠️  IMPORTANTE: Actualiza la contraseña en .env.prod"
    else
        echo "❌ No se encontró env.prod.example"
        exit 1
    fi
else
    echo "✅ Archivo .env.prod ya existe"
fi

# Verificar si Rust está instalado
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Rust no está instalado"
    echo "Por favor, instala Rust desde https://rustup.rs/"
    echo ""
    echo "Comandos de instalación:"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo "source ~/.cargo/env"
    exit 1
fi

echo "✅ Rust detectado: $(cargo --version)"

# Verificar dependencias
echo "🔍 Verificando dependencias..."
cargo check --quiet

if [ $? -eq 0 ]; then
    echo "✅ Todas las dependencias están disponibles"
else
    echo "❌ Error verificando dependencias"
    echo "Ejecuta: cargo build"
    exit 1
fi

echo ""
echo "🎉 Configuración completada exitosamente!"
echo ""
echo "📋 Próximos pasos:"
echo "1. Edita .env.qa y .env.prod con las credenciales correctas"
echo "2. Compila la aplicación: ./build.sh"
echo "3. Ejecuta la interfaz gráfica: cargo run -- --gui"
echo ""
echo "📖 Documentación: README_RUST.md"
echo "🔄 Ejemplos de migración: ./migrate_examples.sh" 