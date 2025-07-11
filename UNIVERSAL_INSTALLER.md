# Instalador Universal

> **Nota:**
> - Para **macOS** se recomienda usar el script `create_app.sh` para obtener una app nativa (`Stress.app`).
> - Para **Linux** y **Windows** puedes compilar con `cargo build --release` y ejecutar el binario generado.

---

## Instalación universal (funciona en la mayoría de los sistemas)

### Opción rápida (bash):
```sh
curl -fsSL https://raw.githubusercontent.com/Guntzx/stress/main/install_universal.sh | bash
```

### Opción rápida (PowerShell en Windows):
```powershell
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/Guntzx/stress/main/install_universal.ps1" -OutFile "install.ps1"; .\install.ps1
```

### Opción rápida (CMD en Windows):
```cmd
install_universal.bat
```

---

## Recomendaciones por sistema

- **macOS:**
  - Ejecuta `./create_app.sh` para crear `Stress.app` y ábrela con doble clic o desde el Finder.
- **Linux:**
  - Compila con `cargo build --release` y ejecuta `./target/release/stress`.
- **Windows:**
  - Compila con `cargo build --release` y ejecuta `target\release\stress.exe`.

---

Consulta el `README.md` para más detalles y opciones avanzadas. 