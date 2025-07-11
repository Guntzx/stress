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

### Archivos de Entorno

La aplicación creará automáticamente los archivos de configuración si no existen:

#### `.env.qa`
```env
URL=https://ms-compra-qas.egt.cl
USERNAME=userKupos
PASS=your_qa_password_here
CANAL_ID=10276
```

#### `.env.prod`
```env
URL=https://ms-compra.egt.cl
USERNAME=userKupos
PASS=your_prod_password_here
CANAL_ID=10276
```

**Importante**: Actualiza las contraseñas en estos archivos antes de usar la aplicación.

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

### Ejemplo 2: Prueba de Carga Completa
```bash
# Prueba completa con alta concurrencia
cargo run -- full \
  -i 100 \
  -e qa \
  -o stgo \
  -d vina \
  --date 31-12-2023 \
  --service-id 22337601 \
  -c 10 \
  -w 2
```

### Ejemplo 3: Prueba de Producción
```bash
# Prueba en entorno de producción
cargo run -- full \
  -i 50 \
  -e prod \
  -o stgo \
  -d mott \
  --date 31-12-2023 \
  --service-id 26847102 \
  -c 5 \
  -w 3
```

## Ventajas sobre los Scripts Bash Originales

1. **Rendimiento**: Rust es significativamente más rápido que bash
2. **Concurrencia**: Manejo nativo de peticiones asíncronas
3. **Interfaz Gráfica**: Experiencia de usuario moderna
4. **Manejo de Errores**: Errores más descriptivos y recuperación automática
5. **Reportes**: Generación automática de Excel sin dependencias externas
6. **Configuración**: Validación automática de configuración
7. **Logging**: Sistema de logging estructurado
8. **Portabilidad**: Binario único sin dependencias externas

## Troubleshooting

### Error: "Archivo de configuración no encontrado"
```bash
# La aplicación creará automáticamente los archivos .env.qa y .env.prod
# Solo necesitas actualizar las contraseñas
```

### Error: "No se encontraron tokens"
```bash
# Ejecuta primero una prueba de login
cargo run -- login -i 1 -e qa
```

### Error: "Timeout en peticiones"
```bash
# Reduce la concurrencia o aumenta el tiempo de espera
cargo run -- login -i 10 -e qa -c 1 -w 5
```

### Error: "Puerto ocupado"
```bash
# La aplicación usa puertos dinámicos, no debería haber conflictos
# Verifica que no haya otras instancias ejecutándose
```

## Desarrollo

### Estructura del Proyecto

```
src/
├── main.rs              # Punto de entrada y CLI
├── models.rs            # Estructuras de datos
├── config.rs            # Gestión de configuración
├── load_test.rs         # Lógica de pruebas de carga
├── cli.rs               # Comandos de línea de comandos
├── gui.rs               # Interfaz gráfica
└── report_generator.rs  # Generación de reportes Excel
```

### Agregar Nuevos Tipos de Prueba

1. Agregar el tipo en `models.rs` (enum `TestType`)
2. Implementar la lógica en `load_test.rs`
3. Agregar comando CLI en `cli.rs`
4. Agregar botón en la GUI en `gui.rs`

### Compilar para Distribución

```bash
# Compilar para Linux
cargo build --release --target x86_64-unknown-linux-gnu

# Compilar para Windows
cargo build --release --target x86_64-pc-windows-msvc

# Compilar para macOS
cargo build --release --target x86_64-apple-darwin
```

## Compilación y ejecución por sistema operativo

### macOS

- Compila y crea la app nativa:
  ```sh
  ./create_app.sh
  ```
  Esto generará `Stress.app` en el directorio actual. Puedes abrirla con:
  ```sh
  open "Stress.app"
  ```
  O arrástrala a la carpeta Aplicaciones.

- Para desarrollo rápido:
  ```sh
  cargo run --release
  ```

### Linux

- Compila el binario:
  ```sh
  cargo build --release
  ```
  Ejecuta con:
  ```sh
  ./target/release/stress
  ```

### Windows

- Compila el binario:
  ```sh
  cargo build --release
  ```
  Ejecuta con:
  ```sh
  .\target\release\stress.exe
  ```

- (Opcional) Usa el instalador NSIS o los scripts incluidos para crear un instalador.

---

## Notas para desarrolladores Rust
- El proyecto usa `eframe/egui` para la GUI y es multiplataforma.
- Puedes modificar y probar la interfaz gráfica con:
  ```sh
  cargo run -- --gui
  ```
- El binario release es el que se debe empaquetar para usuarios finales.
- Para crear la app de macOS, usa el script `create_app.sh`.

---

Consulta el `README.md` principal para instrucciones de usuario final y detalles de instalación por sistema operativo.

## Contribución

1. Fork el proyecto
2. Crea una rama para tu feature (`git checkout -b feature/AmazingFeature`)
3. Commit tus cambios (`git commit -m 'Add some AmazingFeature'`)
4. Push a la rama (`git push origin feature/AmazingFeature`)
5. Abre un Pull Request

## Licencia

Este proyecto está bajo la misma licencia que el proyecto original. 