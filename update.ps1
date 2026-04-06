# Actualizador de Stress para Windows (PowerShell)
# Uso: irm https://raw.githubusercontent.com/Guntzx/stress/main/update.ps1 | iex
#   o: .\update.ps1  (si ya tienes el repositorio clonado)

$ErrorActionPreference = "Stop"

function Write-Info    { Write-Host "[INFO]  $args" -ForegroundColor Cyan }
function Write-Ok      { Write-Host "[OK]    $args" -ForegroundColor Green }
function Write-Warning { Write-Host "[WARN]  $args" -ForegroundColor Yellow }
function Write-Fail    { Write-Host "[ERROR] $args" -ForegroundColor Red; exit 1 }

Write-Host ""
Write-Host "  Actualizador de Stress" -ForegroundColor Magenta
Write-Host "  ======================" -ForegroundColor Magenta
Write-Host ""

# ── Verificar dependencias ────────────────────────────────────────
if (-not (Get-Command git   -ErrorAction SilentlyContinue)) { Write-Fail "Git no encontrado. Instálalo desde https://git-scm.com y vuelve a intentarlo." }
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { Write-Fail "Rust/Cargo no encontrado. Instálalo desde https://rustup.rs y vuelve a intentarlo." }

# ── Obtener o actualizar el repositorio ──────────────────────────
$repoUrl = "https://github.com/Guntzx/stress.git"
$repoDir = "$env:USERPROFILE\.local\share\stress"

if (Test-Path "$repoDir\.git") {
    Write-Info "Actualizando repositorio en $repoDir..."
    git -C $repoDir pull --ff-only
    Set-Location $repoDir
} elseif (Test-Path "Cargo.toml") {
    Write-Info "Repositorio local detectado. Actualizando..."
    git pull --ff-only
} else {
    Write-Info "Clonando repositorio en $repoDir..."
    New-Item -ItemType Directory -Force -Path (Split-Path $repoDir) | Out-Null
    git clone $repoUrl $repoDir
    Set-Location $repoDir
}

# ── Compilar ──────────────────────────────────────────────────────
Write-Info "Compilando nueva versión..."
cargo build --release
if ($LASTEXITCODE -ne 0) { Write-Fail "Error durante la compilación." }
Write-Ok "Compilación completada"

# ── Instalar ──────────────────────────────────────────────────────
$binary  = ".\target\release\stress.exe"
$destDir = "$env:USERPROFILE\.local\bin"
New-Item -ItemType Directory -Force -Path $destDir | Out-Null
Copy-Item $binary "$destDir\stress.exe" -Force
Write-Ok "Actualizado en $destDir\stress.exe"

# Asegurar que el PATH sigue configurado
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*$destDir*") {
    [Environment]::SetEnvironmentVariable("PATH", "$destDir;$userPath", "User")
    $env:PATH = "$destDir;$env:PATH"
    Write-Warning "PATH actualizado. Reinicia tu terminal para aplicar los cambios."
}

Write-Host ""
Write-Ok "Actualizacion completada."
Write-Host ""
Write-Host "  Ejecuta 'stress' para abrir la interfaz grafica."
Write-Host ""
