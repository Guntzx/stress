#!/bin/bash

set -e

APP_NAME="Stress"
BIN_PATH="target/release/stress"
APP_DIR="${APP_NAME}.app"
ICON_SRC="icon.icns"
ICON_DST="${APP_DIR}/Contents/Resources/icon.icns"

# 1. Compilar el binario en release
cargo build --release

# 2. Crear estructura de la app
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"

# 3. Copiar el binario
cp "$BIN_PATH" "$APP_DIR/Contents/MacOS/$APP_NAME"
chmod +x "$APP_DIR/Contents/MacOS/$APP_NAME"

# 4. Crear Info.plist
cat > "$APP_DIR/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>$APP_NAME</string>
    <key>CFBundleExecutable</key>
    <string>$APP_NAME</string>
    <key>CFBundleIdentifier</key>
    <string>com.tuempresa.stress</string>
    <key>CFBundleVersion</key>
    <string>1.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleIconFile</key>
    <string>icon.icns</string>
</dict>
</plist>
EOF

# 5. Copiar icono si existe
if [ -f "$ICON_SRC" ]; then
    cp "$ICON_SRC" "$ICON_DST"
    echo "Icono copiado."
else
    echo "No se encontró icon.icns, la app usará el icono por defecto de macOS."
fi

# 6. Mensaje final
cat <<EOM

¡Listo! La app se ha creado en:
  $APP_DIR

Puedes abrirla con:
  open "$APP_DIR"

Si quieres distribuirla, puedes comprimir la carpeta $APP_DIR o crear un .dmg/.pkg.
EOM 