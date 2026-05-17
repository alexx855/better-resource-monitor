use objc2_foundation::{NSBundle, NSString};
use objc2_service_management::{SMAppService, SMAppServiceStatus};
use std::fs;
use std::path::{Path, PathBuf};

const BUNDLE_ID: &str = "dev.alexpedersen.better-resource-monitor";
const AUTOSTART_AGENT_PLIST: &str = "dev.alexpedersen.better-resource-monitor.autostart.plist";
const LEGACY_LAUNCH_AGENT_IDS: &[&str] = &["better-resource-monitor", "silicon-resources-monitor"];

/// Returns false if not running as the proper app bundle (e.g. `tauri dev` raw binary).
/// SMAppService.mainApp would register the wrong process in that case.
fn is_bundled() -> bool {
    NSBundle::mainBundle()
        .bundleIdentifier()
        .is_some_and(|id| id.to_string() == BUNDLE_ID)
}

fn service_status_label(service: &SMAppService) -> &'static str {
    unsafe {
        match service.status() {
            SMAppServiceStatus::NotRegistered => "not_registered",
            SMAppServiceStatus::Enabled => "enabled",
            SMAppServiceStatus::RequiresApproval => "requires_approval",
            SMAppServiceStatus::NotFound => "not_found",
            _ => "unknown",
        }
    }
}

fn with_autostart_agent_service<T>(f: impl FnOnce(&SMAppService) -> T) -> T {
    let plist = NSString::from_str(AUTOSTART_AGENT_PLIST);
    let service = unsafe { SMAppService::agentServiceWithPlistName(&plist) };
    f(&service)
}

fn with_main_app_service<T>(f: impl FnOnce(&SMAppService) -> T) -> T {
    let service = unsafe { SMAppService::mainAppService() };
    f(&service)
}

pub fn status_label() -> String {
    if !is_bundled() {
        return "not_bundled".to_string();
    }

    with_autostart_agent_service(|agent| {
        with_main_app_service(|main| {
            format!(
                "agent={} main_app={}",
                service_status_label(agent),
                service_status_label(main)
            )
        })
    })
}

fn launch_agents_dir(home: &Path) -> PathBuf {
    home.join("Library").join("LaunchAgents")
}

fn legacy_launch_agent_paths_in(home: &Path) -> Vec<PathBuf> {
    let dir = launch_agents_dir(home);
    LEGACY_LAUNCH_AGENT_IDS
        .iter()
        .map(|id| dir.join(format!("{id}.plist")))
        .collect()
}

fn legacy_launch_agent_paths() -> Vec<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| legacy_launch_agent_paths_in(&home))
        .unwrap_or_default()
}

fn cleanup_legacy_launch_agents() {
    for path in legacy_launch_agent_paths() {
        match fs::remove_file(&path) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => {
                eprintln!(
                    "Failed to remove legacy LaunchAgent {}: {err}",
                    path.display()
                );
            }
        }
    }
}

fn register(service: &SMAppService) -> Result<(), String> {
    unsafe {
        service
            .registerAndReturnError()
            .map_err(|e| format!("SMAppService register failed: {e:?}"))
    }
}

fn unregister(service: &SMAppService) -> Result<(), String> {
    unsafe {
        service
            .unregisterAndReturnError()
            .map_err(|e| format!("SMAppService unregister failed: {e:?}"))
    }
}

fn unregister_if_registered(service: &SMAppService) -> Result<(), String> {
    unsafe {
        match service.status() {
            SMAppServiceStatus::NotRegistered | SMAppServiceStatus::NotFound => Ok(()),
            _ => unregister(service),
        }
    }
}

fn register_if_needed(service: &SMAppService) -> Result<(), String> {
    unsafe {
        if service.status() == SMAppServiceStatus::Enabled {
            return Ok(());
        }
        register(service)
    }
}

fn remove_main_app_login_item() -> Result<(), String> {
    with_main_app_service(unregister_if_registered)
}

pub fn enable() -> Result<(), String> {
    cleanup_legacy_launch_agents();

    if !is_bundled() {
        return Ok(());
    }

    remove_main_app_login_item()?;
    with_autostart_agent_service(register_if_needed)
}

/// Repair autostart registration after updates or reinstalls.
///
/// The old main-app login item can be relaunched by macOS as a stopped process
/// on Ventura Intel before any app setup code runs. The bundled LaunchAgent
/// starts the app executable directly through launchd, which avoids that
/// LaunchServices-suspended path while still using SMAppService-managed metadata.
///
/// Registering a LaunchAgent immediately bootstraps it, so an already-enabled
/// agent must be left alone during normal app startup and second-instance
/// handling. Otherwise each launch would spawn another app instance.
pub fn repair() -> Result<(), String> {
    cleanup_legacy_launch_agents();

    if !is_bundled() {
        return Ok(());
    }

    remove_main_app_login_item()?;

    with_autostart_agent_service(register_if_needed)
}

pub fn disable() -> Result<(), String> {
    cleanup_legacy_launch_agents();

    if !is_bundled() {
        return Ok(());
    }

    with_autostart_agent_service(unregister_if_registered)?;
    remove_main_app_login_item()
}

pub fn is_enabled() -> bool {
    if !is_bundled() {
        return false;
    }

    unsafe {
        with_autostart_agent_service(|agent| agent.status() == SMAppServiceStatus::Enabled)
            || with_main_app_service(|main| main.status() == SMAppServiceStatus::Enabled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn legacy_launch_agent_paths_cover_known_upgrade_names() {
        let home = Path::new("/tmp/brm-test-home");
        let paths = legacy_launch_agent_paths_in(home);

        assert_eq!(
            paths,
            vec![
                home.join("Library/LaunchAgents/better-resource-monitor.plist"),
                home.join("Library/LaunchAgents/silicon-resources-monitor.plist"),
            ]
        );
    }

    #[test]
    fn cleanup_legacy_launch_agents_removes_known_plists() {
        let _guard = env_lock().lock().expect("env lock poisoned");
        let previous_home = std::env::var_os("HOME");
        let test_home = std::env::temp_dir().join(format!(
            "brm-autostart-test-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("unnamed")
        ));
        let launch_agents = launch_agents_dir(&test_home);
        fs::create_dir_all(&launch_agents).expect("create launch agents dir");

        for path in legacy_launch_agent_paths_in(&test_home) {
            fs::write(&path, b"legacy").expect("write legacy plist");
        }

        std::env::set_var("HOME", &test_home);
        cleanup_legacy_launch_agents();

        for path in legacy_launch_agent_paths_in(&test_home) {
            assert!(!path.exists(), "expected {} to be removed", path.display());
        }

        if let Some(home) = previous_home {
            std::env::set_var("HOME", home);
        } else {
            std::env::remove_var("HOME");
        }
        let _ = fs::remove_dir_all(&test_home);
    }
}
