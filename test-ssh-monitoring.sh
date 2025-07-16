#!/bin/bash

# Script para probar el monitoreo SSH sin servidor real
# Simula las respuestas que esperaría la aplicación

echo "🧪 Probando monitoreo SSH..."

# Simular servidor SSH que responde con métricas
nc -l -p 2223 | while read line; do
    case "$line" in
        "CPU")
            echo "$(shuf -i 10-90 -n 1)"
            ;;
        "MEMORY")
            echo "$(shuf -i 20-80 -n 1)"
            ;;
        "DISK")
            echo "$(shuf -i 30-70 -n 1)"
            ;;
        "LOAD")
            echo "$(shuf -i 1-5 -n 1).$(shuf -i 0-99 -n 1)"
            ;;
        *)
            echo "Comando no reconocido: $line"
            ;;
    esac
done &

echo "✅ Servidor simulado iniciado en puerto 2223"
echo "📋 Usar en la aplicación:"
echo "   Host: localhost"
echo "   Puerto: 2223"
echo "   Usuario: test"
echo "   Contraseña: test123" 