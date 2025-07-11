#!/bin/bash

# Script para crear una aplicación .app de macOS
set -e

# Configuración
APP_NAME="Test Stress"
APP_VERSION="1.0.0"
BUNDLE_ID="com.tuempresa.test-stress"
APP_DIR="Test Stress.app"

echo "🚀 Creando aplicación .app para macOS..."

# Crear estructura de la aplicación
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"

# Copiar ejecutable
if [[ "$(uname -m)" == "arm64" ]]; then
    cp releases/test-stress-macos-arm64 "$APP_DIR/Contents/MacOS/test-stress"
else
    cp releases/test-stress-macos-intel "$APP_DIR/Contents/MacOS/test-stress"
fi

chmod +x "$APP_DIR/Contents/MacOS/test-stress"

# Crear Info.plist
cat > "$APP_DIR/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>test-stress</string>
    <key>CFBundleIdentifier</key>
    <string>${BUNDLE_ID}</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleDisplayName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleVersion</key>
    <string>${APP_VERSION}</string>
    <key>CFBundleShortVersionString</key>
    <string>${APP_VERSION}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.developer-tools</string>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key>
            <string>Test Stress Configuration</string>
            <key>CFBundleTypeExtensions</key>
            <array>
                <string>json</string>
            </array>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
        </dict>
    </array>
</dict>
</plist>
EOF

# Crear script de lanzamiento
cat > "$APP_DIR/Contents/MacOS/launcher" << 'EOF'
#!/bin/bash

# Obtener el directorio de la aplicación
APP_DIR="$(cd "$(dirname "$0")/.." && pwd)"
EXECUTABLE="$APP_DIR/MacOS/test-stress"

# Crear directorios de configuración si no existen
mkdir -p "$HOME/.test-stress"/{configs,results,logs}

# Ejecutar la aplicación
exec "$EXECUTABLE" --gui
EOF

chmod +x "$APP_DIR/Contents/MacOS/launcher"

# Actualizar Info.plist para usar el launcher
sed -i '' 's/<string>test-stress<\/string>/<string>launcher<\/string>/' "$APP_DIR/Contents/Info.plist"

echo "✅ Aplicación creada: $APP_DIR"
echo "📦 Para instalar:"
echo "  • Arrastra '$APP_DIR' a la carpeta Aplicaciones"
echo "  • O ejecuta: cp -r '$APP_DIR' /Applications/"
echo ""
echo "🎯 Para usar:"
echo "  • Doble clic en '$APP_DIR'"
echo "  • O desde Spotlight: 'Test Stress'" 