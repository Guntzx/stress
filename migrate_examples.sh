#!/bin/bash

# Script de ejemplos de migración de scripts bash a Rust
# Este archivo muestra cómo convertir comandos bash a comandos Rust

echo "=== Ejemplos de Migración: Scripts Bash → Rust ==="
echo ""

echo "1. COMANDO BASH ORIGINAL:"
echo "./execution_test.sh 1 qa stgo vina 31-12-2023 22337601 10 1"
echo ""
echo "COMANDO RUST EQUIVALENTE:"
echo "cargo run -- full --iterations 1 --environment qa --origin stgo --destiny vina --date 31-12-2023 --service-id 22337601 --concurrent 10 --wait-time 1"
echo ""

echo "2. COMANDO BASH ORIGINAL:"
echo "./execution_test.sh 100 prod stgo mott 31-12-2023 26847102 50 2"
echo ""
echo "COMANDO RUST EQUIVALENTE:"
echo "cargo run -- full --iterations 100 --environment prod --origin stgo --destiny mott --date 31-12-2023 --service-id 26847102 --concurrent 50 --wait-time 2"
echo ""

echo "3. SOLO LOGIN (Bash no tiene comando directo):"
echo "cargo run -- login --iterations 10 --environment qa --concurrent 5 --wait-time 1"
echo ""

echo "4. SOLO SERVICES (requiere tokens previos):"
echo "cargo run -- services --iterations 10 --environment qa --origin stgo --destiny vina --date 31-12-2023 --service-id 22337601 --concurrent 5 --wait-time 1"
echo ""

echo "5. SOLO SEATMAP (requiere tokens previos):"
echo "cargo run -- seatmap --iterations 10 --environment qa --origin stgo --destiny vina --date 31-12-2023 --service-id 22337601 --concurrent 5 --wait-time 1"
echo ""

echo "6. GENERAR REPORTE EXCEL:"
echo "cargo run -- report --results-dir resultados_test_qa_18022025_160000"
echo ""

echo "7. INTERFAZ GRÁFICA (NUEVA FUNCIONALIDAD):"
echo "cargo run -- --gui"
echo ""

echo "=== VENTAJAS DE LA MIGRACIÓN ==="
echo "✅ Mejor rendimiento (Rust vs Bash)"
echo "✅ Concurrencia nativa asíncrona"
echo "✅ Interfaz gráfica moderna"
echo "✅ Manejo robusto de errores"
echo "✅ Reportes Excel automáticos"
echo "✅ Logging estructurado"
echo "✅ Validación de configuración"
echo "✅ Binario único sin dependencias"
echo ""

echo "=== PASOS PARA MIGRAR ==="
echo "1. Compilar la aplicación: cargo build --release"
echo "2. Crear archivos de entorno (.env.qa, .env.prod)"
echo "3. Actualizar contraseñas en los archivos .env"
echo "4. Reemplazar comandos bash por comandos Rust"
echo "5. Usar la interfaz gráfica para configuraciones complejas"
echo ""

echo "=== EJEMPLOS DE USO RÁPIDO ==="
echo "# Prueba rápida de login"
echo "cargo run -- login -i 5 -e qa -c 2 -w 1"
echo ""
echo "# Prueba de carga completa"
echo "cargo run -- full -i 100 -e qa -o stgo -d vina --date 31-12-2023 --service-id 22337601 -c 10 -w 2"
echo ""
echo "# Interfaz gráfica"
echo "cargo run -- --gui" 