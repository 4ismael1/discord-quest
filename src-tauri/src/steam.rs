use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::os::windows::process::CommandExt;
use std::path::{Component, Path, PathBuf};
use std::process::{Child, Command};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{path::BaseDirectory, Emitter, Manager};
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
use winreg::RegKey;

const LEGACY_JOURNAL_NAME: &str = "steam-session.json";
const JOURNAL_PREFIX: &str = "steam-session-";

#[derive(Clone, Deserialize, Serialize)]
struct SteamSession {
    discord_app_id: String,
    steam_app_id: String,
    game_name: String,
    steam_root: PathBuf,
    common_root: PathBuf,
    game_root: PathBuf,
    executable_path: PathBuf,
    executable_backup: Option<PathBuf>,
    manifest_path: PathBuf,
    manifest_backup: Option<PathBuf>,
    marker_path: PathBuf,
    journal_path: PathBuf,
}

struct SteamProcess {
    child: Child,
    session: SteamSession,
}

#[derive(Default, Serialize)]
pub struct SteamCleanupReport {
    pub restored_files: u32,
    pub removed_files: u32,
    pub warnings: Vec<String>,
}

#[derive(Serialize)]
pub struct SteamLaunchResult {
    pub pid: u32,
    pub executable_path: String,
    pub manifest_path: String,
}

static STEAM_PROCESSES: OnceCell<Mutex<HashMap<String, SteamProcess>>> = OnceCell::new();

fn registry() -> &'static Mutex<HashMap<String, SteamProcess>> {
    STEAM_PROCESSES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn process_key(discord_app_id: &str) -> String {
    format!("steam:{}", discord_app_id)
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn validate_numeric_id(value: &str, label: &str) -> Result<(), String> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!("{} inválido", label));
    }
    Ok(())
}

fn normalize_relative_path(value: &str, require_exe: bool) -> Result<PathBuf, String> {
    let candidate = Path::new(value);
    let mut normalized = PathBuf::new();

    for component in candidate.components() {
        match component {
            Component::Normal(segment) => normalized.push(segment),
            Component::CurDir => {}
            _ => return Err("La ruta Steam debe ser relativa y no puede salir del juego".into()),
        }
    }

    if normalized.as_os_str().is_empty() {
        return Err("La ruta Steam está vacía".into());
    }
    if require_exe
        && normalized
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| !extension.eq_ignore_ascii_case("exe"))
            .unwrap_or(true)
    {
        return Err("Steam no devolvió un ejecutable de Windows válido".into());
    }
    Ok(normalized)
}

fn steam_root() -> Result<PathBuf, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let steam = hkcu
        .open_subkey_with_flags("Software\\Valve\\Steam", KEY_READ)
        .map_err(|_| {
            "Steam no está instalado o no aparece en el registro de Windows".to_string()
        })?;
    let value: String = steam
        .get_value("SteamPath")
        .map_err(|_| "Steam no publicó su ruta de instalación".to_string())?;
    let root = PathBuf::from(value);
    if !root.join("steam.exe").is_file() {
        return Err(format!(
            "No se encontró steam.exe en {}",
            display_path(&root)
        ));
    }
    Ok(root)
}

fn active_steam_owner() -> String {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.open_subkey_with_flags("Software\\Valve\\Steam\\ActiveProcess", KEY_READ)
        .ok()
        .and_then(|key| key.get_value::<u32, _>("ActiveUser").ok())
        .map(|account_id| (u64::from(account_id) + 76_561_197_960_265_728).to_string())
        .unwrap_or_else(|| "0".to_string())
}

fn vdf_escape(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect::<String>()
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
}

fn manifest_content(
    steam_app_id: &str,
    game_name: &str,
    install_dir: &str,
    steam_root: &Path,
    depot_id: Option<&str>,
) -> String {
    let staged_depot = depot_id
        .filter(|value| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
        .map(|value| {
            format!(
                "\n\t\t\"{}\"\n\t\t{{\n\t\t\t\"manifest\"\t\t\"0\"\n\t\t\t\"size\"\t\t\"1073741824\"\n\t\t\t\"dlcappid\"\t\t\"0\"\n\t\t}}",
                value
            )
        })
        .unwrap_or_default();

    format!(
        "\"AppState\"\n{{\n\t\"appid\"\t\t\"{}\"\n\t\"universe\"\t\t\"1\"\n\t\"LauncherPath\"\t\t\"{}\"\n\t\"name\"\t\t\"{}\"\n\t\"StateFlags\"\t\t\"1026\"\n\t\"installdir\"\t\t\"{}\"\n\t\"LastUpdated\"\t\t\"0\"\n\t\"LastPlayed\"\t\t\"0\"\n\t\"SizeOnDisk\"\t\t\"0\"\n\t\"StagingSize\"\t\t\"1073741824\"\n\t\"buildid\"\t\t\"0\"\n\t\"LastOwner\"\t\t\"{}\"\n\t\"DownloadType\"\t\t\"1\"\n\t\"UpdateResult\"\t\t\"4\"\n\t\"BytesToDownload\"\t\t\"1073741824\"\n\t\"BytesDownloaded\"\t\t\"27262976\"\n\t\"BytesToStage\"\t\t\"1073741824\"\n\t\"BytesStaged\"\t\t\"27262976\"\n\t\"TargetBuildID\"\t\t\"0\"\n\t\"AutoUpdateBehavior\"\t\t\"0\"\n\t\"AllowOtherDownloadsWhileRunning\"\t\t\"0\"\n\t\"ScheduledAutoUpdate\"\t\t\"0\"\n\t\"InstalledDepots\"\n\t{{\n\t}}\n\t\"StagedDepots\"\n\t{{{}\n\t}}\n\t\"UserConfig\"\n\t{{\n\t}}\n\t\"MountedConfig\"\n\t{{\n\t}}\n}}\n",
        steam_app_id,
        vdf_escape(&display_path(&steam_root.join("steam.exe"))),
        vdf_escape(game_name),
        vdf_escape(install_dir),
        active_steam_owner(),
        staged_depot
    )
}

fn backup_path(path: &Path, suffix: &str) -> PathBuf {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("file");
    path.with_file_name(format!("{}.discordquest-{}", name, suffix))
}

fn journal_directory(handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let directory = handle
        .path()
        .app_local_data_dir()
        .map_err(|error| format!("No se encontró la carpeta de datos: {}", error))?;
    std::fs::create_dir_all(&directory)
        .map_err(|error| format!("No se pudo crear la carpeta de datos: {}", error))?;
    Ok(directory)
}

fn journal_path(handle: &tauri::AppHandle, discord_app_id: &str) -> Result<PathBuf, String> {
    Ok(journal_directory(handle)?.join(format!("{}{}.json", JOURNAL_PREFIX, discord_app_id)))
}

fn write_journal(session: &SteamSession) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(session)
        .map_err(|error| format!("No se pudo serializar la sesión Steam: {}", error))?;
    std::fs::write(&session.journal_path, bytes)
        .map_err(|error| format!("No se pudo guardar la recuperación Steam: {}", error))
}

fn remove_with_retry(path: &Path, report: &mut SteamCleanupReport, description: &str) -> bool {
    for attempt in 0..5 {
        match std::fs::remove_file(path) {
            Ok(()) => {
                report.removed_files += 1;
                return true;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return true,
            Err(error) if attempt < 4 => {
                let _ = error;
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(error) => {
                report.warnings.push(format!(
                    "No se pudo {} {}: {}",
                    description,
                    display_path(path),
                    error
                ));
                return false;
            }
        }
    }
    false
}

fn restore_backup(
    target: &Path,
    backup: Option<&Path>,
    report: &mut SteamCleanupReport,
    description: &str,
) -> bool {
    let Some(backup) = backup else {
        return true;
    };
    if !backup.is_file() {
        report.warnings.push(format!(
            "Falta el respaldo para {}: {}",
            description,
            display_path(backup)
        ));
        return false;
    }
    if target.exists() && !remove_with_retry(target, report, "retirar el archivo temporal") {
        return false;
    }
    match std::fs::copy(backup, target) {
        Ok(_) => {
            report.restored_files += 1;
            true
        }
        Err(error) => {
            report.warnings.push(format!(
                "No se pudo restaurar {} desde {}: {}",
                description,
                display_path(backup),
                error
            ));
            false
        }
    }
}

fn cleanup_artifact(
    target: &Path,
    backup: Option<&Path>,
    report: &mut SteamCleanupReport,
    temporary_description: &str,
    original_description: &str,
) -> bool {
    match backup {
        Some(backup) if backup.is_file() => {
            if !remove_with_retry(target, report, temporary_description) {
                return false;
            }
            restore_backup(target, Some(backup), report, original_description)
        }
        // A missing backup with the target present means a previous cleanup
        // already restored the original and removed its backup. Preserve it.
        Some(_) if target.is_file() => true,
        Some(backup) => {
            report.warnings.push(format!(
                "Faltan tanto {} como su respaldo {}",
                original_description,
                display_path(backup)
            ));
            false
        }
        None => remove_with_retry(target, report, temporary_description),
    }
}

fn session_paths_are_safe(session: &SteamSession) -> bool {
    let expected_manifest = session
        .steam_root
        .join("steamapps")
        .join(format!("appmanifest_{}.acf", session.steam_app_id));
    let expected_marker = session.executable_path.with_file_name(format!(
        "{}.discordquest-steam",
        session
            .executable_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("game.exe")
    ));

    session.steam_root.is_absolute()
        && validate_numeric_id(&session.discord_app_id, "AppID de Discord").is_ok()
        && validate_numeric_id(&session.steam_app_id, "AppID de Steam").is_ok()
        && session.common_root == session.steam_root.join("steamapps").join("common")
        && session.game_root.starts_with(&session.common_root)
        && session.executable_path.starts_with(&session.game_root)
        && session.manifest_path == expected_manifest
        && session.marker_path == expected_marker
        && session
            .executable_backup
            .as_ref()
            .is_none_or(|path| path == &backup_path(&session.executable_path, "backup.exe"))
        && session
            .manifest_backup
            .as_ref()
            .is_none_or(|path| path == &backup_path(&session.manifest_path, "backup"))
}

fn artifact_is_untouched(target: &Path, backup: Option<&Path>) -> bool {
    match backup {
        Some(backup) => target.is_file() && !backup.exists(),
        None => !target.exists(),
    }
}

fn prune_empty_game_directories(session: &SteamSession) {
    let mut current = session.executable_path.parent().map(Path::to_path_buf);
    while let Some(directory) = current {
        if directory == session.common_root {
            break;
        }
        if std::fs::remove_dir(&directory).is_err() {
            break;
        }
        current = directory.parent().map(Path::to_path_buf);
    }
}

fn cleanup_session(session: &SteamSession) -> SteamCleanupReport {
    let mut report = SteamCleanupReport::default();
    if !session_paths_are_safe(session) {
        report
            .warnings
            .push("Se rechazó un journal Steam con rutas inseguras".to_string());
        return report;
    }

    let expected_marker = format!(
        "discord_app_id={}\nsteam_app_id={}\n",
        session.discord_app_id, session.steam_app_id
    );
    let marker_matches = std::fs::read_to_string(&session.marker_path)
        .map(|content| content == expected_marker)
        .unwrap_or(false);
    if !marker_matches {
        // The process may have stopped between writing the journal and its
        // marker. At that point no backup or temporary artifact has been
        // created, so removing only the journal and empty folders is safe.
        let transaction_never_started =
            artifact_is_untouched(
                &session.executable_path,
                session.executable_backup.as_deref(),
            ) && artifact_is_untouched(&session.manifest_path, session.manifest_backup.as_deref());
        if transaction_never_started {
            let _ = remove_with_retry(
                &session.journal_path,
                &mut report,
                "eliminar el journal Steam incompleto",
            );
            if report.warnings.is_empty() {
                prune_empty_game_directories(session);
            }
            return report;
        }
        report.warnings.push(format!(
            "El marcador Steam falta o no coincide: {}",
            display_path(&session.marker_path)
        ));
        return report;
    }

    let _ = cleanup_artifact(
        &session.executable_path,
        session.executable_backup.as_deref(),
        &mut report,
        "eliminar el runner Steam",
        "el ejecutable original",
    );
    let _ = cleanup_artifact(
        &session.manifest_path,
        session.manifest_backup.as_deref(),
        &mut report,
        "eliminar el appmanifest temporal",
        "el appmanifest original",
    );

    if report.warnings.is_empty() {
        if let Some(backup) = session.executable_backup.as_deref() {
            let _ = remove_with_retry(backup, &mut report, "eliminar el respaldo restaurado");
        }
        if let Some(backup) = session.manifest_backup.as_deref() {
            let _ = remove_with_retry(backup, &mut report, "eliminar el respaldo restaurado");
        }
    }

    if report.warnings.is_empty() {
        let _ = remove_with_retry(
            &session.marker_path,
            &mut report,
            "eliminar el marcador Steam",
        );
    }

    if report.warnings.is_empty() {
        let _ = remove_with_retry(
            &session.journal_path,
            &mut report,
            "eliminar el journal Steam",
        );
        if report.warnings.is_empty() {
            prune_empty_game_directories(session);
        }
    }
    report
}

fn cleanup_failed_backup(session: &SteamSession) {
    if let Some(path) = session.executable_backup.as_deref() {
        let _ = std::fs::remove_file(path);
    }
    if let Some(path) = session.manifest_backup.as_deref() {
        let _ = std::fs::remove_file(path);
    }
    let _ = std::fs::remove_file(&session.marker_path);
    let _ = std::fs::remove_file(&session.journal_path);
}

fn spawn_runner(path: &Path, name: &str, discord_app_id: &str) -> Result<Child, String> {
    let delays = [0, 100, 250, 500];
    let mut last_error = None;
    for delay in delays {
        if delay > 0 {
            std::thread::sleep(Duration::from_millis(delay));
        }
        match Command::new(path)
            .arg("--title")
            .arg(name)
            .arg("--app-id")
            .arg(discord_app_id)
            .arg("--steam-mode")
            .current_dir(path.parent().unwrap_or_else(|| Path::new(".")))
            .creation_flags(0x00000008)
            .spawn()
        {
            Ok(child) => return Ok(child),
            Err(error) => {
                let retryable = matches!(
                    error.kind(),
                    std::io::ErrorKind::PermissionDenied
                        | std::io::ErrorKind::Interrupted
                        | std::io::ErrorKind::WouldBlock
                ) || matches!(error.raw_os_error(), Some(5) | Some(32) | Some(33));
                last_error = Some(error);
                if !retryable {
                    break;
                }
            }
        }
    }

    let error = last_error
        .map(|error| {
            let code = error
                .raw_os_error()
                .map(|code| format!(" (código de Windows {})", code))
                .unwrap_or_default();
            format!("{}{}", error, code)
        })
        .unwrap_or_else(|| "error desconocido".to_string());
    Err(format!("No se pudo iniciar el runner Steam: {}", error))
}

fn emit_exit(handle: &tauri::AppHandle, session: &SteamSession, report: SteamCleanupReport) {
    let _ = handle.emit(
        "process_exited",
        serde_json::json!({
            "app_id": session.discord_app_id,
            "executable_name": session.executable_path.file_name().and_then(|name| name.to_str()).unwrap_or(""),
            "game_name": session.game_name,
            "cleanup_warnings": report.warnings,
        }),
    );
}

fn monitor_process(handle: tauri::AppHandle, key: String) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(500));
        let finished = {
            let mut processes = registry().lock().unwrap();
            match processes.get_mut(&key) {
                Some(process) => matches!(process.child.try_wait(), Ok(Some(_)) | Err(_)),
                None => return,
            }
        };
        if finished {
            let process = registry().lock().unwrap().remove(&key);
            if let Some(process) = process {
                let report = cleanup_session(&process.session);
                emit_exit(&handle, &process.session, report);
            }
            return;
        }
    });
}

#[tauri::command(rename_all = "snake_case")]
pub async fn recover_steam_session(handle: tauri::AppHandle) -> SteamCleanupReport {
    let Ok(directory) = journal_directory(&handle) else {
        return SteamCleanupReport::default();
    };
    let Ok(entries) = std::fs::read_dir(directory) else {
        return SteamCleanupReport::default();
    };
    let mut total = SteamCleanupReport::default();

    for path in entries.filter_map(Result::ok).map(|entry| entry.path()) {
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let is_session_journal = file_name == LEGACY_JOURNAL_NAME
            || (file_name.starts_with(JOURNAL_PREFIX) && file_name.ends_with(".json"));
        if !is_session_journal || !path.is_file() {
            continue;
        }

        match std::fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<SteamSession>(&bytes).ok())
        {
            Some(mut session) => {
                // The recovery file being read is authoritative. Never trust a
                // serialized path when deciding which journal may be removed.
                session.journal_path = path;
                let report = cleanup_session(&session);
                total.restored_files += report.restored_files;
                total.removed_files += report.removed_files;
                total.warnings.extend(report.warnings);
            }
            None => total.warnings.push(format!(
                "No se pudo leer la recuperación Steam: {}",
                display_path(&path)
            )),
        }
    }

    total
}

#[tauri::command(rename_all = "snake_case")]
pub async fn run_steam_game(
    handle: tauri::AppHandle,
    name: String,
    discord_app_id: String,
    steam_app_id: String,
    install_dir: String,
    executable_path: String,
    depot_id: Option<String>,
) -> Result<SteamLaunchResult, String> {
    validate_numeric_id(&discord_app_id, "AppID de Discord")?;
    validate_numeric_id(&steam_app_id, "AppID de Steam")?;
    if let Some(depot_id) = depot_id.as_deref() {
        validate_numeric_id(depot_id, "DepotID de Steam")?;
    }
    // Hold the registry lock throughout preparation to serialize rapid starts.
    // Each active game has an independent recovery journal.
    let mut processes = registry().lock().unwrap();
    if processes.contains_key(&process_key(&discord_app_id)) {
        return Err("Este juego ya tiene una simulación Steam activa".to_string());
    }
    if processes
        .values()
        .any(|process| process.session.steam_app_id == steam_app_id)
    {
        return Err("Ese AppID de Steam ya tiene una simulación activa".to_string());
    }

    let root = steam_root()?;
    let common_root = root.join("steamapps").join("common");
    let install_relative = normalize_relative_path(&install_dir, false)?;
    let executable_relative = normalize_relative_path(&executable_path, true)?;
    let game_root = common_root.join(&install_relative);
    let target = game_root.join(&executable_relative);
    let manifest = root
        .join("steamapps")
        .join(format!("appmanifest_{}.acf", steam_app_id));
    let marker = target.with_file_name(format!(
        "{}.discordquest-steam",
        target
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("game.exe")
    ));
    let executable_backup_path = backup_path(&target, "backup.exe");
    let manifest_backup_path = backup_path(&manifest, "backup");
    let resource = handle
        .path()
        .resolve("data/src-win.exe", BaseDirectory::Resource)
        .map_err(|error| format!("No se encontró el runner integrado: {}", error))?;
    if !resource.is_file() {
        return Err(format!(
            "El runner integrado no existe: {}",
            display_path(&resource)
        ));
    }

    if executable_backup_path.exists() || manifest_backup_path.exists() {
        return Err(
            "Hay respaldos Steam pendientes; reinicia DiscordQuest para recuperarlos".into(),
        );
    }

    std::fs::create_dir_all(target.parent().ok_or("Ruta Steam inválida")?)
        .map_err(|error| format!("No se pudo crear la carpeta Steam: {}", error))?;
    let canonical_common = common_root
        .canonicalize()
        .map_err(|error| format!("No se pudo validar steamapps/common: {}", error))?;
    let canonical_parent = target
        .parent()
        .ok_or("Ruta Steam inválida")?
        .canonicalize()
        .map_err(|error| format!("No se pudo validar la carpeta del runner: {}", error))?;
    if !canonical_parent.starts_with(&canonical_common) {
        return Err("La carpeta Steam resuelta sale de steamapps/common".into());
    }

    let recovery_directory = journal_directory(&handle)?;
    let recovery_journal = journal_path(&handle, &discord_app_id)?;
    if recovery_journal.is_file() || recovery_directory.join(LEGACY_JOURNAL_NAME).is_file() {
        return Err(
            "Hay una limpieza Steam pendiente; reinicia DiscordQuest antes de continuar".into(),
        );
    }

    let session = SteamSession {
        discord_app_id: discord_app_id.clone(),
        steam_app_id: steam_app_id.clone(),
        game_name: name.clone(),
        steam_root: root.clone(),
        common_root,
        game_root,
        executable_path: target.clone(),
        executable_backup: target.is_file().then_some(executable_backup_path),
        manifest_path: manifest.clone(),
        manifest_backup: manifest.is_file().then_some(manifest_backup_path),
        marker_path: marker.clone(),
        journal_path: recovery_journal,
    };
    write_journal(&session)?;

    // The marker must exist before any backup is created. If the app stops at
    // any later instruction, recovery can safely identify this transaction.
    if let Err(error) = std::fs::write(
        &marker,
        format!(
            "discord_app_id={}\nsteam_app_id={}\n",
            discord_app_id, steam_app_id
        ),
    ) {
        cleanup_failed_backup(&session);
        return Err(format!("No se pudo crear el marcador Steam: {}", error));
    }

    if let Some(backup) = session.executable_backup.as_deref() {
        if let Err(error) = std::fs::copy(&target, backup) {
            cleanup_failed_backup(&session);
            return Err(format!(
                "No se pudo respaldar el ejecutable original: {}",
                error
            ));
        }
    }
    if let Some(backup) = session.manifest_backup.as_deref() {
        if let Err(error) = std::fs::copy(&manifest, backup) {
            cleanup_failed_backup(&session);
            return Err(format!(
                "No se pudo respaldar el appmanifest original: {}",
                error
            ));
        }
    }

    let content = manifest_content(
        &steam_app_id,
        name.trim(),
        &install_relative.to_string_lossy(),
        &root,
        depot_id.as_deref(),
    );
    if let Err(error) = std::fs::write(&manifest, content) {
        let report = cleanup_session(&session);
        return Err(format!(
            "No se pudo crear el appmanifest: {}{}",
            error,
            if report.warnings.is_empty() {
                String::new()
            } else {
                format!("; limpieza: {}", report.warnings.join(" | "))
            }
        ));
    }

    if let Err(error) = std::fs::copy(&resource, &target) {
        let report = cleanup_session(&session);
        return Err(format!(
            "No se pudo copiar el runner a Steam: {}{}",
            error,
            if report.warnings.is_empty() {
                String::new()
            } else {
                format!("; limpieza: {}", report.warnings.join(" | "))
            }
        ));
    }

    let child = match spawn_runner(&target, &name, &discord_app_id) {
        Ok(child) => child,
        Err(error) => {
            let report = cleanup_session(&session);
            return Err(format!(
                "{}{}",
                error,
                if report.warnings.is_empty() {
                    String::new()
                } else {
                    format!("; limpieza: {}", report.warnings.join(" | "))
                }
            ));
        }
    };
    let pid = child.id();
    let key = process_key(&discord_app_id);
    processes.insert(
        key.clone(),
        SteamProcess {
            child,
            session: session.clone(),
        },
    );
    drop(processes);
    monitor_process(handle, key);

    Ok(SteamLaunchResult {
        pid,
        executable_path: display_path(&target),
        manifest_path: display_path(&manifest),
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn stop_steam_process(
    handle: tauri::AppHandle,
    discord_app_id: String,
) -> Result<(), String> {
    let key = process_key(&discord_app_id);
    let process = registry().lock().unwrap().remove(&key);
    let Some(mut process) = process else {
        return Ok(());
    };

    let _ = process.child.kill();
    let _ = process.child.wait();
    let report = cleanup_session(&process.session);
    emit_exit(&handle, &process.session, report);
    Ok(())
}

pub fn active_processes() -> Vec<serde_json::Value> {
    registry()
        .lock()
        .unwrap()
        .values()
        .map(|process| {
            serde_json::json!({
                "app_id": process.session.discord_app_id,
                "executable_name": process.session.executable_path.file_name().and_then(|name| name.to_str()).unwrap_or(""),
                "key": process_key(&process.session.discord_app_id),
                "mode": "steam",
            })
        })
        .collect()
}

pub fn shutdown() {
    let processes = {
        let mut processes = registry().lock().unwrap();
        std::mem::take(&mut *processes)
    };
    for (_, mut process) in processes {
        let _ = process.child.kill();
        let _ = process.child.wait();
        let _ = cleanup_session(&process.session);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        backup_path, cleanup_session, manifest_content, normalize_relative_path, SteamSession,
    };
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_root(label: &str) -> PathBuf {
        let sequence = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "discordquest-steam-{}-{}-{}",
            label,
            std::process::id(),
            sequence
        ))
    }

    fn test_session(root: &Path, with_originals: bool) -> SteamSession {
        let common_root = root.join("steamapps").join("common");
        let game_root = common_root.join("Example");
        let executable_path = game_root.join("Bin").join("Example.exe");
        let manifest_path = root.join("steamapps").join("appmanifest_123.acf");
        let marker_path = executable_path.with_file_name("Example.exe.discordquest-steam");
        SteamSession {
            discord_app_id: "456".into(),
            steam_app_id: "123".into(),
            game_name: "Example".into(),
            steam_root: root.to_path_buf(),
            common_root,
            game_root,
            executable_backup: with_originals.then(|| backup_path(&executable_path, "backup.exe")),
            manifest_backup: with_originals.then(|| backup_path(&manifest_path, "backup")),
            executable_path,
            manifest_path,
            marker_path,
            journal_path: root.join("steam-session.json"),
        }
    }

    fn write_temporary_session(session: &SteamSession, with_originals: bool) {
        std::fs::create_dir_all(session.executable_path.parent().unwrap()).unwrap();
        if with_originals {
            std::fs::write(session.executable_backup.as_ref().unwrap(), b"original exe").unwrap();
            std::fs::write(
                session.manifest_backup.as_ref().unwrap(),
                b"original manifest",
            )
            .unwrap();
        }
        std::fs::write(&session.executable_path, b"temporary runner").unwrap();
        std::fs::write(&session.manifest_path, b"temporary manifest").unwrap();
        std::fs::write(
            &session.marker_path,
            b"discord_app_id=456\nsteam_app_id=123\n",
        )
        .unwrap();
        std::fs::write(&session.journal_path, b"journal").unwrap();
    }

    #[test]
    fn rejects_paths_outside_the_steam_game() {
        assert!(normalize_relative_path(r"Bin\\Game.exe", true).is_ok());
        assert!(normalize_relative_path(r"..\\Game.exe", true).is_err());
        assert!(normalize_relative_path(r"C:\\Game.exe", true).is_err());
        assert!(normalize_relative_path("Game.dll", true).is_err());
    }

    #[test]
    fn creates_orbshacker_manifest_state() {
        let content = manifest_content(
            "123",
            "Example",
            "Example",
            Path::new(r"C:\\Steam"),
            Some("456"),
        );
        assert!(content.contains("\"appid\"\t\t\"123\""));
        assert!(content.contains("\"StateFlags\"\t\t\"1026\""));
        assert!(content.contains("\"456\""));
    }

    #[test]
    fn cleanup_removes_only_temporary_steam_files() {
        let root = test_root("temporary");
        let session = test_session(&root, false);
        write_temporary_session(&session, false);

        let report = cleanup_session(&session);

        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
        assert!(!session.executable_path.exists());
        assert!(!session.manifest_path.exists());
        assert!(!session.marker_path.exists());
        assert!(!session.journal_path.exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn cleanup_restores_preexisting_files() {
        let root = test_root("restore");
        let session = test_session(&root, true);
        write_temporary_session(&session, true);

        let report = cleanup_session(&session);

        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
        assert_eq!(
            std::fs::read(&session.executable_path).unwrap(),
            b"original exe"
        );
        assert_eq!(
            std::fs::read(&session.manifest_path).unwrap(),
            b"original manifest"
        );
        assert!(!session.executable_backup.as_ref().unwrap().exists());
        assert!(!session.manifest_backup.as_ref().unwrap().exists());
        assert!(!session.marker_path.exists());
        assert!(!session.journal_path.exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn cleanup_recovers_a_session_interrupted_before_marker() {
        let root = test_root("before-marker");
        let session = test_session(&root, true);
        std::fs::create_dir_all(session.executable_path.parent().unwrap()).unwrap();
        std::fs::write(&session.executable_path, b"original exe").unwrap();
        std::fs::write(&session.manifest_path, b"original manifest").unwrap();
        std::fs::write(&session.journal_path, b"journal").unwrap();

        let report = cleanup_session(&session);

        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
        assert_eq!(
            std::fs::read(&session.executable_path).unwrap(),
            b"original exe"
        );
        assert_eq!(
            std::fs::read(&session.manifest_path).unwrap(),
            b"original manifest"
        );
        assert!(!session.journal_path.exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn cleanup_keeps_other_steam_sessions_intact() {
        let root = test_root("multiple");
        let mut first = test_session(&root, false);
        first.journal_path = root.join("steam-session-456.json");

        let mut second = test_session(&root, false);
        second.discord_app_id = "789".into();
        second.steam_app_id = "321".into();
        second.game_root = second.common_root.join("Other");
        second.executable_path = second.game_root.join("Other.exe");
        second.manifest_path = root.join("steamapps").join("appmanifest_321.acf");
        second.marker_path = second
            .executable_path
            .with_file_name("Other.exe.discordquest-steam");
        second.journal_path = root.join("steam-session-789.json");

        write_temporary_session(&first, false);
        std::fs::create_dir_all(second.executable_path.parent().unwrap()).unwrap();
        std::fs::write(&second.executable_path, b"second runner").unwrap();
        std::fs::write(&second.manifest_path, b"second manifest").unwrap();
        std::fs::write(
            &second.marker_path,
            b"discord_app_id=789\nsteam_app_id=321\n",
        )
        .unwrap();
        std::fs::write(&second.journal_path, b"second journal").unwrap();

        let first_report = cleanup_session(&first);

        assert!(
            first_report.warnings.is_empty(),
            "{:?}",
            first_report.warnings
        );
        assert!(second.executable_path.is_file());
        assert!(second.manifest_path.is_file());
        assert!(second.marker_path.is_file());
        assert!(second.journal_path.is_file());

        let second_report = cleanup_session(&second);
        assert!(
            second_report.warnings.is_empty(),
            "{:?}",
            second_report.warnings
        );
        let _ = std::fs::remove_dir_all(root);
    }
}
