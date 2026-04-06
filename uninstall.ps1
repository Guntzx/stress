# Desinstalador de Stress para Windows (PowerShell)
# Uso: irm https://raw.githubusercontent.com/Guntzx/stress/main/uninstall.ps1 | iex
#   o: .\uninstall.ps1  (si ya tienes el repositorio clonado)

$ErrorActionPreference = "Stop"

function Write-Info    { Write-Host "[INFO]  $args" -ForegroundColor Cyan }
function Write-Ok      { Write-Host "[OK]    $args" -ForegroundColor Green }
function Write-Warning { Write-Host "[WARN]  $args" -ForegroundColor Yellow }
function Write-Fail    { Write-Host "[ERROR] $args" -ForegroundColor Red; exit 1 }

Write-Host ""
Write-Host "  Desinstalador de Stress" -ForegroundColor Magenta
Write-Host "  =======================" -ForegroundColor Magenta
Write-Host ""

$destDir  = "$env:USERPROFILE\.local\bin"
$binary   = "$destDir\stress.exe"

# ── Eliminar binario ──────────────────────────────────────────────
if (-not (Test-Path $binary)) {
    Write-Warning "No se encontró stress en $binary. Puede que ya esté desinstalado."
    exit 0
}

Write-Info "Eliminando $binary..."
Remove-Item $binary -Force
Write-Ok "stress.exe eliminado de $destDir"

# ── Limpiar PATH de usuario ───────────────────────────────────────
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -like "*$destDir*") {
    $newPath = ($userPath -split ";" | Where-Object { $_ -ne $destDir }) -join ";"
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    $env:PATH = ($env:PATH -split ";" | Where-Object { $_ -ne $destDir }) -join ";"
    Write-Ok "Entrada eliminada del PATH de usuario."
}

Write-Host ""
Write-Ok "Desinstalacion completada."
Write-Host ""
