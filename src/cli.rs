use std::env;
use std::fs;
use std::path::PathBuf;

// ── Ruta de instalación estándar ─────────────────────────────────────────────

fn get_install_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        dirs::home_dir()
            .expect("No se pudo determinar el directorio home")
            .join(".local")
            .join("bin")
            .join("stress.exe")
    }
    #[cfg(not(target_os = "windows"))]
    {
        PathBuf::from("/usr/local/bin/stress")
    }
}

// ── Asset de GitHub Releases según plataforma/arquitectura ───────────────────

fn get_release_asset_name() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    { "test-stress-macos-arm64" }

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    { "test-stress-macos-intel" }

    #[cfg(target_os = "linux")]
    { "test-stress-linux" }

    #[cfg(target_os = "windows")]
    { "test-stress-windows.exe" }
}

// ── Comparación de versiones semánticas ──────────────────────────────────────

fn is_newer_version(current: &str, latest: &str) -> bool {
    let parse = |s: &str| -> (u64, u64, u64) {
        let mut parts = s.split('.').filter_map(|p| p.parse::<u64>().ok());
        (parts.next().unwrap_or(0), parts.next().unwrap_or(0), parts.next().unwrap_or(0))
    };
    parse(latest) > parse(current)
}

// ── stress uninstall ─────────────────────────────────────────────────────────

pub fn uninstall() -> Result<(), Box<dyn std::error::Error>> {
    let install_path = get_install_path();

    println!();
    println!("  Desinstalando stress");
    println!("  ====================");
    println!();

    if !install_path.exists() {
        let current_exe = env::current_exe()?;
        println!("[WARN] No se encontró stress en {}", install_path.display());
        if current_exe != install_path {
            println!("[INFO] Ejecutable actual: {}", current_exe.display());
            println!("[WARN] Elimínalo manualmente si deseas desinstalar.");
        } else {
            println!("[WARN] stress ya parece estar desinstalado.");
        }
        return Ok(());
    }

    println!("[INFO] Se eliminará: {}", install_path.display());
    print!("[INFO] ¿Confirmar? [s/N] ");

    use std::io::{self, BufRead, Write};
    io::stdout().flush()?;
    let stdin = io::stdin();
    let line = stdin.lock().lines().next().unwrap_or(Ok(String::new()))?;
    if !matches!(line.trim().to_lowercase().as_str(), "s" | "si" | "sí" | "y" | "yes") {
        println!("[INFO] Desinstalación cancelada.");
        return Ok(());
    }

    match fs::remove_file(&install_path) {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            #[cfg(unix)]
            {
                println!("[INFO] Permisos insuficientes, reintentando con sudo...");
                let status = std::process::Command::new("sudo")
                    .arg("rm")
                    .arg("-f")
                    .arg(&install_path)
                    .status()?;
                if !status.success() {
                    return Err(format!(
                        "No se pudo eliminar {}. Intenta: sudo rm {}",
                        install_path.display(),
                        install_path.display()
                    ).into());
                }
            }
            #[cfg(not(unix))]
            return Err(e.into());
        }
        Err(e) => return Err(e.into()),
    }

    println!("[OK]   stress eliminado de {}", install_path.display());

    // En Windows, limpiar también el PATH de usuario
    #[cfg(target_os = "windows")]
    {
        if let Some(dir) = install_path.parent() {
            let ps_cmd = format!(
                "$p = [Environment]::GetEnvironmentVariable('PATH','User'); \
                 $new = ($p -split ';' | Where-Object {{ $_ -ne '{}' }}) -join ';'; \
                 [Environment]::SetEnvironmentVariable('PATH', $new, 'User')",
                dir.display()
            );
            let _ = std::process::Command::new("powershell")
                .args(["-NoProfile", "-Command", &ps_cmd])
                .status();
            println!("[OK]   Entrada eliminada del PATH de usuario.");
        }
    }

    println!();
    println!("[OK]   Desinstalación completada.");
    println!();

    Ok(())
}

/// Versión sin prompt interactivo para llamar desde la GUI.
pub fn uninstall_silent() -> Result<(), Box<dyn std::error::Error>> {
    let install_path = get_install_path();

    if !install_path.exists() {
        return Err(format!("No se encontró stress en {}", install_path.display()).into());
    }

    match std::fs::remove_file(&install_path) {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            #[cfg(unix)]
            {
                let status = std::process::Command::new("sudo")
                    .arg("rm")
                    .arg("-f")
                    .arg(&install_path)
                    .status()?;
                if !status.success() {
                    return Err(format!(
                        "No se pudo eliminar {}. Intenta: sudo rm {}",
                        install_path.display(),
                        install_path.display()
                    ).into());
                }
            }
            #[cfg(not(unix))]
            return Err(e.into());
        }
        Err(e) => return Err(e.into()),
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(dir) = install_path.parent() {
            let ps_cmd = format!(
                "$p = [Environment]::GetEnvironmentVariable('PATH','User'); \
                 $new = ($p -split ';' | Where-Object {{ $_ -ne '{}' }}) -join ';'; \
                 [Environment]::SetEnvironmentVariable('PATH', $new, 'User')",
                dir.display()
            );
            let _ = std::process::Command::new("powershell")
                .args(["-NoProfile", "-Command", &ps_cmd])
                .status();
        }
    }

    Ok(())
}

// ── stress update ─────────────────────────────────────────────────────────────

pub async fn update() -> Result<(), Box<dyn std::error::Error>> {
    println!();
    println!("  Actualizador de stress");
    println!("  ======================");
    println!();
    println!("[INFO] Versión actual: {}", env!("CARGO_PKG_VERSION"));
    println!("[INFO] Consultando la última versión en GitHub...");

    let client = reqwest::Client::builder()
        .user_agent(concat!("stress/", env!("CARGO_PKG_VERSION")))
        .build()?;

    let response = client
        .get("https://api.github.com/repos/Guntzx/stress/releases/latest")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("GitHub API respondió con estado {}", response.status()).into());
    }

    let json: serde_json::Value = response.json().await?;
    let tag = json["tag_name"].as_str().ok_or("No se encontró tag_name en la respuesta")?;
    let latest = tag.trim_start_matches('v');
    let current = env!("CARGO_PKG_VERSION");

    if !is_newer_version(current, latest) {
        println!("[OK]   Ya tienes la versión más reciente ({}).", current);
        println!();
        return Ok(());
    }

    println!("[INFO] Nueva versión disponible: {} → {}", current, latest);

    let asset_name = get_release_asset_name();
    let assets = json["assets"].as_array().ok_or("No se encontraron assets en el release")?;

    let asset = assets
        .iter()
        .find(|a| a["name"].as_str() == Some(asset_name))
        .ok_or_else(|| format!("Asset '{}' no encontrado en el release", asset_name))?;

    let download_url = asset["browser_download_url"]
        .as_str()
        .ok_or("URL de descarga no encontrada")?;

    println!("[INFO] Descargando {}...", asset_name);

    let bytes = client.get(download_url).send().await?.bytes().await?;

    let install_path = get_install_path();

    if let Some(parent) = install_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let temp_path = install_path.with_extension("tmp");
    fs::write(&temp_path, &bytes)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&temp_path, fs::Permissions::from_mode(0o755))?;
    }

    match replace_binary(&temp_path, &install_path) {
        Ok(_) => {
            println!("[OK]   Actualizado a la versión {} en {}", latest, install_path.display());
        }
        Err(e) if e.to_string().to_lowercase().contains("permission") => {
            #[cfg(unix)]
            {
                println!("[INFO] Permisos insuficientes, reintentando con sudo...");
                let status = std::process::Command::new("sudo")
                    .args(["mv", temp_path.to_str().unwrap_or(""), install_path.to_str().unwrap_or("")])
                    .status()?;
                if status.success() {
                    let _ = std::process::Command::new("sudo")
                        .args(["chmod", "+x", install_path.to_str().unwrap_or("")])
                        .status();
                    println!("[OK]   Actualizado a la versión {} en {}", latest, install_path.display());
                } else {
                    let _ = fs::remove_file(&temp_path);
                    return Err(format!(
                        "No se pudo instalar. Intenta: sudo mv {} {}",
                        temp_path.display(),
                        install_path.display()
                    ).into());
                }
            }
            #[cfg(not(unix))]
            {
                let _ = fs::remove_file(&temp_path);
                return Err(e);
            }
        }
        Err(e) => {
            let _ = fs::remove_file(&temp_path);
            return Err(e);
        }
    }

    println!();
    println!("[OK]   Actualización completada. Ejecuta 'stress' para abrir la interfaz.");
    println!();

    Ok(())
}

// En Unix: rename directo funciona incluso sobre un binario en ejecución.
// En Windows: si el exe está en uso, se lanza un batch script que lo reemplaza al cerrar.
fn replace_binary(
    temp_path: &std::path::Path,
    install_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        match fs::rename(temp_path, install_path) {
            Ok(_) => Ok(()),
            Err(_) => {
                // El binario está en uso: programar reemplazo con un script bat
                let bat = format!(
                    "@echo off\r\n\
                     :loop\r\n\
                     timeout /t 1 /nobreak >nul\r\n\
                     move /y \"{src}\" \"{dst}\" >nul 2>&1\r\n\
                     if errorlevel 1 goto loop\r\n\
                     del \"%~f0\"\r\n",
                    src = temp_path.display(),
                    dst = install_path.display()
                );
                let bat_path = temp_path.with_extension("bat");
                fs::write(&bat_path, bat)?;
                std::process::Command::new("cmd")
                    .args(["/c", "start", "/min", "\"\"", bat_path.to_str().unwrap_or("")])
                    .spawn()?;
                println!("[INFO] El reemplazo se aplicará al cerrar stress. Reinícialo.");
                Ok(())
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        fs::rename(temp_path, install_path)?;
        Ok(())
    }
}

// ── Ayuda ─────────────────────────────────────────────────────────────────────

pub fn print_help() {
    println!();
    println!("  stress — herramienta de pruebas de carga");
    println!();
    println!("  USO:");
    println!("    stress              Abre la interfaz gráfica");
    println!("    stress update       Actualiza al último release");
    println!("    stress uninstall    Desinstala stress del sistema");
    println!("    stress help         Muestra esta ayuda");
    println!();
}
