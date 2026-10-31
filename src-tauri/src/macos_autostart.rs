use objc2_foundation::NSBundle;
use objc2_service_management::{SMAppService, SMAppServiceStatus};

const BUNDLE_ID: &str = "dev.alexpedersen.better-resource-monitor";

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

pub fn toggle() -> Result<bool, String> {
    toggle_with(
        is_enabled,
        |enabled| {
            if enabled {
                enable()
            } else {
                disable()
            }
        },
    )
}

fn toggle_with(
    is_enabled: impl FnOnce() -> bool,
    set_enabled: impl FnOnce(bool) -> Result<(), String>,
) -> Result<bool, String> {
    let requested = !is_enabled();
    set_enabled(requested)?;
    Ok(requested)
}

/// Read and apply status together on the UI thread, so queued work cannot
/// overwrite a newer click with a status sampled on the monitoring thread.
pub fn sync_menu(item: &tauri::menu::CheckMenuItem<tauri::Wry>) -> tauri::Result<()> {
    sync_menu_with(
        is_enabled,
        || item.is_checked(),
        |value| item.set_checked(value),
    )
}

fn sync_menu_with<E>(
    is_enabled: impl FnOnce() -> bool,
    is_checked: impl FnOnce() -> Result<bool, E>,
    set_checked: impl FnOnce(bool) -> Result<(), E>,
) -> Result<(), E> {
    let enabled = is_enabled();
    if is_checked()? != enabled {
        set_checked(enabled)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn sync(enabled: &Cell<bool>, checked: &Cell<bool>) {
        sync_menu_with(
            || enabled.get(),
            || Ok::<_, ()>(checked.get()),
            |value| {
                checked.set(value);
                Ok(())
            },
        )
        .unwrap();
    }

    #[test]
    fn approval_then_external_disable_updates_menu_and_next_click() {
        let enabled = Cell::new(false);
        let checked = Cell::new(false);
        let requested = Cell::new(false);

        // Registration can succeed while approval is still pending.
        assert!(toggle_with(
            || enabled.get(),
            |value| {
                requested.set(value);
                Ok(())
            }
        )
        .unwrap());
        assert!(requested.get());
        checked.set(true); // Native check menu items toggle before the handler.
        sync(&enabled, &checked);
        assert!(!checked.get());

        // Approval in System Settings must be visible without another click.
        enabled.set(true);
        sync(&enabled, &checked);
        assert!(checked.get());
        assert!(!toggle_with(
            || enabled.get(),
            |value| {
                requested.set(value);
                enabled.set(value);
                Ok(())
            }
        )
        .unwrap());
        sync(&enabled, &checked);
        assert!(!requested.get());
        assert!(!checked.get());

        enabled.set(true);
        sync(&enabled, &checked);
        enabled.set(false); // User disables the item outside this app.
        sync(&enabled, &checked);
        assert!(!checked.get());
        assert!(toggle_with(
            || enabled.get(),
            |value| {
                requested.set(value);
                enabled.set(value);
                Ok(())
            }
        )
        .unwrap());
        sync(&enabled, &checked);
        assert!(requested.get());
        assert!(checked.get());
    }

    #[test]
    fn queued_refresh_reads_status_when_executed() {
        let enabled = Cell::new(true);
        let checked = Cell::new(true);
        let queued_refresh = || sync(&enabled, &checked);
        toggle_with(
            || enabled.get(),
            |value| {
                enabled.set(value);
                Ok(())
            },
        )
        .unwrap();
        queued_refresh();
        assert!(!checked.get());
    }

    #[test]
    fn refresh_retries_failed_updates_and_skips_unchanged_items() {
        let checked = Cell::new(false);
        assert_eq!(
            sync_menu_with(|| true, || Ok(checked.get()), |_| Err("UI unavailable")),
            Err("UI unavailable")
        );
        sync(&Cell::new(true), &checked);
        assert!(checked.get());
        sync_menu_with::<()>(
            || true,
            || Ok(checked.get()),
            |_| panic!("unchanged menu should not be rewritten"),
        )
        .unwrap();
    }

    #[test]
    fn failed_toggle_restores_live_state_after_native_checkmark_change() {
        let enabled = Cell::new(false);
        let checked = Cell::new(true);
        assert_eq!(
            toggle_with(|| enabled.get(), |_| Err("registration failed".into())),
            Err("registration failed".into())
        );
        sync(&enabled, &checked);
        assert!(!checked.get());
    }
}
