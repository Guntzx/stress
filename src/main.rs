#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod cli;
mod config;
mod gui;
mod load_test;
mod models;
mod report_generator;

/// En Windows, los binarios marcados como `windows_subsystem = "windows"` no tienen
/// consola adjunta al lanzarse. Esto hace que println!/eprintln! no aparezcan en
/// terminal. Esta función adjunta la consola del proceso padre (el terminal que
/// lanzó el exe) y redirige los handles estándar para que la salida sea visible.
#[cfg(target_os = "windows")]
fn attach_cli_console() {
    use std::ffi::c_void;
    #[allow(non_snake_case)]
    extern "system" {
        fn AttachConsole(dwProcessId: u32) -> i32;
        fn SetStdHandle(nStdHandle: u32, hHandle: *mut c_void) -> i32;
        fn CreateFileW(
            lpFileName: *const u16,
            dwDesiredAccess: u32,
            dwShareMode: u32,
            lpSecurityAttributes: *mut c_void,
            dwCreationDisposition: u32,
            dwFlagsAndAttributes: u32,
            hTemplateFile: *mut c_void,
        ) -> *mut c_void;
    }

    const ATTACH_PARENT_PROCESS: u32 = 0xFFFF_FFFF;
    const GENERIC_READ: u32 = 0x8000_0000;
    const GENERIC_WRITE: u32 = 0x4000_0000;
    const FILE_SHARE_READ_WRITE: u32 = 0x3;
    const OPEN_EXISTING: u32 = 3;
    const STD_INPUT_HANDLE: u32 = 0xFFFF_FFF6;
    const STD_OUTPUT_HANDLE: u32 = 0xFFFF_FFF5;
    const STD_ERROR_HANDLE: u32 = 0xFFFF_FFF4;
    const INVALID_HANDLE_VALUE: *mut c_void = -1_isize as usize as *mut c_void;

    unsafe {
        // Adjuntar la consola del proceso padre (el terminal que lanzó el exe)
        AttachConsole(ATTACH_PARENT_PROCESS);

        // Redirigir stdout y stderr a CONOUT$ (consola adjunta)
        let conout: Vec<u16> = "CONOUT$\0".encode_utf16().collect();
        let hout = CreateFileW(
            conout.as_ptr(),
            GENERIC_WRITE,
            FILE_SHARE_READ_WRITE,
            std::ptr::null_mut(),
            OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        );
        if hout != INVALID_HANDLE_VALUE {
            SetStdHandle(STD_OUTPUT_HANDLE, hout);
            SetStdHandle(STD_ERROR_HANDLE, hout);
        }

        // Redirigir stdin a CONIN$ (para comandos interactivos como uninstall)
        let conin: Vec<u16> = "CONIN$\0".encode_utf16().collect();
        let hin = CreateFileW(
            conin.as_ptr(),
            GENERIC_READ,
            FILE_SHARE_READ_WRITE,
            std::ptr::null_mut(),
            OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        );
        if hin != INVALID_HANDLE_VALUE {
            SetStdHandle(STD_INPUT_HANDLE, hin);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if let Some(subcommand) = args.get(1) {
        // Adjuntar consola del terminal para que println!/eprintln! sean visibles
        #[cfg(target_os = "windows")]
        attach_cli_console();
        // Crear runtime Tokio para los subcomandos CLI asíncronos
        let rt = tokio::runtime::Runtime::new()?;
        match subcommand.as_str() {
            "update" => {
                rt.block_on(cli::update())?;
                return Ok(());
            }
            "uninstall" => {
                cli::uninstall()?;
                return Ok(());
            }
            "help" | "--help" | "-h" => {
                cli::print_help();
                return Ok(());
            }
            other => {
                eprintln!("[ERROR] Subcomando desconocido: '{other}'");
                eprintln!("        Ejecuta 'stress help' para ver los comandos disponibles.");
                std::process::exit(1);
            }
        }
    }

    // Inicializar logging
    tracing_subscriber::fmt::init();

    // Lanzar la interfaz gráfica Slint
    gui::run_app()
}
