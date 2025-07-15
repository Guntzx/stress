#!/bin/bash

# Script para compilar test-stress para múltiples plataformas
echo "🚀 Compilando test-stress para múltiples plataformas..."

# Crear directorio de releases
mkdir -p releases

# Compilar para macOS ARM64 (Apple Silicon)
echo "📦 Compilando para macOS ARM64..."
cargo build --release --target aarch64-apple-darwin
if [ $? -eq 0 ]; then
    cp target/aarch64-apple-darwin/release/test-stress releases/stress-macos-arm64
    echo "✅ macOS ARM64 compilado exitosamente"
else
    echo "❌ Error compilando para macOS ARM64"
fi

# Compilar para macOS Intel
echo "📦 Compilando para macOS Intel..."
cargo build --release --target x86_64-apple-darwin
if [ $? -eq 0 ]; then
    cp target/x86_64-apple-darwin/release/test-stress releases/stress-macos-intel
    echo "✅ macOS Intel compilado exitosamente"
else
    echo "❌ Error compilando para macOS Intel"
fi

# Compilar para Linux
echo "📦 Compilando para Linux..."
cargo build --release --target x86_64-unknown-linux-gnu
if [ $? -eq 0 ]; then
    cp target/x86_64-unknown-linux-gnu/release/test-stress releases/test-stress-linux
    echo "✅ Linux compilado exitosamente"
else
    echo "❌ Error compilando para Linux"
fi

# Compilar para Windows
echo "📦 Compilando para Windows..."
cargo build --release --target x86_64-pc-windows-msvc
if [ $? -eq 0 ]; then
    cp target/x86_64-pc-windows-msvc/release/test-stress.exe releases/test-stress-windows.exe
    echo "✅ Windows compilado exitosamente"
else
    echo "❌ Error compilando para Windows"
fi

echo ""
echo "🎉 Compilación completada!"
echo "📁 Los ejecutables están en el directorio 'releases/':"
ls -la releases/
echo ""
echo "📋 Instrucciones de uso:"
echo "  • macOS ARM64: ./stress-macos-arm64 --gui"
echo "  • macOS Intel:  ./stress-macos-intel --gui"
echo "  • Linux:        ./test-stress-linux --gui"
echo "  • Windows:      test-stress-windows.exe --gui" 