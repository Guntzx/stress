# Stress

Herramienta de pruebas de carga con interfaz gráfica.  
Compatible con macOS, Linux y Windows.

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
