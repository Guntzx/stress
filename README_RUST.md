# Test Stress - Aplicación de Pruebas de Carga en Rust

Esta es una aplicación moderna en Rust que reemplaza los scripts bash originales para ejecutar pruebas de carga. Incluye tanto una interfaz de línea de comandos como una interfaz gráfica amigable.

## Características

- ✅ **Interfaz de línea de comandos** completa
- ✅ **Interfaz gráfica** moderna y fácil de usar
- ✅ **Múltiples tipos de pruebas**: Login, Services, Seatmap, Itineraries
- ✅ **Gestión de entornos**: QA y Producción
- ✅ **Concurrencia configurable** para pruebas de carga
- ✅ **Generación automática de reportes Excel**
- ✅ **Logging detallado** y estadísticas en tiempo real
- ✅ **Manejo robusto de errores**
- ✅ **Progreso visual** durante las pruebas

## Instalación

### Prerrequisitos

- Rust 1.70+ (instalar desde https://rustup.rs/)
- Git

### Compilación

```bash
# Clonar el repositorio
git clone <repository-url>
cd stress

# Compilar en modo release (recomendado para producción)
cargo build --release

# O compilar en modo debug para desarrollo
cargo build
```

## Configuración

## Uso

### Interfaz Gráfica (Recomendado)

```bash
# Ejecutar la interfaz gráfica
cargo run -- --gui

# O desde el ejecutable compilado
./target/release/test-stress --gui
```

La interfaz gráfica incluye:
- **Panel de Configuración**: Entorno, iteraciones, concurrencia, tiempo de espera
- **Panel de Parámetros**: Origen, destino, fecha, ID de servicio
- **Panel de Controles**: Botones para ejecutar diferentes tipos de pruebas
- **Panel de Resultados**: Estadísticas en tiempo real
- **Panel de Logs**: Registro detallado de la ejecución

### Interfaz de Línea de Comandos

#### Prueba de Login
```bash
cargo run -- login --iterations 10 --environment qa --concurrent 5 --wait-time 1
```

#### Prueba de Services
```bash
cargo run -- services \
  --iterations 10 \
  --environment qa \
  --origin stgo \
  --destiny vina \
  --date 31-12-2023 \
  --service-id 22337601 \
  --concurrent 5 \
  --wait-time 1
```

#### Prueba de Seatmap
```bash
cargo run -- seatmap \
  --iterations 10 \
  --environment qa \
  --origin stgo \
  --destiny vina \
  --date 31-12-2023 \
  --service-id 22337601 \
  --concurrent 5 \
  --wait-time 1
```

#### Prueba Completa (Login + Services + Seatmap)
```bash
cargo run -- full \
  --iterations 10 \
  --environment qa \
  --origin stgo \
  --destiny vina \
  --date 31-12-2023 \
  --service-id 22337601 \
  --concurrent 5 \
  --wait-time 1
```

#### Generar Reporte Excel
```bash
cargo run -- report --results-dir resultados_test_qa_18022025_160000
```

### Parámetros de Línea de Comandos

| Parámetro | Descripción | Valor por defecto |
|-----------|-------------|-------------------|
| `--iterations, -i` | Número de iteraciones | 10 |
| `--environment, -e` | Entorno (qa/prod) | Requerido |
| `--origin, -o` | Ciudad de origen | Requerido |
| `--destiny, -d` | Ciudad de destino | Requerido |
| `--date` | Fecha del viaje (DD-MM-YYYY) | Requerido |
| `--service-id` | ID del servicio | Requerido |
| `--concurrent, -c` | Peticiones simultáneas | 1 |
| `--wait-time, -w` | Tiempo de espera entre lotes (segundos) | 1 |
| `--output-dir` | Directorio de salida personalizado | Auto-generado |

## Estructura de Archivos de Salida

La aplicación crea directorios con el formato `resultados_test_{entorno}_{timestamp}` que contienen:

```
resultados_test_qa_18022025_160000/
├── get_tokens.txt              # Tokens obtenidos del login
├── stats_for_excel.txt         # Estadísticas para Excel
├── login_results_18022025-16:00.json
├── services_results_18022025-16:00.json
├── seatmap_results_18022025-16:00.json
└── excels/
    └── reporte_18022025_160000.xlsx
```

## Ejemplos de Uso

### Ejemplo 1: Prueba Rápida de Login
```bash
# Prueba simple de login con 5 iteraciones
cargo run -- login -i 5 -e qa -c 2 -w 1
```
