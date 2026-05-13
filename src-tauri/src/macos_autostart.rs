use objc2_foundation::NSBundle;
use objc2_service_management::{SMAppService, SMAppServiceStatus};
use std::fs;
use std::path::{Path, PathBuf};

const BUNDLE_ID: &str = "dev.alexpedersen.better-resource-monitor";
const LEGACY_LAUNCH_AGENT_IDS: &[&str] = &["better-resource-monitor", "silicon-resources-monitor"];

/// Returns false if not running as the proper app bundle (e.g. `tauri dev` raw binary).
/// SMAppService.mainApp would register the wrong process in that case.
fn is_bundled() -> bool {
    NSBundle::mainBundle()
        .bundleIdentifier()
        .is_some_and(|id| id.to_string() == BUNDLE_ID)
}

pub fn status_label() -> &'static str {
    if !is_bundled() {
        return "not_bundled";
    }

    unsafe {
        match SMAppService::mainAppService().status() {
            SMAppServiceStatus::NotRegistered => "not_registered",
            SMAppServiceStatus::Enabled => "enabled",
            SMAppServiceStatus::RequiresApproval => "requires_approval",
            SMAppServiceStatus::NotFound => "not_found",
            _ => "unknown",
        }
    }
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

fn cleanup_legacy_launch_agents() -> Result<(), String> {
    let mut errors = Vec::new();

    for path in legacy_launch_agent_paths() {
        match fs::remove_file(&path) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => errors.push(format!("{}: {err}", path.display())),
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "legacy LaunchAgent cleanup failed: {}",
            errors.join("; ")
        ))
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

pub fn enable() -> Result<(), String> {
    cleanup_legacy_launch_agents()?;

    if !is_bundled() {
        return Ok(());
    }

    unsafe {
        let service = SMAppService::mainAppService();
        if service.status() == SMAppServiceStatus::Enabled {
            return Ok(());
        }
        register(&service)
    }
}

/// Repair an existing enabled login-item registration after updates or reinstalls.
/// Apple recommends re-registering updated executables, ideally with an explicit
/// unregister first, or the service may stay enabled but stop launching.
pub fn repair() -> Result<(), String> {
    cleanup_legacy_launch_agents()?;

    if !is_bundled() {
        return Ok(());
    }

    unsafe {
        let service = SMAppService::mainAppService();
        if service.status() == SMAppServiceStatus::Enabled {
            unregister(&service)?;
        }
        register(&service)
    }
}

pub fn disable() -> Result<(), String> {
    cleanup_legacy_launch_agents()?;

    if !is_bundled() {
        return Ok(());
    }

    unsafe {
        let service = SMAppService::mainAppService();
        match service.status() {
            SMAppServiceStatus::NotRegistered | SMAppServiceStatus::NotFound => Ok(()),
            _ => unregister(&service),
        }
    }
}

pub fn is_enabled() -> bool {
    if !is_bundled() {
        return false;
    }

    unsafe { SMAppService::mainAppService().status() == SMAppServiceStatus::Enabled }
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
        cleanup_legacy_launch_agents().expect("cleanup legacy launch agents");

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
