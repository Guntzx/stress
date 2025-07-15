# 🚀 Guía de Inicio Rápido - Test Stress

> **Nota:**
> - En **macOS** se recomienda usar el script `create_app.sh` para obtener una app nativa (`Stress.app`).
> - En **Linux** y **Windows** puedes compilar con `cargo build --release` y ejecutar el binario generado.

## ⚡ Configuración en 5 minutos

### 🚀 Opción 1: Instalador Universal (Recomendado)

**Una sola línea para todo:**

```bash
curl -fsSL https://raw.githubusercontent.com/Guntzx/stress/main/install_universal.sh | bash
```

**¡Listo!** El instalador detecta automáticamente tu sistema y hace todo por ti.

---

### Ejemplo rápido por sistema operativo

#### macOS
```sh
./create_app.sh
open "Stress.app"
```

#### Linux
```sh
cargo build --release
./target/release/stress --gui
```

#### Windows
```cmd
cargo build --release
.target\release\stress.exe --gui
```

---

### 🔧 Opción 2: Instalación Manual

#### 1. Instalar Rust (si no está instalado)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

#### 2. Configurar el proyecto
```bash
./setup.sh
```

#### 3. Editar credenciales
```bash
# Editar archivo de QA
nano .env.qa

# Editar archivo de Producción
nano .env.prod
```

#### 4. Compilar la aplicación
```bash
./build.sh
```

#### 5. ¡Listo! Ejecutar la aplicación
```bash
# Interfaz gráfica (recomendado)
./target/release/stress --gui

# O línea de comandos
./target/release/stress --help
```

## 🎯 Ejemplos Rápidos

### Prueba Rápida de Login
```bash
./target/release/stress login -i 5 -e qa -c 2 -w 1
```

### Prueba Completa Básica
```bash
./target/release/stress full -i 10 -e qa -o stgo -d vina --date 31-12-2023 --service-id 22337601 -c 5 -w 1
```

### Prueba de Carga Intensa
```bash
./target/release/stress full -i 100 -e qa -o stgo -d vina --date 31-12-2023 --service-id 22337601 -c 10 -w 2
```

## 📋 Parámetros Principales

| Parámetro | Descripción | Ejemplo |
|-----------|-------------|---------|
| `-i, --iterations` | Número de iteraciones | `-i 100` |
| `-e, --environment` | Entorno (qa/prod) | `-e qa` |
| `-o, --origin` | Ciudad de origen | `-o stgo` |
| `-d, --destiny` | Ciudad de destino | `-d vina` |
| `--date` | Fecha (DD-MM-YYYY) | `--date 31-12-2023` |
| `--service-id` | ID del servicio | `--service-id 22337601` |
| `-c, --concurrent` | Peticiones simultáneas | `-c 10` |
| `-w, --wait-time` | Tiempo de espera (segundos) | `-w 2` |

## 🔄 Migración desde Bash

### Comando Bash Original
```bash
./execution_test.sh 1 qa stgo vina 31-12-2023 22337601 10 1
```

### Comando Rust Equivalente
```bash
./target/release/stress full -i 1 -e qa -o stgo -d vina --date 31-12-2023 --service-id 22337601 -c 10 -w 1
```

## 🎨 Interfaz Gráfica

La interfaz gráfica incluye:

- **Panel de Configuración**: Entorno, iteraciones, concurrencia
- **Panel de Parámetros**: Origen, destino, fecha, ID de servicio
- **Panel de Controles**: Botones para ejecutar pruebas
- **Panel de Resultados**: Estadísticas en tiempo real
- **Panel de Logs**: Registro detallado

## 📊 Resultados

Los resultados se guardan en:
```
resultados_test_qa_18022025_160000/
├── get_tokens.txt              # Tokens obtenidos
├── stats_for_excel.txt         # Estadísticas
├── login_results_*.json        # Resultados detallados
├── services_results_*.json
├── seatmap_results_*.json
└── excels/
    └── reporte_*.xlsx          # Reporte Excel
```

## 🆘 Solución de Problemas

### Error: "Rust no está instalado"
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Error: "Archivo de configuración no encontrado"
```bash
./setup.sh
```

### Error: "No se encontraron tokens"
```bash
# Ejecutar primero login
./target/release/stress login -i 1 -e qa
```

### Error: "Timeout en peticiones"
```bash
# Reducir concurrencia o aumentar tiempo de espera
./target/release/stress login -i 10 -e qa -c 1 -w 5
```

## 📚 Documentación Completa

- **README_RUST.md**: Documentación detallada
- **COMPARISON.md**: Comparación con scripts bash
- **migrate_examples.sh**: Ejemplos de migración

## 🎉 ¡Listo!

Ya tienes todo configurado para usar la nueva aplicación de pruebas de carga en Rust. ¡Disfruta del mejor rendimiento y la interfaz moderna! 