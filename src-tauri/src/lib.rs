use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::env;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Child;
use std::sync::Mutex;
use tauri::{path::BaseDirectory, Emitter, Manager};

// Global process registry: maps "app_id:executable_name" -> Child
static PROCESS_REGISTRY: OnceCell<Mutex<HashMap<String, Child>>> = OnceCell::new();

fn get_process_registry() -> &'static Mutex<HashMap<String, Child>> {
    PROCESS_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

#[tauri::command(rename_all = "snake_case")]
async fn create_fake_game(
    handle: tauri::AppHandle,
    path: &str,
    executable_name: &str,
    path_len: i64,
    app_id: String,
) -> Result<String, String> {
    let _ = path_len;
    let exe_path: std::path::PathBuf = env::current_exe().unwrap_or_default();
    let exe_dir = exe_path.parent().unwrap_or_else(|| Path::new(""));

    let normalized_path = Path::new(path).to_string_lossy().to_string();

    let game_folder_path = exe_dir
        .join("games")
        .join(&app_id)
        .join(normalized_path);

    match std::fs::create_dir_all(&game_folder_path) {
        Ok(_) => {}
        Err(e) => return Err(format!("Error al crear carpeta del juego: {}", e)),
    };

    let resource_path = handle
        .path()
        .resolve("data/src-win.exe", BaseDirectory::Resource)
        .unwrap_or_default();

    let target_executable_path = game_folder_path.join(executable_name);
    match std::fs::copy(&resource_path, &target_executable_path) {
        Ok(_) => Ok(format!(
            "Ejecutable creado en: {:?}",
            target_executable_path
        )),
        Err(e) => Err(format!("Error al copiar ejecutable: {}", e)),
    }
}

#[tauri::command(rename_all = "snake_case")]
async fn run_background_process(
    handle: tauri::AppHandle,
    name: &str,
    path: &str,
    executable_name: &str,
    path_len: i64,
    app_id: String,
) -> Result<String, String> {
    let _ = path_len;
    let exe_path = env::current_exe().unwrap_or_default();
    let exe_dir = exe_path.parent().unwrap_or_else(|| Path::new(""));

    let normalized_path = Path::new(path).to_string_lossy().to_string();

    let game_folder_path = exe_dir
        .join("games")
        .join(&app_id)
        .join(normalized_path);
    let executable_path = game_folder_path.join(executable_name);

    let args = vec!["--title".to_string(), name.to_string()];
    // Always show window visible (no --tray flag)

    match std::process::Command::new(&executable_path)
        .args(&args)
        .current_dir(&game_folder_path)
        .spawn()
    {
        Ok(child) => {
            let process_key = format!("{}:{}", app_id, executable_name);
            let child_id = child.id();

            // Store the child in the process registry
            {
                let mut registry = get_process_registry().lock().unwrap();
                registry.insert(process_key.clone(), child);
            }

            // Spawn a monitoring thread that waits for the process to exit
            let app_handle = handle.clone();
            let key_clone = process_key.clone();
            let app_id_clone = app_id.clone();
            let exec_name_clone = executable_name.to_string();
            let game_name_clone = name.to_string();

            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    let mut registry = get_process_registry().lock().unwrap();
                    if let Some(child) = registry.get_mut(&key_clone) {
                        match child.try_wait() {
                            Ok(Some(_status)) => {
                                registry.remove(&key_clone);
                                let payload = serde_json::json!({
                                    "app_id": app_id_clone.to_string(),
                                    "executable_name": exec_name_clone,
                                    "game_name": game_name_clone,
                                });
                                let _ = app_handle.emit("process_exited", payload);
                                break;
                            }
                            Ok(None) => {
                                // Still running
                            }
                            Err(_) => {
                                registry.remove(&key_clone);
                                let payload = serde_json::json!({
                                    "app_id": app_id_clone.to_string(),
                                    "executable_name": exec_name_clone,
                                    "game_name": game_name_clone,
                                });
                                let _ = app_handle.emit("process_exited", payload);
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }
            });

            Ok(format!("Proceso iniciado (PID: {})", child_id))
        }
        Err(e) => Err(format!("Error al iniciar proceso: {}", e)),
    }
}

#[tauri::command(rename_all = "snake_case")]
async fn stop_process(exec_name: String, app_id: Option<String>) -> Result<(), String> {
    if let Some(app_id_str) = &app_id {
        let process_key = format!("{}:{}", app_id_str, exec_name);

        // Extract child from registry quickly, then release lock
        let child_opt = {
            let mut registry = get_process_registry().lock().unwrap();
            registry.remove(&process_key)
        };

        if let Some(mut child) = child_opt {
            let pid = child.id();

            // 1) Graceful: taskkill without /F sends WM_CLOSE (non-blocking spawn)
            let _ = std::process::Command::new("taskkill")
                .arg("/PID")
                .arg(pid.to_string())
                .creation_flags(0x08000000)
                .spawn();

            // 2) Background thread handles wait + force-kill fallback (no UI blocking)
            std::thread::spawn(move || {
                for _ in 0..15 {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    if let Ok(Some(_)) = child.try_wait() {
                        return;
                    }
                }
                let _ = child.kill();
                let _ = child.wait();
            });

            return Ok(());
        }

        // Not in registry — force kill by executable name (non-blocking)
        let _ = std::process::Command::new("taskkill")
            .args(&["/F", "/IM", &exec_name])
            .creation_flags(0x08000000)
            .spawn();
        return Ok(());
    }

    // No app_id: force kill by image name (non-blocking)
    let _ = std::process::Command::new("taskkill")
        .args(&["/F", "/IM", &exec_name])
        .creation_flags(0x08000000)
        .spawn();
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
fn get_active_processes() -> Vec<serde_json::Value> {
    let registry = get_process_registry().lock().unwrap();
    registry
        .keys()
        .map(|key| {
            let parts: Vec<&str> = key.splitn(2, ':').collect();
            serde_json::json!({
                "app_id": parts.get(0).unwrap_or(&""),
                "executable_name": parts.get(1).unwrap_or(&""),
                "key": key,
            })
        })
        .collect()
}




#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            create_fake_game,
            stop_process,
            run_background_process,
            get_active_processes
        ])
        .run(tauri::generate_context!())
        .expect("Error al ejecutar la aplicación");
}
