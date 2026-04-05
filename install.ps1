# Instalador de Stress para Windows (PowerShell)
# Uso: irm https://raw.githubusercontent.com/Guntzx/stress/main/install.ps1 | iex
#   o: .\install.ps1  (si ya tienes el repositorio clonado)

$ErrorActionPreference = "Stop"

function Write-Info    { Write-Host "[INFO]  $args" -ForegroundColor Cyan }
function Write-Ok      { Write-Host "[OK]    $args" -ForegroundColor Green }
function Write-Warning { Write-Host "[WARN]  $args" -ForegroundColor Yellow }
function Write-Fail    { Write-Host "[ERROR] $args" -ForegroundColor Red; exit 1 }

Write-Host ""
Write-Host "  Instalador de Stress" -ForegroundColor Magenta
Write-Host "  ====================" -ForegroundColor Magenta
Write-Host ""

# ── Instalar Rust si falta ────────────────────────────────────────
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Info "Rust no encontrado. Instalando..."
    $rustupUrl = "https://win.rustup.rs/x86_64"
    $rustupExe = "$env:TEMP\rustup-init.exe"
    Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupExe
    & $rustupExe -y --no-modify-path
    Remove-Item $rustupExe -Force
    # Refrescar PATH
    $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
    Write-Ok "Rust instalado: $(cargo --version)"
} else {
    Write-Ok "Rust: $(cargo --version)"
}

# ── Obtener el código fuente ──────────────────────────────────────
if (-not (Test-Path "Cargo.toml")) {
    if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
        Write-Fail "Git no encontrado. Instálalo desde https://git-scm.com y vuelve a intentarlo."
    }
    Write-Info "Clonando repositorio..."
    git clone https://github.com/Guntzx/stress.git
    Set-Location stress
}

# ── Compilar ──────────────────────────────────────────────────────
Write-Info "Compilando (puede tardar unos minutos la primera vez)..."
cargo build --release
if ($LASTEXITCODE -ne 0) { Write-Fail "Error durante la compilación." }
Write-Ok "Compilación completada"

# ── Instalar ──────────────────────────────────────────────────────
$binary  = ".\target\release\stress.exe"
$destDir = "$env:USERPROFILE\.local\bin"
New-Item -ItemType Directory -Force -Path $destDir | Out-Null
Copy-Item $binary "$destDir\stress.exe" -Force
Write-Ok "Instalado en $destDir\stress.exe"

# Agregar al PATH de usuario si aún no está
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*$destDir*") {
    [Environment]::SetEnvironmentVariable("PATH", "$destDir;$userPath", "User")
    $env:PATH = "$destDir;$env:PATH"
    Write-Warning "PATH actualizado. Reinicia tu terminal para aplicar los cambios."
}

# ── Verificar ─────────────────────────────────────────────────────
Write-Host ""
Write-Ok "Instalacion completada."
Write-Host ""
Write-Host "  Ejecuta 'stress' para abrir la interfaz grafica."
Write-Host ""
