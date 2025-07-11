@echo off
REM Script para compilar el instalador de Stress en Windows

REM Verificar que NSIS esté instalado
where makensis >nul 2>nul
if errorlevel 1 (
    echo "NSIS (makensis) no encontrado. Instala NSIS y agrega makensis al PATH."
    exit /b 1
)

REM Verificar que el ejecutable existe
if not exist "releases\stress-windows-x64.exe" (
    echo "No se encontró releases\stress-windows-x64.exe. Compila el binario primero."
    exit /b 1
)

REM Compilar el instalador
makensis installer.nsi
if errorlevel 1 (
    echo "Error al compilar el instalador."
    exit /b 1
)

echo "Instalador generado: stress-setup.exe" 