use objc2_foundation::NSBundle;
use objc2_service_management::{SMAppService, SMAppServiceStatus};

const BUNDLE_ID: &str = "dev.alexpedersen.better-resource-monitor";

// Greenfield TestFlight/App Store runtime only. The removed bundled LaunchAgent
// path was never a supported shipped artifact, so this module intentionally does
// not migrate or clean up old agent registrations.

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

fn with_main_app_service<T>(f: impl FnOnce(&SMAppService) -> T) -> T {
    let service = unsafe { SMAppService::mainAppService() };
    f(&service)
}

pub fn status_label() -> String {
    if !is_bundled() {
        return "not_bundled".to_string();
    }

    with_main_app_service(|main| format!("main_app={}", service_status_label(main)))
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
    if !is_bundled() {
        return Ok(());
    }

    with_main_app_service(register_if_needed)
}

pub fn disable() -> Result<(), String> {
    if !is_bundled() {
        return Ok(());
    }

    remove_main_app_login_item()
}

pub fn is_enabled() -> bool {
    if !is_bundled() {
        return false;
    }

    with_main_app_service(|main| unsafe { main.status() == SMAppServiceStatus::Enabled })
}
