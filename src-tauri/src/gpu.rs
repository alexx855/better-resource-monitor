//! GPU utilization monitoring
//!
//! Platform-specific implementations:
//! - macOS: Uses IOReport framework for Apple Silicon GPU residency
//! - Linux: Uses NVML (NVIDIA Management Library) for NVIDIA GPU utilization

// ============================================================================
// macOS Implementation (Apple Silicon via IOReport)
// ============================================================================

#[cfg(target_os = "macos")]
mod macos {
    use std::ffi::c_void;
    use std::marker::{PhantomData, PhantomPinned};
    use std::mem::MaybeUninit;
    use std::ptr::null;

    use core_foundation::array::{CFArrayGetCount, CFArrayGetValueAtIndex, CFArrayRef};
    use core_foundation::base::{kCFAllocatorDefault, CFRelease, CFTypeRef};
    use core_foundation::dictionary::{
        CFDictionaryCreateMutableCopy, CFDictionaryGetCount, CFDictionaryGetValue, CFDictionaryRef,
        CFMutableDictionaryRef,
    };
    use core_foundation::string::{
        kCFStringEncodingUTF8, CFStringCreateWithBytesNoCopy, CFStringGetCString, CFStringRef,
    };

    type CVoidRef = *const c_void;

    #[repr(C)]
    struct IOReportSubscription {
        _data: [u8; 0],
        _phantom: PhantomData<(*mut u8, PhantomPinned)>,
    }

    type IOReportSubscriptionRef = *const IOReportSubscription;

    #[link(name = "IOReport")]
    unsafe extern "C" {
        fn IOReportCopyChannelsInGroup(
            group: CFStringRef,
            subgroup: CFStringRef,
            c: u64,
            d: u64,
            e: u64,
        ) -> CFDictionaryRef;

        fn IOReportCreateSubscription(
            a: CVoidRef,
            b: CFMutableDictionaryRef,
            c: *mut CFMutableDictionaryRef,
            d: u64,
            e: CFTypeRef,
        ) -> IOReportSubscriptionRef;
        fn IOReportCreateSamples(
            a: IOReportSubscriptionRef,
            b: CFMutableDictionaryRef,
            c: CFTypeRef,
        ) -> CFDictionaryRef;
        fn IOReportCreateSamplesDelta(
            a: CFDictionaryRef,
            b: CFDictionaryRef,
            c: CFTypeRef,
        ) -> CFDictionaryRef;
        fn IOReportChannelGetChannelName(a: CFDictionaryRef) -> CFStringRef;
        fn IOReportChannelGetSubGroup(a: CFDictionaryRef) -> CFStringRef;
        fn IOReportStateGetCount(a: CFDictionaryRef) -> i32;
        fn IOReportStateGetNameForIndex(a: CFDictionaryRef, b: i32) -> CFStringRef;
        fn IOReportStateGetResidency(a: CFDictionaryRef, b: i32) -> i64;
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

    fn from_cfstr(val: CFStringRef) -> String {
        if val.is_null() {
            return String::new();
        }
        unsafe {
            let mut buf = [0i8; 128];
            if CFStringGetCString(val, buf.as_mut_ptr(), 128, kCFStringEncodingUTF8) != 0 {
                std::ffi::CStr::from_ptr(buf.as_ptr())
                    .to_string_lossy()
                    .to_string()
            } else {
                String::new()
            }
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

    pub struct GpuSampler {
        subs: IOReportSubscriptionRef,
        chan: CFMutableDictionaryRef,
        prev_sample: Option<CFDictionaryRef>,
    }

    impl GpuSampler {
        pub fn new() -> Option<Self> {
            let group_name = cfstr("GPU Stats");
            let subgroup_name = cfstr("GPU Performance States");

            let chan = unsafe { IOReportCopyChannelsInGroup(group_name, subgroup_name, 0, 0, 0) };
            unsafe {
                CFRelease(group_name as _);
                CFRelease(subgroup_name as _);
            }

            if chan.is_null() {
                return None;
            }

            if cfdict_get_val(chan, "IOReportChannels").is_none() {
                unsafe { CFRelease(chan as _) };
                return None;
            }

            let size = unsafe { CFDictionaryGetCount(chan) };
            let mutable_chan = unsafe { CFDictionaryCreateMutableCopy(kCFAllocatorDefault, size, chan) };
            unsafe { CFRelease(chan as _) };

            let mut s: MaybeUninit<CFMutableDictionaryRef> = MaybeUninit::uninit();
            let subs = unsafe { IOReportCreateSubscription(null(), mutable_chan, s.as_mut_ptr(), 0, null()) };

            if subs.is_null() {
                unsafe { CFRelease(mutable_chan as _) };
                return None;
            }

            Some(Self { subs, chan: mutable_chan, prev_sample: None })
        }

        pub fn sample(&mut self) -> f32 {
            unsafe {
                let current = IOReportCreateSamples(self.subs, self.chan, null());
                if current.is_null() {
                    return 0.0;
                }

                let usage = if let Some(prev) = self.prev_sample {
                    let delta = IOReportCreateSamplesDelta(prev, current, null());
                    CFRelease(prev as _);
                    
                    if delta.is_null() {
                        0.0
                    } else {
                        let usage = self.calculate_gpu_usage(delta);
                        CFRelease(delta as _);
                        usage
                    }
                } else {
                    0.0
                };

                self.prev_sample = Some(current);
                usage
            }
        }

        fn calculate_gpu_usage(&self, delta: CFDictionaryRef) -> f32 {
            let items = match cfdict_get_val(delta, "IOReportChannels") {
                Some(v) => v as CFArrayRef,
                None => return 0.0,
            };

            let count = unsafe { CFArrayGetCount(items) };

            for i in 0..count {
                let item = unsafe { CFArrayGetValueAtIndex(items, i) } as CFDictionaryRef;
                if item.is_null() {
                    continue;
                }

                let subgroup = unsafe { IOReportChannelGetSubGroup(item) };
                let subgroup_str = from_cfstr(subgroup);
                if subgroup_str != "GPU Performance States" {
                    continue;
                }

                let channel_name = unsafe { IOReportChannelGetChannelName(item) };
                let channel_str = from_cfstr(channel_name);
                    if channel_str != "GPUPH" {
                    continue;
                }

                return self.calc_residency_usage(item);
            }

            0.0
        }

        fn calc_residency_usage(&self, item: CFDictionaryRef) -> f32 {
            let state_count = unsafe { IOReportStateGetCount(item) };
            if state_count <= 0 {
                return 0.0;
            }

            let mut total_idle: i64 = 0;
            let mut total_active: i64 = 0;

            for s in 0..state_count {
                let state_name = unsafe { IOReportStateGetNameForIndex(item, s) };
                let state_name_str = from_cfstr(state_name);
                let residency = unsafe { IOReportStateGetResidency(item, s) };

                if state_name_str == "OFF" || state_name_str == "IDLE" || state_name_str == "DOWN" {
                    total_idle += residency;
                } else {
                    total_active += residency;
                }
            }

            let total = total_active + total_idle;
            if total > 0 {
                (total_active as f64 / total as f64 * 100.0) as f32
            } else {
                0.0
            }
        }
    }

    impl Drop for GpuSampler {
        fn drop(&mut self) {
            unsafe {
                if let Some(prev) = self.prev_sample {
                    CFRelease(prev as _);
                }
                CFRelease(self.chan as _);
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
        device_index: u32,
    }

    impl GpuSampler {
        /// Creates a new GPU sampler for NVIDIA GPUs via NVML.
        /// Returns None if NVML cannot be initialized (no NVIDIA driver) or no GPU found.
        pub fn new() -> Option<Self> {
            let nvml = Nvml::init().ok()?;
            
            // Check if at least one device exists
            let device_count = nvml.device_count().ok()?;
            if device_count == 0 {
                return None;
            }

            // Use first GPU (index 0)
            Some(Self {
                nvml,
                device_index: 0,
            })
        }

        /// Samples current GPU utilization percentage.
        /// Returns 0.0 if sampling fails.
        pub fn sample(&mut self) -> f32 {
            self.nvml
                .device_by_index(self.device_index)
                .ok()
                .and_then(|device| device.utilization_rates().ok())
                .map(|rates| rates.gpu as f32)
                .unwrap_or(0.0)
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