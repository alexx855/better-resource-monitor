use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct StorageSample {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_percent: f32,
}

pub(crate) fn sample() -> Option<StorageSample> {
    platform_sample()
}

pub(crate) fn sample_from_bytes(total_bytes: u128, available_bytes: u128) -> Option<StorageSample> {
    if total_bytes == 0 {
        return None;
    }

    let available_bytes = available_bytes.min(total_bytes);
    let used_bytes = total_bytes.saturating_sub(available_bytes);
    let used_percent = ((used_bytes as f64 / total_bytes as f64) * 100.0).clamp(0.0, 100.0) as f32;

    Some(StorageSample {
        total_bytes: total_bytes.min(u64::MAX as u128) as u64,
        available_bytes: available_bytes.min(u64::MAX as u128) as u64,
        used_percent,
    })
}

#[cfg(any(test, target_os = "linux"))]
pub(crate) fn sample_from_blocks(
    blocks: u128,
    available_blocks: u128,
    fragment_size: u128,
    block_size: u128,
) -> Option<StorageSample> {
    let bytes_per_block = if fragment_size > 0 {
        fragment_size
    } else {
        block_size
    };

    if bytes_per_block == 0 {
        return None;
    }

    sample_from_bytes(
        blocks.saturating_mul(bytes_per_block),
        available_blocks.saturating_mul(bytes_per_block),
    )
}

fn home_path_or_root() -> PathBuf {
    std::env::var_os("HOME")
        .filter(|home| !home.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

#[cfg(target_os = "macos")]
fn platform_sample() -> Option<StorageSample> {
    use core::ffi::c_void;
    use core_foundation::array::CFArray;
    use core_foundation::base::TCFType;
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::number::{CFNumber, CFNumberRef};
    use core_foundation::url::{
        kCFURLVolumeAvailableCapacityKey, kCFURLVolumeTotalCapacityKey, CFURL,
    };
    use core_foundation_sys::dictionary::CFDictionaryGetValueIfPresent;
    use core_foundation_sys::string::CFStringRef;
    use core_foundation_sys::url::CFURLCopyResourcePropertiesForKeys;
    use std::ptr;

    unsafe fn dictionary_capacity(dict: &CFDictionary, key: CFStringRef) -> Option<u64> {
        let mut value: *const c_void = ptr::null();
        if CFDictionaryGetValueIfPresent(
            dict.as_concrete_TypeRef(),
            key as *const c_void,
            &mut value,
        ) == 0
            || value.is_null()
        {
            return None;
        }

        let number: CFNumber = TCFType::wrap_under_get_rule(value as CFNumberRef);
        u64::try_from(number.to_i64()?).ok()
    }

    let url = CFURL::from_path(home_path_or_root(), true)?;
    let keys = unsafe {
        [
            kCFURLVolumeTotalCapacityKey,
            kCFURLVolumeAvailableCapacityKey,
        ]
    };
    let keys = CFArray::from_copyable(&keys);
    let dict_ref = unsafe {
        CFURLCopyResourcePropertiesForKeys(
            url.as_concrete_TypeRef(),
            keys.as_concrete_TypeRef(),
            ptr::null_mut(),
        )
    };

    if dict_ref.is_null() {
        return None;
    }

    let dict: CFDictionary = unsafe { TCFType::wrap_under_create_rule(dict_ref) };
    let total_bytes = unsafe { dictionary_capacity(&dict, kCFURLVolumeTotalCapacityKey) }?;
    let available_bytes = unsafe { dictionary_capacity(&dict, kCFURLVolumeAvailableCapacityKey) }?;

    sample_from_bytes(total_bytes as u128, available_bytes as u128)
}

#[cfg(target_os = "linux")]
fn platform_sample() -> Option<StorageSample> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let path = home_path_or_root();
    let c_path = CString::new(path.as_os_str().as_bytes()).ok()?;
    let mut stat = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    let result = unsafe { libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) };
    if result != 0 {
        return None;
    }

    let stat = unsafe { stat.assume_init() };
    sample_from_blocks(
        stat.f_blocks as u128,
        stat.f_bavail as u128,
        stat.f_frsize as u128,
        stat.f_bsize as u128,
    )
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn platform_sample() -> Option<StorageSample> {
    None
}
