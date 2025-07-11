# Instalador de Stress para Windows

## 🚀 Instalación rápida

1. Descarga el archivo `stress-setup.exe` (o compílalo con el script batch)
2. Haz doble clic en el instalador
3. Sigue los pasos del asistente
4. Al finalizar, tendrás accesos directos en el escritorio y menú inicio

## 🖥️ Requisitos
- Windows 10 o superior (64 bits)
- Permisos de administrador para instalar
- NSIS (solo si vas a compilar el instalador)

## 🛠️ Compilar el instalador

1. Compila el binario de la app:
   ```cmd
   cargo build --release --target x86_64-pc-windows-msvc
   copy target\release\stress.exe releases\stress-windows-x64.exe
   ```
2. Ejecuta el script batch:
   ```cmd
   build_installer.bat
   ```
3. El instalador generado será `stress-setup.exe`

## 🧩 ¿Qué hace el instalador?
- Copia el ejecutable a `C:\Program Files\Stress\`
- Crea accesos directos en el escritorio y menú inicio
- Permite desinstalar desde el panel de control

## 🐛 Solución de problemas
- Si Windows bloquea el instalador, haz clic en "Más información" y luego en "Ejecutar de todas formas"
- Si ves errores de permisos, ejecuta el instalador como administrador

## 🗑️ Desinstalación
- Desde el panel de control o menú inicio, selecciona "Desinstalar Stress" 