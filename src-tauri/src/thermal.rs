use std::time::{Duration, Instant};

use objc2_foundation::{NSProcessInfo, NSProcessInfoThermalState};
use objc2_io_kit::{
    kIOPMThermalWarningLevelCrisis, kIOPMThermalWarningLevelDanger, kIOPMThermalWarningLevelNormal,
    kIOReturnSuccess, IOPMGetThermalWarningLevel,
};

const LOWER_READINGS_REQUIRED: u8 = 2;
pub const POLL_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ThermalStatus {
    Unavailable,
    Nominal,
    Fair,
    Serious,
    Critical,
}

impl ThermalStatus {
    fn rank(self) -> Option<u8> {
        match self {
            Self::Unavailable => None,
            Self::Nominal => Some(0),
            Self::Fair => Some(1),
            Self::Serious => Some(2),
            Self::Critical => Some(3),
        }
    }

    pub fn from_process_raw(raw: isize) -> Option<Self> {
        match raw {
            value if value == NSProcessInfoThermalState::Nominal.0 => Some(Self::Nominal),
            value if value == NSProcessInfoThermalState::Fair.0 => Some(Self::Fair),
            value if value == NSProcessInfoThermalState::Serious.0 => Some(Self::Serious),
            value if value == NSProcessInfoThermalState::Critical.0 => Some(Self::Critical),
            _ => None,
        }
    }

    pub fn from_iopm_level(level: u32) -> Option<Self> {
        match level {
            value if value == kIOPMThermalWarningLevelNormal => Some(Self::Nominal),
            value if value == kIOPMThermalWarningLevelDanger => Some(Self::Serious),
            value if value == kIOPMThermalWarningLevelCrisis => Some(Self::Critical),
            _ => None,
        }
    }
}

pub fn normalize(process: Option<ThermalStatus>, iopm: Option<ThermalStatus>) -> ThermalStatus {
    match (process, iopm) {
        (Some(primary), Some(power)) => {
            if primary.rank() >= power.rank() {
                primary
            } else {
                power
            }
        }
        (Some(status), None) | (None, Some(status)) => status,
        (None, None) => ThermalStatus::Unavailable,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ThermalTracker {
    displayed: ThermalStatus,
    lower_readings: u8,
}

impl ThermalTracker {
    pub fn new(initial: ThermalStatus) -> Self {
        Self {
            displayed: initial,
            lower_readings: 0,
        }
    }

    pub fn displayed(self) -> ThermalStatus {
        self.displayed
    }

    pub fn observe(&mut self, next: ThermalStatus) -> bool {
        let previous = self.displayed;

        match (previous.rank(), next.rank()) {
            (_, None) => {
                self.displayed = ThermalStatus::Unavailable;
                self.lower_readings = 0;
            }
            (None, Some(_)) => {
                self.displayed = next;
                self.lower_readings = 0;
            }
            (Some(previous_rank), Some(next_rank)) if next_rank > previous_rank => {
                self.displayed = next;
                self.lower_readings = 0;
            }
            (Some(previous_rank), Some(next_rank)) if next_rank < previous_rank => {
                self.lower_readings = self.lower_readings.saturating_add(1);
                if self.lower_readings >= LOWER_READINGS_REQUIRED {
                    self.displayed = next;
                    self.lower_readings = 0;
                }
            }
            (Some(_), Some(_)) => {
                self.lower_readings = 0;
            }
        }

        self.displayed != previous
    }
}

pub fn should_poll(enabled: bool, last_sample: Option<Instant>, now: Instant) -> bool {
    enabled
        && last_sample
            .map(|last| now.duration_since(last) >= POLL_INTERVAL)
            .unwrap_or(true)
}

fn normalize_iopm_result(return_code: i32, level: u32) -> Option<ThermalStatus> {
    (return_code == kIOReturnSuccess).then(|| ThermalStatus::from_iopm_level(level))?
}

pub fn sample() -> ThermalStatus {
    let process = NSProcessInfo::processInfo().thermalState();
    let process = ThermalStatus::from_process_raw(process.0);

    let mut level = 0_u32;
    let iopm = unsafe { IOPMGetThermalWarningLevel(&mut level) };
    let iopm = normalize_iopm_result(iopm, level);

    normalize(process, iopm)
}

#[cfg(test)]
mod tests {
    use super::*;
    use objc2_io_kit::kIOReturnNotFound;

    #[test]
    fn normalizes_each_provider_and_uses_the_highest_state() {
        assert_eq!(
            normalize(Some(ThermalStatus::Nominal), None),
            ThermalStatus::Nominal
        );
        assert_eq!(
            normalize(None, Some(ThermalStatus::Critical)),
            ThermalStatus::Critical
        );
        assert_eq!(
            normalize(Some(ThermalStatus::Fair), Some(ThermalStatus::Serious)),
            ThermalStatus::Serious
        );
        assert_eq!(
            normalize(Some(ThermalStatus::Critical), Some(ThermalStatus::Nominal)),
            ThermalStatus::Critical
        );
        assert_eq!(normalize(None, None), ThermalStatus::Unavailable);
    }

    #[test]
    fn maps_public_raw_values_and_rejects_unknown_values() {
        assert_eq!(
            ThermalStatus::from_process_raw(NSProcessInfoThermalState::Nominal.0),
            Some(ThermalStatus::Nominal)
        );
        assert_eq!(
            ThermalStatus::from_process_raw(NSProcessInfoThermalState::Critical.0),
            Some(ThermalStatus::Critical)
        );
        assert_eq!(ThermalStatus::from_process_raw(99), None);
        assert_eq!(
            ThermalStatus::from_iopm_level(kIOPMThermalWarningLevelNormal),
            Some(ThermalStatus::Nominal)
        );
        assert_eq!(
            ThermalStatus::from_iopm_level(kIOPMThermalWarningLevelDanger),
            Some(ThermalStatus::Serious)
        );
        assert_eq!(ThermalStatus::from_iopm_level(99), None);
    }

    #[test]
    fn ignores_iopm_failures_and_unknown_levels() {
        assert_eq!(
            normalize_iopm_result(kIOReturnSuccess, kIOPMThermalWarningLevelNormal),
            Some(ThermalStatus::Nominal)
        );
        assert_eq!(normalize_iopm_result(kIOReturnNotFound as i32, 0), None);
        assert_eq!(normalize_iopm_result(kIOReturnSuccess, 99), None);
    }

    #[test]
    fn tracker_promotes_immediately_and_demotes_after_two_lower_reads() {
        let mut tracker = ThermalTracker::new(ThermalStatus::Nominal);
        assert!(tracker.observe(ThermalStatus::Critical));
        assert_eq!(tracker.displayed(), ThermalStatus::Critical);
        assert!(!tracker.observe(ThermalStatus::Fair));
        assert_eq!(tracker.displayed(), ThermalStatus::Critical);
        assert!(tracker.observe(ThermalStatus::Fair));
        assert_eq!(tracker.displayed(), ThermalStatus::Fair);
    }

    #[test]
    fn tracker_resets_recovery_after_same_or_higher_reading() {
        let mut tracker = ThermalTracker::new(ThermalStatus::Critical);
        assert!(!tracker.observe(ThermalStatus::Fair));
        assert!(!tracker.observe(ThermalStatus::Critical));
        assert!(!tracker.observe(ThermalStatus::Fair));
        assert!(tracker.observe(ThermalStatus::Fair));
    }

    #[test]
    fn tracker_hides_unavailable_immediately() {
        let mut tracker = ThermalTracker::new(ThermalStatus::Serious);
        assert!(tracker.observe(ThermalStatus::Unavailable));
        assert_eq!(tracker.displayed(), ThermalStatus::Unavailable);
    }

    #[test]
    fn poll_is_immediate_then_waits_for_the_interval() {
        let now = Instant::now();
        assert!(should_poll(true, None, now));
        assert!(!should_poll(true, Some(now), now + Duration::from_secs(4)));
        assert!(should_poll(true, Some(now), now + POLL_INTERVAL));
        assert!(!should_poll(false, None, now));
    }
}
