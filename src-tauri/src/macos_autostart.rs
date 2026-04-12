use objc2_foundation::NSBundle;
use objc2_service_management::{SMAppService, SMAppServiceStatus};

const BUNDLE_ID: &str = "dev.alexpedersen.better-resource-monitor";

/// Returns false if not running as the proper app bundle (e.g. `tauri dev` raw binary).
/// SMAppService.mainApp would register the wrong process in that case.
fn is_bundled() -> bool {
    unsafe {
        NSBundle::mainBundle()
            .bundleIdentifier()
            .is_some_and(|id| id.to_string() == BUNDLE_ID)
    }
}

pub fn enable() -> Result<(), String> {
    if !is_bundled() {
        return Ok(());
    }
    unsafe {
        let service = SMAppService::mainAppService();
        if service.status() == SMAppServiceStatus::Enabled {
            return Ok(());
        }
        service
            .registerAndReturnError()
            .map_err(|e| format!("SMAppService register failed: {e:?}"))
    }
}

pub fn disable() -> Result<(), String> {
    if !is_bundled() {
        return Ok(());
    }
    unsafe {
        let service = SMAppService::mainAppService();
        if service.status() == SMAppServiceStatus::NotRegistered {
            return Ok(());
        }
        service
            .unregisterAndReturnError()
            .map_err(|e| format!("SMAppService unregister failed: {e:?}"))
    }
}

pub fn is_enabled() -> bool {
    if !is_bundled() {
        return false;
    }
    unsafe { SMAppService::mainAppService().status() == SMAppServiceStatus::Enabled }
}
