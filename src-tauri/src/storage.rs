use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct StorageSample {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_percent: f32,
}

/// Samples the volume that contains `path`. Passing [`home_path_or_root`]
/// gives the "Startup disk" choice in the tray menu, which is the default.
pub(crate) fn sample_at(path: &Path) -> Option<StorageSample> {
    platform_sample(path)
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

pub(crate) fn home_path_or_root() -> PathBuf {
    std::env::var_os("HOME")
        .filter(|home| !home.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

#[cfg(target_os = "macos")]
fn platform_sample(path: &Path) -> Option<StorageSample> {
    use core::ffi::c_void;
    use core_foundation::array::CFArray;
    use core_foundation::base::TCFType;
    use core_foundation::dictionary::{CFDictionary, CFDictionaryGetValueIfPresent};
    use core_foundation::number::{CFNumber, CFNumberRef};
    use core_foundation::string::CFStringRef;
    use core_foundation::url::{
        kCFURLVolumeAvailableCapacityKey, kCFURLVolumeTotalCapacityKey,
        CFURLCopyResourcePropertiesForKeys, CFURL,
    };
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

    let url = CFURL::from_path(path, true)?;
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
fn platform_sample(path: &Path) -> Option<StorageSample> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

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
fn platform_sample(_path: &Path) -> Option<StorageSample> {
    None
}

/// A mounted volume the user can pick as the storage metric's source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Volume {
    /// Name shown in the tray menu, such as "Backup".
    pub label: String,
    /// Where the volume is mounted. This is what the setting stores, because
    /// it stays the same when the same drive is plugged in again.
    pub mount_point: PathBuf,
}

/// How a platform lays out its mount points, which decides which volumes are
/// worth offering in the menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MountLayout {
    /// Extra volumes live under `/Volumes`. `/` and `/System/Volumes/*` share
    /// the startup disk's APFS container, so they would only repeat the
    /// "Startup disk" numbers.
    #[cfg(any(test, target_os = "macos"))]
    MacOs,
    /// Volumes can be mounted almost anywhere, so everything sysinfo reports
    /// is offered except boot and snap package mounts.
    #[cfg(any(test, not(target_os = "macos")))]
    Linux,
}

#[cfg(target_os = "macos")]
pub(crate) const CURRENT_MOUNT_LAYOUT: MountLayout = MountLayout::MacOs;
#[cfg(not(target_os = "macos"))]
pub(crate) const CURRENT_MOUNT_LAYOUT: MountLayout = MountLayout::Linux;

/// Returns true when a volume mounted at `mount_point` should be offered in
/// the storage disk menu.
pub(crate) fn is_selectable_mount_point(mount_point: &Path, layout: MountLayout) -> bool {
    match layout {
        #[cfg(any(test, target_os = "macos"))]
        MountLayout::MacOs => {
            mount_point.starts_with("/Volumes") && mount_point != Path::new("/Volumes")
        }
        #[cfg(any(test, not(target_os = "macos")))]
        MountLayout::Linux => {
            const HIDDEN_PREFIXES: [&str; 5] =
                ["/boot", "/efi", "/snap", "/var/snap", "/var/lib/snapd"];
            mount_point.is_absolute()
                && !HIDDEN_PREFIXES
                    .iter()
                    .any(|prefix| mount_point.starts_with(prefix))
        }
    }
}

/// Returns true for file systems that live on another machine, such as
/// `fuse.sshfs`, `smb3` or `ceph`, compared case-insensitively.
///
/// These are kept out of the storage disk menu. Checking the
/// capacity of a hard-mounted share whose server has gone away can block for
/// minutes, and the chosen disk is checked on every update, so offering one
/// could freeze the whole tray.
///
/// sysinfo already leaves out `nfs`, `nfs4`, `cifs` and `autofs` on Linux, and
/// leaves out every volume macOS does not report as local. They are listed
/// here anyway so the menu stays safe if a sysinfo update or feature flag
/// changes that.
///
/// Local FUSE file systems, such as `fuseblk` (NTFS and exFAT drives),
/// `fuse.mergerfs` pools, and encrypted folders, are still offered.
pub(crate) fn is_network_file_system(file_system: &OsStr) -> bool {
    const NETWORK_FILE_SYSTEMS: &[&str] = &[
        // Kernel network and cluster file systems (Linux).
        "nfs",
        "nfs4",
        "cifs",
        "smb3",
        "smbfs",
        "autofs",
        "9p",
        "afs",
        "ceph",
        "coda",
        "davfs",
        "glusterfs",
        "lustre",
        "ncpfs",
        "gpfs",
        "beegfs",
        // Network file systems (macOS).
        "afpfs",
        "webdav",
        // FUSE clients for remote storage, named `fuse.<program>` on Linux.
        "fuse.sshfs",
        "fuse.rclone",
        "fuse.s3fs",
        "fuse.gvfsd-fuse",
        "fuse.curlftpfs",
    ];
    const NETWORK_FUSE_PREFIXES: &[&str] = &[
        "fuse.ceph",
        "fuse.glusterfs",
        "fuse.juicefs",
        "fuse.gcsfuse",
    ];
    let file_system = file_system.to_string_lossy().to_ascii_lowercase();
    NETWORK_FILE_SYSTEMS.contains(&file_system.as_str())
        || NETWORK_FUSE_PREFIXES
            .iter()
            .any(|prefix| file_system.starts_with(prefix))
}

/// Returns the mount point that holds `path`: the longest mount point that
/// `path` starts with.
pub(crate) fn mount_point_containing<'a>(
    path: &Path,
    mount_points: impl IntoIterator<Item = &'a Path>,
) -> Option<&'a Path> {
    mount_points
        .into_iter()
        .filter(|mount_point| path.starts_with(mount_point))
        .max_by_key(|mount_point| mount_point.components().count())
}

/// The label for a volume that is no longer mounted, taken from the last part
/// of its mount point (`/Volumes/Backup` becomes "Backup").
pub(crate) fn label_for_mount_point(mount_point: &Path) -> String {
    mount_point
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| mount_point.display().to_string())
}

/// Turns a raw list of `(volume name, mount point)` pairs into the volumes the
/// menu offers, sorted by label.
///
/// The volume holding `home` is left out, because the "Startup disk" choice
/// already covers it. Volumes without a usable name fall back to their mount
/// point's last part, and two volumes with the same name (two drives both
/// called "Untitled") are told apart the same way, because macOS gives each
/// one its own mount point.
pub(crate) fn selectable_volumes<'a>(
    raw: impl IntoIterator<Item = (&'a OsStr, &'a Path)>,
    home: &Path,
    layout: MountLayout,
) -> Vec<Volume> {
    let raw: Vec<(&OsStr, &Path)> = raw.into_iter().collect();
    let home_mount = mount_point_containing(home, raw.iter().map(|(_, mount)| *mount));

    let mut seen = HashSet::new();
    let mut volumes: Vec<Volume> = raw
        .into_iter()
        .filter(|(_, mount)| Some(*mount) != home_mount)
        .filter(|(_, mount)| is_selectable_mount_point(mount, layout))
        .filter(|(_, mount)| seen.insert(mount.to_path_buf()))
        .map(|(name, mount)| {
            let name = name.to_string_lossy();
            // Linux reports device paths such as /dev/sdb1 as names.
            let label = if name.trim().is_empty() || name.starts_with("/dev/") {
                label_for_mount_point(mount)
            } else {
                name.into_owned()
            };
            Volume {
                label,
                mount_point: mount.to_path_buf(),
            }
        })
        .collect();

    let labels: Vec<String> = volumes.iter().map(|volume| volume.label.clone()).collect();
    for volume in &mut volumes {
        if labels
            .iter()
            .filter(|label| **label == volume.label)
            .count()
            > 1
        {
            volume.label = label_for_mount_point(&volume.mount_point);
        }
    }

    volumes.sort_by(|a, b| {
        a.label
            .to_lowercase()
            .cmp(&b.label.to_lowercase())
            .then_with(|| a.mount_point.cmp(&b.mount_point))
    });
    volumes
}

/// Decides which path to sample: the chosen volume while it is mounted,
/// otherwise the home folder's volume. Keeping the choice when a drive is
/// unplugged means it comes back by itself when the drive is plugged in again.
pub(crate) fn resolve_sample_path(
    selected: Option<&Path>,
    volumes: &[Volume],
    home: &Path,
) -> PathBuf {
    selected
        .filter(|selected| {
            volumes
                .iter()
                .any(|volume| volume.mount_point.as_path() == *selected)
        })
        .map(Path::to_path_buf)
        .unwrap_or_else(|| home.to_path_buf())
}

/// Keeps the list of mounted volumes up to date. Listing volumes is cheap but
/// not free, and it can block on a stale mount, so a background thread calls
/// [`VolumeWatcher::refresh`] every few seconds, never the monitor loop.
pub(crate) struct VolumeWatcher {
    disks: sysinfo::Disks,
}

impl VolumeWatcher {
    pub(crate) fn new() -> Self {
        Self {
            disks: sysinfo::Disks::new(),
        }
    }

    /// Re-reads the mounted volumes and returns the ones the menu offers.
    pub(crate) fn refresh(&mut self) -> Vec<Volume> {
        // Only names and mount points are needed; capacity comes from
        // `sample_at` so every choice is measured the same way.
        self.disks
            .refresh_specifics(true, sysinfo::DiskRefreshKind::nothing());
        let mut volumes = selectable_volumes(
            self.disks
                .list()
                .iter()
                .filter(|disk| !is_network_file_system(disk.file_system()))
                .map(|disk| (disk.name(), disk.mount_point())),
            &home_path_or_root(),
            CURRENT_MOUNT_LAYOUT,
        );
        // Leave out volumes that will not report their capacity, such as an
        // unreadable FUSE mount, so every choice in the menu works.
        volumes.retain(|volume| sample_at(&volume.mount_point).is_some());
        volumes
    }
}
