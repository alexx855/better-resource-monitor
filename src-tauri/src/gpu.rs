//! GPU utilization monitoring
//!
//! Platform-specific implementations:
//! - macOS: Uses IOAccelerator via public IOKit APIs for Apple Silicon device utilization
//! - Linux: Uses NVML (NVIDIA Management Library) for NVIDIA GPU utilization

// ============================================================================
// macOS Implementation (Apple Silicon via IOAccelerator)
// ============================================================================

#[cfg(target_os = "macos")]
mod macos {
    use std::ffi::c_void;

    use core_foundation::base::{kCFAllocatorDefault, CFRelease, CFTypeRef};
    use core_foundation::dictionary::{
        CFDictionaryGetValue, CFDictionaryRef, CFMutableDictionaryRef,
    };
    use core_foundation::string::{
        kCFStringEncodingUTF8, CFStringCreateWithBytesNoCopy, CFStringRef,
    };

    #[allow(non_camel_case_types)]
    type io_object_t = u32;
    #[allow(non_camel_case_types)]
    type io_iterator_t = u32;
    #[allow(non_camel_case_types)]
    type io_registry_entry_t = u32;
    #[allow(non_camel_case_types)]
    type kern_return_t = i32;

    const KERN_SUCCESS: kern_return_t = 0;
    const IO_OBJECT_NULL: io_object_t = 0;
    const CF_NUMBER_SINT64_TYPE: isize = 4;

    #[link(name = "IOKit", kind = "framework")]
    unsafe extern "C" {
        fn IOServiceMatching(name: *const i8) -> CFMutableDictionaryRef;
        fn IOServiceGetMatchingServices(
            main_port: u32,
            matching: CFDictionaryRef,
            existing: *mut io_iterator_t,
        ) -> kern_return_t;
        fn IOIteratorNext(iterator: io_iterator_t) -> io_object_t;
        fn IORegistryEntryCreateCFProperties(
            entry: io_registry_entry_t,
            properties: *mut CFMutableDictionaryRef,
            allocator: *const c_void,
            options: u32,
        ) -> kern_return_t;
        fn IOObjectRelease(object: io_object_t) -> kern_return_t;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFNumberGetValue(number: CFTypeRef, the_type: isize, value_ptr: *mut c_void) -> u8;
    }

    fn cfstr(val: &str) -> CFStringRef {
        unsafe {
            CFStringCreateWithBytesNoCopy(
                kCFAllocatorDefault,
                val.as_ptr(),
                val.len() as isize,
                kCFStringEncodingUTF8,
                0,
                core_foundation::base::kCFAllocatorNull,
            )
        }
    }

    fn cfdict_get_val(dict: CFDictionaryRef, key: &str) -> Option<CFTypeRef> {
        unsafe {
            let key = cfstr(key);
            let val = CFDictionaryGetValue(dict, key as _);
            CFRelease(key as _);

            if val.is_null() {
                None
            } else {
                Some(val)
            }
        }
    }

    fn read_gpu_utilization(service: io_registry_entry_t) -> Option<f32> {
        unsafe {
            let mut props: CFMutableDictionaryRef = std::ptr::null_mut();
            let kr = IORegistryEntryCreateCFProperties(
                service,
                &mut props,
                kCFAllocatorDefault as *const c_void,
                0,
            );

            if kr != KERN_SUCCESS || props.is_null() {
                return None;
            }

            let result = cfdict_get_val(props as CFDictionaryRef, "PerformanceStatistics")
                .and_then(|stats_ptr| {
                    let util_ref =
                        cfdict_get_val(stats_ptr as CFDictionaryRef, "Device Utilization %")?;
                    let mut value: i64 = 0;
                    let ok = CFNumberGetValue(
                        util_ref,
                        CF_NUMBER_SINT64_TYPE,
                        &mut value as *mut i64 as *mut c_void,
                    );
                    if ok != 0 {
                        Some(value.clamp(0, 100) as f32)
                    } else {
                        None
                    }
                });

            CFRelease(props as CFTypeRef);
            result
        }
    }

    pub struct GpuSampler {
        service: io_registry_entry_t,
    }

    impl GpuSampler {
        pub fn new() -> Option<Self> {
            unsafe {
                let matching = IOServiceMatching(b"IOAccelerator\0".as_ptr().cast());
                if matching.is_null() {
                    return None;
                }

                let mut iterator: io_iterator_t = IO_OBJECT_NULL;
                let kr = IOServiceGetMatchingServices(
                    0,
                    matching as CFDictionaryRef,
                    &mut iterator,
                );
                if kr != KERN_SUCCESS || iterator == IO_OBJECT_NULL {
                    return None;
                }

                let service = IOIteratorNext(iterator);
                IOObjectRelease(iterator);

                if service == IO_OBJECT_NULL {
                    return None;
                }

                // Verify this service actually has PerformanceStatistics
                if read_gpu_utilization(service).is_none() {
                    IOObjectRelease(service);
                    return None;
                }

                Some(Self { service })
            }
        }

        pub fn sample(&mut self) -> Option<f32> {
            read_gpu_utilization(self.service)
        }
    }

    impl Drop for GpuSampler {
        fn drop(&mut self) {
            unsafe {
                IOObjectRelease(self.service);
            }
        }
    }

    unsafe impl Send for GpuSampler {}
}

// ============================================================================
// Linux Implementation (NVIDIA via NVML)
// ============================================================================

#[cfg(target_os = "linux")]
mod linux {
    use nvml_wrapper::Nvml;

    pub struct GpuSampler {
        nvml: Nvml,
        device_count: u32,
    }

    impl GpuSampler {
        /// Creates a new GPU sampler for NVIDIA GPUs via NVML.
        /// Returns None if NVML cannot be initialized (no NVIDIA driver) or no GPU found.
        pub fn new() -> Option<Self> {
            let nvml = Nvml::init().ok()?;
            let device_count = nvml.device_count().ok()?;
            if device_count == 0 {
                return None;
            }

            Some(Self { nvml, device_count })
        }

        /// Samples current GPU utilization percentage (max across all NVIDIA GPUs).
        pub fn sample(&mut self) -> Option<f32> {
            (0..self.device_count)
                .filter_map(|i| {
                    self.nvml
                        .device_by_index(i)
                        .ok()
                        .and_then(|d| d.utilization_rates().ok())
                        .map(|r| r.gpu as f32)
                })
                .reduce(f32::max)
        }
    }

    unsafe impl Send for GpuSampler {}
}

// ============================================================================
// Re-export platform-specific implementation
// ============================================================================

#[cfg(target_os = "macos")]
pub use macos::GpuSampler;

#[cfg(target_os = "linux")]
pub use linux::GpuSampler;
