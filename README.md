# Stress

Herramienta de pruebas de carga con interfaz gráfica.  
Compatible con macOS, Linux y Windows.

---

## Plataformas y arquitecturas soportadas

| Sistema operativo | Arquitectura          | Procesador                        |
|-------------------|-----------------------|-----------------------------------|
| macOS             | ARM64 (Apple Silicon) | Apple M1, M2, M3, M4 y superiores |
| macOS             | x86_64 (Intel)        | Intel Core (2010 en adelante)     |
| Linux             | x86_64                | AMD64 / Intel 64-bit              |
| Windows           | x86_64                | AMD64 / Intel 64-bit              |

> **Nota:** En macOS, los binarios son nativos para cada arquitectura (no se usa Rosetta 2). Descarga el binario correcto según tu procesador o compila desde el código fuente.

---

## Instalación

### macOS / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/Guntzx/stress/main/install.sh | bash
```

### Windows — PowerShell

```powershell
irm https://raw.githubusercontent.com/Guntzx/stress/main/install.ps1 | iex
```

### Windows — Git Bash / WSL

```bash
curl -fsSL https://raw.githubusercontent.com/Guntzx/stress/main/install.sh | bash
```

> El instalador detecta tu sistema, instala Rust si es necesario, compila la app y la añade al PATH.

---

## Actualización

Una vez instalado, actualiza directamente desde la terminal:

```bash
stress update
```

Descarga el binario más reciente desde GitHub Releases y lo reemplaza automáticamente.

---

## Desinstalación

```bash
stress uninstall
```

Elimina el binario del sistema y limpia la entrada del PATH.

---

## Compilar manualmente (desarrolladores)

**Requisitos:** [Rust 1.88+](https://rustup.rs/) y Git.

```bash
git clone https://github.com/Guntzx/stress.git
cd stress
cargo build --release
```

Ejecutables generados:

| SO      | Ruta                          |
|---------|-------------------------------|
| macOS / Linux | `./target/release/stress` |
| Windows | `.\target\release\stress.exe` |

---

## Uso

| Comando             | Descripción                              |
|---------------------|------------------------------------------|
| `stress`            | Abre la interfaz gráfica                 |
| `stress update`     | Actualiza al último release              |
| `stress uninstall`  | Desinstala stress del sistema            |
| `stress help`       | Muestra los comandos disponibles         |

---

## Solución de problemas

| Error | Solución |
|-------|----------|
| `Permission denied` | `chmod +x stress` |
| `stress: command not found` | Reinicia la terminal o ejecuta `source ~/.bashrc` |
| `Library not found` (Linux) | `sudo apt-get install libxkbcommon-dev libwayland-dev libfontconfig-dev libssl-dev` |
| GUI no inicia (Linux) | `sudo apt-get install libxkbcommon-dev libwayland-dev libfontconfig-dev` |
| Windows bloquea el ejecutable | Clic derecho → Propiedades → "Desbloquear" |

Para más ayuda, abre un [issue en GitHub](https://github.com/Guntzx/stress/issues).
