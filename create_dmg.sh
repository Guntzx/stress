#!/bin/bash

# Script para crear un instalador DMG de macOS
set -e

# Configuración
APP_NAME="Test Stress"
DMG_NAME="Test Stress Installer"
VOLUME_NAME="Test Stress"
APP_DIR="Test Stress.app"
DMG_FILE="Test Stress Installer.dmg"

echo "🚀 Creando instalador DMG para macOS..."

# Crear la aplicación .app primero
./create_app.sh

# Crear directorio temporal para el DMG
TEMP_DIR=$(mktemp -d)
echo "📁 Directorio temporal: $TEMP_DIR"

# Copiar la aplicación al directorio temporal
cp -r "$APP_DIR" "$TEMP_DIR/"

# Crear enlace a Aplicaciones
ln -s /Applications "$TEMP_DIR/Applications"

# Crear archivo de información
cat > "$TEMP_DIR/README.txt" << 'EOF'
Test Stress - Instalación

Para instalar Test Stress:

1. Arrastra "Test Stress.app" a la carpeta "Applications"
2. Abre Test Stress desde la carpeta Applications o Spotlight

Test Stress se abrirá automáticamente en modo interfaz gráfica.

Para más información, visita: https://github.com/Guntzx/stress
EOF

# Crear DMG
echo "📦 Creando DMG..."
hdiutil create -volname "$VOLUME_NAME" -srcfolder "$TEMP_DIR" -ov -format UDZO "$DMG_FILE"

# Limpiar
rm -rf "$TEMP_DIR"

echo "✅ DMG creado: $DMG_FILE"
echo "📦 Para distribuir:"
echo "  • Envía el archivo '$DMG_FILE'"
echo "  • Los usuarios pueden abrirlo con doble clic"
echo "  • Arrastrar la aplicación a la carpeta Applications" 