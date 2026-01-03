//! GPU utilization monitoring for Apple Silicon using IOReport
//!
//! This module provides GPU utilization percentage by sampling the "GPU Stats" / "GPU Performance States"
//! channel from IOReport and calculating active residency.

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

// Type aliases
type CVoidRef = *const c_void;

#[repr(C)]
struct IOReportSubscription {
    _data: [u8; 0],
    _phantom: PhantomData<(*mut u8, PhantomPinned)>,
}

type IOReportSubscriptionRef = *const IOReportSubscription;

// IOReport bindings - these are in libIOReport.dylib, not IOKit.framework
#[link(name = "IOReport")]
unsafe extern "C" {
    fn IOReportCopyChannelsInGroup(
        group: CFStringRef,
        subgroup: CFStringRef,
        c: u64,
        d: u64,
        e: u64,
    ) -> CFDictionaryRef;
    fn IOReportMergeChannels(a: CFDictionaryRef, b: CFDictionaryRef, nil: CFTypeRef);
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
            0, // false as u8 for C
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

/// GPU sampler that uses IOReport to measure GPU utilization
pub struct GpuSampler {
    subs: IOReportSubscriptionRef,
    chan: CFMutableDictionaryRef,
}

impl GpuSampler {
    /// Create a new GPU sampler. Returns None if IOReport initialization fails.
    pub fn new() -> Option<Self> {
        // Get GPU Stats channels with "GPU Performance States" subgroup
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

        // Check if we got valid channels
        if cfdict_get_val(chan, "IOReportChannels").is_none() {
            unsafe { CFRelease(chan as _) };
            return None;
        }

        let size = unsafe { CFDictionaryGetCount(chan) };
        let chan = unsafe { CFDictionaryCreateMutableCopy(kCFAllocatorDefault, size, chan) };

        let mut s: MaybeUninit<CFMutableDictionaryRef> = MaybeUninit::uninit();
        let subs = unsafe { IOReportCreateSubscription(null(), chan, s.as_mut_ptr(), 0, null()) };

        if subs.is_null() {
            unsafe { CFRelease(chan as _) };
            return None;
        }

        Some(Self { subs, chan })
    }

    /// Sample GPU utilization over the given duration (in milliseconds).
    /// Returns the GPU usage percentage (0.0 - 100.0).
    pub fn sample(&self, duration_ms: u64) -> f32 {
        unsafe {
            let sample1 = IOReportCreateSamples(self.subs, self.chan, null());
            if sample1.is_null() {
                return 0.0;
            }

            std::thread::sleep(std::time::Duration::from_millis(duration_ms));

            let sample2 = IOReportCreateSamples(self.subs, self.chan, null());
            if sample2.is_null() {
                CFRelease(sample1 as _);
                return 0.0;
            }

            let delta = IOReportCreateSamplesDelta(sample1, sample2, null());
            CFRelease(sample1 as _);
            CFRelease(sample2 as _);

            if delta.is_null() {
                return 0.0;
            }

            let usage = self.calculate_gpu_usage(delta);
            CFRelease(delta as _);
            usage
        }
    }

    /// Calculate GPU usage from delta sample
    /// Looks for GPUPH channel in "GPU Performance States" subgroup
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

            // Check subgroup
            let subgroup = unsafe { IOReportChannelGetSubGroup(item) };
            let subgroup_str = from_cfstr(subgroup);
            if subgroup_str != "GPU Performance States" {
                continue;
            }

            // Check channel name - we want GPUPH
            let channel_name = unsafe { IOReportChannelGetChannelName(item) };
            let channel_str = from_cfstr(channel_name);
            if channel_str != "GPUPH" {
                continue;
            }

            // Found GPUPH - calculate usage from residencies
            return self.calc_residency_usage(item);
        }

        0.0
    }

    /// Calculate usage percentage from state residencies
    /// OFF/IDLE states are idle, everything else is active
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

            // OFF, IDLE are idle states - everything else is an active frequency state
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
            CFRelease(self.chan as _);
            // Note: IOReportSubscriptionRef doesn't need explicit release
        }
    }
}

// Safety: GpuSampler can be sent between threads
unsafe impl Send for GpuSampler {}
