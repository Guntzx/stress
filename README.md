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

### macOS / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/Guntzx/stress/main/update.sh | bash
```

### Windows — PowerShell

```powershell
irm https://raw.githubusercontent.com/Guntzx/stress/main/update.ps1 | iex
```

### Windows — Git Bash / WSL

```bash
curl -fsSL https://raw.githubusercontent.com/Guntzx/stress/main/update.sh | bash
```

> El actualizador descarga los últimos cambios, recompila y reemplaza el binario instalado.

---

## Desinstalación

### macOS / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/Guntzx/stress/main/uninstall.sh | bash
```

### Windows — PowerShell

```powershell
irm https://raw.githubusercontent.com/Guntzx/stress/main/uninstall.ps1 | iex
```

### Windows — Git Bash / WSL

```bash
curl -fsSL https://raw.githubusercontent.com/Guntzx/stress/main/uninstall.sh | bash
```

> El desinstalador elimina el binario y limpia la entrada del PATH.

---

## Compilar manualmente (desarrolladores)

**Requisitos:** [Rust 1.70+](https://rustup.rs/) y Git.

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

Ejecuta `stress` (o doble clic en el ejecutable) para abrir la interfaz gráfica.

---

## Solución de problemas

| Error | Solución |
|-------|----------|
| `Permission denied` | `chmod +x stress` |
| `stress: command not found` | Reinicia la terminal o ejecuta `source ~/.bashrc` |
| `Library not found` (Linux) | `sudo apt-get install libssl-dev libgtk-3-dev libwebkit2gtk-4.0-dev` |
| GUI no inicia (Linux) | `sudo apt-get install libgtk-3-dev libwebkit2gtk-4.0-dev` |
| Windows bloquea el ejecutable | Clic derecho → Propiedades → "Desbloquear" |

Para más ayuda, abre un [issue en GitHub](https://github.com/Guntzx/stress/issues).
