#!/bin/bash

# Script para crear un paquete .pkg para macOS
set -e

# Configuración
APP_NAME="Test Stress"
APP_VERSION="1.0.0"
BUNDLE_ID="com.tuempresa.test-stress"
INSTALL_DIR="/usr/local/bin"
PACKAGE_NAME="test-stress-${APP_VERSION}.pkg"

echo "🚀 Creando paquete .pkg para macOS..."

# Crear directorio temporal
TEMP_DIR=$(mktemp -d)
echo "📁 Directorio temporal: $TEMP_DIR"

# Crear estructura del paquete
mkdir -p "$TEMP_DIR/root$INSTALL_DIR"
mkdir -p "$TEMP_DIR/scripts"

# Copiar ejecutable
if [[ "$(uname -m)" == "arm64" ]]; then
    cp releases/test-stress-macos-arm64 "$TEMP_DIR/root$INSTALL_DIR/test-stress"
else
    cp releases/test-stress-macos-intel "$TEMP_DIR/root$INSTALL_DIR/test-stress"
fi

chmod +x "$TEMP_DIR/root$INSTALL_DIR/test-stress"

# Crear script de postinstall
cat > "$TEMP_DIR/scripts/postinstall" << 'EOF'
#!/bin/bash

# Crear directorios de configuración
mkdir -p "$HOME/.test-stress/configs"
mkdir -p "$HOME/.test-stress/results"
mkdir -p "$HOME/.test-stress/logs"

# Establecer permisos
chmod 755 /usr/local/bin/test-stress

echo "✅ Test Stress instalado correctamente"
echo "🎯 Para usar: test-stress --gui"
echo "📁 Configuraciones en: $HOME/.test-stress/"
EOF

chmod +x "$TEMP_DIR/scripts/postinstall"

# Crear archivo de distribución
cat > "$TEMP_DIR/Distribution" << EOF
<?xml version="1.0" encoding="utf-8"?>
<installer-gui-script minSpecVersion="1">
    <title>${APP_NAME}</title>
    <organization>${BUNDLE_ID}</organization>
    <domains enable_localSystem="true"/>
    <options customize="never" require-scripts="true" rootVolumeOnly="true"/>
    <pkg-ref id="${BUNDLE_ID}"/>
    <choices-outline>
        <line choice="${BUNDLE_ID}"/>
    </choices-outline>
    <choice id="${BUNDLE_ID}" title="${APP_NAME}">
        <pkg-ref id="${BUNDLE_ID}"/>
    </choice>
    <pkg-ref id="${BUNDLE_ID}" version="${APP_VERSION}" onConclusion="none">test-stress.pkg</pkg-ref>
</installer-gui-script>
EOF

# Crear paquete de componentes
pkgbuild --root "$TEMP_DIR/root" \
         --scripts "$TEMP_DIR/scripts" \
         --identifier "$BUNDLE_ID" \
         --version "$APP_VERSION" \
         --install-location "/" \
         "$TEMP_DIR/test-stress.pkg"

# Crear paquete de distribución
productbuild --distribution "$TEMP_DIR/Distribution" \
             --resources . \
             --package-path "$TEMP_DIR" \
             "$PACKAGE_NAME"

# Limpiar
rm -rf "$TEMP_DIR"

echo "✅ Paquete creado: $PACKAGE_NAME"
echo "📦 Para instalar: sudo installer -pkg $PACKAGE_NAME -target /" 