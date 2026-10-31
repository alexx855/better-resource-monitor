use super::*;
use crate::i18n;
use std::sync::{Mutex, OnceLock};

const UPDATE_INTERVAL_ENV: &str = "SILICON_UPDATE_INTERVAL";

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct EnvVarRestore {
    key: &'static str,
    previous: Option<std::ffi::OsString>,
}

impl Drop for EnvVarRestore {
    fn drop(&mut self) {
        if let Some(value) = &self.previous {
            std::env::set_var(self.key, value);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

fn with_update_interval_env<R>(value: Option<&str>, assertion: impl FnOnce() -> R) -> R {
    let _guard = env_lock().lock().expect("env lock poisoned");
    let _restore = EnvVarRestore {
        key: UPDATE_INTERVAL_ENV,
        previous: std::env::var_os(UPDATE_INTERVAL_ENV),
    };

    if let Some(value) = value {
        std::env::set_var(UPDATE_INTERVAL_ENV, value);
    } else {
        std::env::remove_var(UPDATE_INTERVAL_ENV);
    }

    assertion()
}

fn base_render_config<'a>() -> tray_render::RenderConfig<'a> {
    tray_render::RenderConfig {
        sizing: APP_SIZING,
        cpu_usage: 50.0,
        mem_percent: 50.0,
        storage_available_str: "0.0 KB",
        storage_available_bytes: None,
        gpu_usage: 0.0,
        down_str: "0 KB",
        up_str: "0 KB",
        show_cpu: true,
        show_mem: true,
        show_storage: false,
        show_gpu: false,
        show_net: false,
        show_alerts: true,
        use_light_icons: true,
        background: None,
    }
}

fn assert_render_size(buffer: &[u8], width: u32, height: u32, expected_width: u32) {
    assert_eq!(width, expected_width);
    assert_eq!(height, APP_SIZING.icon_height);
    assert_eq!(buffer.len(), (width * height * 4) as usize);
}

fn visible_gap_between_icon_and_text(
    buffer: &[u8],
    image_width: u32,
    image_height: u32,
    segment_left: u32,
    segment_width: u32,
    icon_width: u32,
) -> u32 {
    let icon_right = (segment_left..segment_left + icon_width)
        .rev()
        .find(|&x| (0..image_height).any(|y| buffer[((y * image_width + x) * 4 + 3) as usize] > 0))
        .expect("rendered icon must contain visible pixels");
    let text_left = (segment_left + icon_width..segment_left + segment_width)
        .find(|&x| (0..image_height).any(|y| buffer[((y * image_width + x) * 4 + 3) as usize] > 0))
        .expect("rendered label must contain visible pixels");

    text_left
        .checked_sub(icon_right + 1)
        .expect("rendered label must not overlap its icon")
}

fn assert_sizing(sizing: tray_render::Sizing, expected: (u32, u32, u32, u32, u32, u32, f32)) {
    let (
        segment_width,
        segment_width_storage,
        segment_width_net,
        edge_padding,
        segment_gap,
        icon_height,
        font_size,
    ) = expected;

    assert_eq!(sizing.segment_width, segment_width);
    assert_eq!(sizing.segment_width_storage, segment_width_storage);
    assert_eq!(sizing.segment_width_net, segment_width_net);
    assert_eq!(sizing.edge_padding, edge_padding);
    assert_eq!(sizing.segment_gap, segment_gap);
    assert_eq!(sizing.icon_height, icon_height);
    assert_eq!(sizing.font_size, font_size);
}

#[test]
fn test_cap_percent() {
    for (input, expected) in [
        (0.0, 0.0),
        (50.0, 50.0),
        (99.0, 99.0),
        (100.0, 99.0),
        (150.0, 99.0),
        (-10.0, 0.0),
    ] {
        assert_eq!(tray_render::cap_percent(input), expected, "input={input}");
    }
}

#[test]
fn test_should_update_threshold() {
    for (previous, new, threshold, expected) in [
        (10.0, 12.0, 2.0, true),
        (10.0, 8.0, 2.0, true),
        (10.0, 12.001, 2.0, true),
        (10.0, 11.9, 2.0, false),
        (10.0, 9.1, 2.0, false),
        (10.0, 10.0, 2.0, false),
    ] {
        assert_eq!(
            should_update(previous, new, threshold),
            expected,
            "previous={previous}, new={new}, threshold={threshold}"
        );
    }
}

#[test]
fn test_normalize_metric_flags() {
    for (input, expected) in [
        (
            (false, false, true, false, false, true),
            (false, false, true, false, false),
        ),
        (
            (false, false, false, true, false, true),
            (false, false, false, true, false),
        ),
        (
            (false, false, false, true, false, false),
            (true, false, false, false, false),
        ),
        (
            (false, true, false, true, false, false),
            (false, true, false, false, false),
        ),
        (
            (false, false, false, false, false, true),
            (true, false, false, false, false),
        ),
        (
            (false, false, false, false, false, false),
            (true, false, false, false, false),
        ),
    ] {
        let (cpu, mem, storage, gpu, net, gpu_available) = input;
        assert_eq!(
            normalize_metric_flags(cpu, mem, storage, gpu, net, gpu_available),
            expected,
            "input={input:?}"
        );
    }
}

#[test]
fn test_storage_setting_migration() {
    for (stored_storage, has_legacy_metric_settings, expected) in [
        (Some(true), true, true),
        (Some(false), true, false),
        (Some(true), false, true),
        (Some(false), false, false),
        (None, false, true),
        (None, true, false),
    ] {
        assert_eq!(
            migrate_storage_setting(stored_storage, has_legacy_metric_settings),
            expected,
            "stored_storage={stored_storage:?}, has_legacy_metric_settings={has_legacy_metric_settings}"
        );
    }
}

#[test]
fn test_storage_sample_math() {
    let sample = storage::sample_from_bytes(1_000, 250).expect("valid sample");
    assert_eq!(sample.total_bytes, 1_000);
    assert_eq!(sample.available_bytes, 250);
    assert_eq!(sample.used_percent, 75.0);

    assert_eq!(storage::sample_from_bytes(0, 0), None);

    let clamped = storage::sample_from_bytes(1_000, 2_000).expect("clamped sample");
    assert_eq!(clamped.available_bytes, 1_000);
    assert_eq!(clamped.used_percent, 0.0);
}

#[test]
fn test_storage_block_math() {
    let sample = storage::sample_from_blocks(10, 4, 1024, 512).expect("valid statvfs sample");
    assert_eq!(sample.total_bytes, 10 * 1024);
    assert_eq!(sample.available_bytes, 4 * 1024);
    assert_eq!(sample.used_percent, 60.0);

    let fallback = storage::sample_from_blocks(10, 4, 0, 512).expect("block size fallback");
    assert_eq!(fallback.total_bytes, 10 * 512);
    assert_eq!(fallback.available_bytes, 4 * 512);

    assert_eq!(storage::sample_from_blocks(10, 4, 0, 0), None);

    let huge = storage::sample_from_blocks(u128::MAX, u128::MAX / 2, u128::MAX, 0)
        .expect("huge values saturate");
    assert_eq!(huge.total_bytes, u64::MAX);
    assert!(huge.used_percent <= 100.0);
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[test]
fn test_storage_sample_reads_home_filesystem() {
    let sample =
        storage::sample_at(&storage::home_path_or_root()).expect("home filesystem storage sample");
    assert!(sample.total_bytes > 0);
    assert!(sample.available_bytes <= sample.total_bytes);
    assert!((0.0..=100.0).contains(&sample.used_percent));
}

fn volume(label: &str, mount_point: &str) -> storage::Volume {
    storage::Volume {
        label: label.to_string(),
        mount_point: PathBuf::from(mount_point),
    }
}

fn selectable(
    raw: &[(&str, &str)],
    home: &str,
    layout: storage::MountLayout,
) -> Vec<storage::Volume> {
    storage::selectable_volumes(
        raw.iter()
            .map(|(name, mount)| (std::ffi::OsStr::new(*name), Path::new(*mount))),
        Path::new(home),
        layout,
    )
}

#[test]
fn test_selectable_mount_points_per_layout() {
    use storage::MountLayout::{Linux, MacOs};
    let cases = [
        ("/Volumes/Backup", MacOs, true),
        ("/Volumes/My Passport", MacOs, true),
        ("/Volumes", MacOs, false),
        ("/", MacOs, false),
        ("/System/Volumes/Data", MacOs, false),
        ("/System/Volumes/VM", MacOs, false),
        ("/VolumesBackup", MacOs, false),
        ("/", Linux, true),
        ("/home", Linux, true),
        ("/mnt/data", Linux, true),
        ("/media/adam/USB", Linux, true),
        ("/run/media/adam/USB", Linux, true),
        ("/boot", Linux, false),
        ("/boot/efi", Linux, false),
        ("/efi", Linux, false),
        ("/snap/core22/1380", Linux, false),
        ("/var/snap/lxd/common", Linux, false),
        ("relative/path", Linux, false),
    ];
    for (mount_point, layout, expected) in cases {
        assert_eq!(
            storage::is_selectable_mount_point(Path::new(mount_point), layout),
            expected,
            "mount_point={mount_point} layout={layout:?}"
        );
    }
}

#[test]
fn test_mount_point_containing_picks_the_deepest_mount() {
    let mounts = [
        Path::new("/"),
        Path::new("/home"),
        Path::new("/home/adam/data"),
    ];
    let containing = |path: &str| storage::mount_point_containing(Path::new(path), mounts);
    assert_eq!(containing("/home/adam"), Some(Path::new("/home")));
    assert_eq!(
        containing("/home/adam/data/x"),
        Some(Path::new("/home/adam/data"))
    );
    assert_eq!(containing("/etc"), Some(Path::new("/")));
    // Path::starts_with compares whole components, so /homework is not /home.
    assert_eq!(containing("/homework"), Some(Path::new("/")));
    assert_eq!(storage::mount_point_containing(Path::new("/x"), []), None);
}

#[test]
fn test_selectable_volumes_on_macos() {
    let volumes = selectable(
        &[
            ("Macintosh HD", "/"),
            ("Macintosh HD - Data", "/System/Volumes/Data"),
            ("Untitled", "/Volumes/Untitled"),
            ("backup", "/Volumes/backup"),
            ("Untitled", "/Volumes/Untitled 1"),
            ("Photos", "/Volumes/Photos"),
            ("Photos", "/Volumes/Photos"),
        ],
        "/Users/adam",
        storage::MountLayout::MacOs,
    );

    assert_eq!(
        volumes,
        vec![
            volume("backup", "/Volumes/backup"),
            volume("Photos", "/Volumes/Photos"),
            // Same name twice: told apart by the mount point macOS picked.
            volume("Untitled", "/Volumes/Untitled"),
            volume("Untitled 1", "/Volumes/Untitled 1"),
        ]
    );
}

#[test]
fn test_selectable_volumes_on_linux() {
    let volumes = selectable(
        &[
            ("/dev/nvme0n1p2", "/"),
            ("/dev/nvme0n1p3", "/home"),
            ("/dev/nvme0n1p1", "/boot/efi"),
            ("/dev/loop3", "/snap/core22/1380"),
            ("/dev/sdb1", "/media/adam/USB"),
            ("", "/mnt/scratch"),
        ],
        "/home/adam",
        storage::MountLayout::Linux,
    );

    // /home holds the home folder, so "Startup disk" already covers it.
    assert_eq!(
        volumes,
        vec![
            volume("/", "/"),
            volume("scratch", "/mnt/scratch"),
            volume("USB", "/media/adam/USB"),
        ]
    );
}

#[test]
fn test_network_file_systems_are_not_offered() {
    use std::ffi::OsStr;

    // Checking capacity on these can block for minutes when the server is gone.
    for file_system in [
        "nfs",
        "nfs4",
        "cifs",
        "smb3",
        "9p",
        "ceph",
        "afpfs",
        "webdav",
        "fuse.sshfs",
        "fuse.rclone",
        "fuse.gvfsd-fuse",
        "fuse.ceph-fuse",
        "fuse.juicefs",
        "FUSE.SSHFS",
    ] {
        assert!(
            storage::is_network_file_system(OsStr::new(file_system)),
            "{file_system} should count as a network file system"
        );
    }

    // Local disks, including FUSE ones such as NTFS drives and mergerfs pools,
    // stay in the menu.
    for file_system in [
        "ext4",
        "btrfs",
        "xfs",
        "vfat",
        "exfat",
        "apfs",
        "hfs",
        "fuseblk",
        "fuse.mergerfs",
        "fuse.gocryptfs",
        "",
    ] {
        assert!(
            !storage::is_network_file_system(OsStr::new(file_system)),
            "{file_system:?} should count as a local file system"
        );
    }
}

#[test]
fn test_resolve_sample_path_falls_back_to_home() {
    let home = Path::new("/Users/adam");
    let volumes = [volume("Backup", "/Volumes/Backup")];

    assert_eq!(storage::resolve_sample_path(None, &volumes, home), home);
    assert_eq!(
        storage::resolve_sample_path(Some(Path::new("/Volumes/Backup")), &volumes, home),
        Path::new("/Volumes/Backup")
    );
    // Unplugged: show the home folder's disk until it comes back.
    assert_eq!(
        storage::resolve_sample_path(Some(Path::new("/Volumes/Gone")), &volumes, home),
        home
    );
    assert_eq!(
        storage::resolve_sample_path(Some(Path::new("/Volumes/Backup")), &[], home),
        home
    );
}

#[test]
fn test_label_for_mount_point() {
    assert_eq!(
        storage::label_for_mount_point(Path::new("/Volumes/Backup")),
        "Backup"
    );
    assert_eq!(storage::label_for_mount_point(Path::new("/")), "/");
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[test]
fn test_volume_watcher_lists_mounted_volumes() {
    let mut watcher = storage::VolumeWatcher::new();
    let volumes = watcher.refresh();

    for volume in &volumes {
        assert!(volume.mount_point.is_absolute(), "{volume:?}");
        assert!(!volume.label.is_empty(), "{volume:?}");
        assert!(
            storage::is_selectable_mount_point(&volume.mount_point, storage::CURRENT_MOUNT_LAYOUT),
            "{volume:?}"
        );
        assert!(
            storage::sample_at(&volume.mount_point).is_some(),
            "listed volume can be sampled: {volume:?}"
        );
    }
    // A second refresh with nothing plugged in or out gives the same list.
    assert_eq!(watcher.refresh(), volumes);
}

#[test]
fn test_storage_sampler_rejects_previous_disk_and_failed_reads() {
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    let mut sampler = storage::StorageSampler::default();
    let disk_a = Path::new("/Volumes/A");
    let disk_b = Path::new("/Volumes/B");
    let sample_a = storage::sample_from_bytes(1_000, 250).unwrap();
    let (release_a, blocked_a) = mpsc::channel();
    assert_eq!(
        sampler.poll_with(disk_a, true, move |_| {
            blocked_a.recv().unwrap();
            Some(sample_a)
        }),
        None
    );
    // A blocked read must neither block polling nor start duplicate workers.
    let (duplicate, duplicates) = mpsc::channel();
    for _ in 0..10 {
        let duplicate = duplicate.clone();
        assert_eq!(
            sampler.poll_with(disk_a, true, move |_| {
                duplicate.send(()).unwrap();
                None
            }),
            None
        );
    }
    assert_eq!(
        duplicates.recv_timeout(Duration::from_millis(100)),
        Err(mpsc::RecvTimeoutError::Timeout)
    );
    assert_eq!(sampler.poll_with(disk_b, true, |_| None), None);
    release_a.send(()).unwrap();
    // A's late result must never appear while B is selected.
    let deadline = Instant::now() + Duration::from_millis(100);
    while Instant::now() < deadline {
        assert_eq!(sampler.poll_with(disk_b, false, |_| None), None);
        std::thread::yield_now();
    }
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if sampler.poll_with(disk_a, false, move |_| Some(sample_a)) == Some(sample_a) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "background disk read did not complete"
        );
        std::thread::yield_now();
    }
    // Even after A was displayed successfully, a failing B clears its value.
    assert_eq!(sampler.poll_with(disk_b, false, |_| None), None);
}

#[test]
fn test_storage_sampler_can_switch_away_from_a_blocked_disk() {
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    let mut sampler = storage::StorageSampler::default();
    let (release, blocked) = mpsc::channel();
    let stuck = Path::new("/Volumes/Stuck");
    let home = Path::new("/home/test");
    let sample = storage::sample_from_bytes(10_000, 8_000).unwrap();
    sampler.poll_with(stuck, true, move |_| {
        blocked.recv().unwrap();
        None
    });
    assert_eq!(sampler.poll_with(home, true, move |_| Some(sample)), None);
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if sampler.poll_with(home, false, |_| panic!("unexpected read")) == Some(sample) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "healthy disk was blocked by old disk"
        );
        std::thread::yield_now();
    }
    release.send(()).unwrap();
}

#[test]
fn test_storage_volume_menu_ids_round_trip() {
    assert_eq!(
        storage_volume_from_menu_id(STORAGE_VOLUME_STARTUP_ID),
        Some(None)
    );
    let id = storage_volume_menu_id(Path::new("/Volumes/My Passport"));
    assert_eq!(id, "storage_volume:/Volumes/My Passport");
    assert_eq!(
        storage_volume_from_menu_id(&id),
        Some(Some(PathBuf::from("/Volumes/My Passport")))
    );
    assert_eq!(storage_volume_from_menu_id(menu_id::SHOW_STORAGE), None);
    assert_eq!(storage_volume_from_menu_id(menu_id::QUIT), None);
    assert_eq!(storage_volume_from_menu_id("storage_volume:relative"), None);
}

#[test]
fn test_storage_menu_entries() {
    let t = i18n::Language::English.translations();
    let volumes = [
        volume("Backup", "/Volumes/Backup"),
        volume("Photos", "/Volumes/Photos"),
    ];
    let entry = |id: &str, label: &str, checked: bool, enabled: bool| StorageMenuEntry {
        id: id.to_string(),
        label: label.to_string(),
        checked,
        enabled,
    };

    // Hidden while storage is off, and when the startup disk is the only choice.
    assert!(storage_menu_entries(false, &volumes, None, t).is_empty());
    assert!(storage_menu_entries(true, &[], None, t).is_empty());

    assert_eq!(
        storage_menu_entries(true, &volumes, None, t),
        vec![
            entry(STORAGE_VOLUME_STARTUP_ID, "Startup disk", true, true),
            entry("storage_volume:/Volumes/Backup", "Backup", false, true),
            entry("storage_volume:/Volumes/Photos", "Photos", false, true),
        ]
    );

    assert_eq!(
        storage_menu_entries(true, &volumes, Some(Path::new("/Volumes/Photos")), t),
        vec![
            entry(STORAGE_VOLUME_STARTUP_ID, "Startup disk", false, true),
            entry("storage_volume:/Volumes/Backup", "Backup", false, true),
            entry("storage_volume:/Volumes/Photos", "Photos", true, true),
        ]
    );

    // An unplugged choice stays listed and greyed out, and the check mark
    // moves to the startup disk the tray falls back to.
    assert_eq!(
        storage_menu_entries(true, &volumes[..1], Some(Path::new("/Volumes/Photos")), t),
        vec![
            entry(STORAGE_VOLUME_STARTUP_ID, "Startup disk", true, true),
            entry("storage_volume:/Volumes/Backup", "Backup", false, true),
            entry(
                "storage_volume:/Volumes/Photos",
                "Photos (not connected)",
                false,
                false
            ),
        ]
    );
    assert_eq!(
        storage_menu_entries(true, &[], Some(Path::new("/Volumes/Photos")), t).len(),
        2,
        "an unplugged choice keeps the submenu visible so it can be cleared"
    );
}

#[test]
fn test_storage_disk_translations_match_supported_locales() {
    let expected = [
        (
            i18n::Language::English,
            ["Storage disk", "Startup disk", "not connected"],
        ),
        (
            i18n::Language::Spanish,
            [
                "Disco de almacenamiento",
                "Disco de arranque",
                "no conectado",
            ],
        ),
        (
            i18n::Language::Portuguese,
            [
                "Disco de armazenamento",
                "Disco de inicialização",
                "não conectado",
            ],
        ),
        (i18n::Language::Chinese, ["存储磁盘", "启动磁盘", "未连接"]),
        (
            i18n::Language::Polish,
            ["Dysk", "Dysk startowy", "niepodłączony"],
        ),
    ];

    for (language, expected) in expected {
        let t = language.translations();
        assert_eq!(
            [
                t.storage_disk,
                t.storage_disk_startup,
                t.storage_disk_not_connected
            ],
            expected,
            "language={language:?}"
        );
    }
}

#[cfg(all(target_os = "macos", feature = "app-store"))]
#[test]
fn test_app_store_feature_uses_macos_gpu_sampler() {
    assert_eq!(crate::gpu::IMPLEMENTATION, "macos_ioaccelerator");
}

#[cfg(target_os = "macos")]
#[test]
fn test_macos_process_lock_path_is_per_user() {
    assert_eq!(
        macos_process_lock_path(501),
        std::path::PathBuf::from("/tmp/dev.alexpedersen.better-resource-monitor.501.lock")
    );
}

#[cfg(target_os = "macos")]
#[test]
fn test_supported_macos_bundle_runtime() {
    const TRASH_PATH: &str = "/Users/xeliapedersen/.Trash/Better Resource Monitor.app/Contents/MacOS/better-resource-monitor";

    for (bundle_id, executable_path, has_receipt, is_direct, should_exit) in [
        // App Store build: receipt at the supported path.
        (
            Some(MACOS_BUNDLE_ID),
            MACOS_SUPPORTED_EXECUTABLE_PATH,
            true,
            false,
            false,
        ),
        // Stray copy in the Trash exits even with a receipt.
        (Some(MACOS_BUNDLE_ID), TRASH_PATH, true, false, true),
        // Non-direct build without a receipt exits (existing behavior).
        (
            Some(MACOS_BUNDLE_ID),
            MACOS_SUPPORTED_EXECUTABLE_PATH,
            false,
            false,
            true,
        ),
        // Direct-distribution build: no receipt, supported path — allowed.
        (
            Some(MACOS_BUNDLE_ID),
            MACOS_SUPPORTED_EXECUTABLE_PATH,
            false,
            true,
            false,
        ),
        // Direct-distribution build in the Trash still exits.
        (Some(MACOS_BUNDLE_ID), TRASH_PATH, false, true, true),
        // Other bundle ids are never the guard's business.
        (None, "/tmp/better-resource-monitor", false, false, false),
    ] {
        assert_eq!(
            should_exit_unsupported_macos_bundle(
                bundle_id,
                std::path::Path::new(executable_path),
                has_receipt,
                is_direct
            ),
            should_exit,
            "executable_path={executable_path} has_receipt={has_receipt} is_direct={is_direct}"
        );
    }
}

#[cfg(target_os = "macos")]
#[test]
fn test_macos_receipt_path_for_executable_uses_bundle_contents() {
    assert_eq!(
        macos_receipt_path_for_executable(std::path::Path::new(MACOS_SUPPORTED_EXECUTABLE_PATH))
            .as_deref(),
        Some(std::path::Path::new(
            "/Applications/Better Resource Monitor.app/Contents/_MASReceipt/receipt"
        ))
    );
}

#[test]
fn test_format_speed() {
    for (input, expected) in [
        (0.0, "0.0 KB"),
        (500.0, "0.5 KB"),
        (1_500.0, "1.5 KB"),
        (9_000.0, "9.0 KB"),
        (9_900.0, "9.9 KB"),
        (9_950.0, "9.9 KB"),
        (100_000.0, "100 KB"),
        (500_000.0, "500 KB"),
        (999_000.0, "999 KB"),
        (999_500.0, "1.0 MB"),
        (1_500_000.0, "1.5 MB"),
        (9_900_000.0, "9.9 MB"),
        (9_950_000.0, "9.9 MB"),
        (10_000_000.0, "10 MB"),
        (100_000_000.0, "100 MB"),
        (500_000_000.0, "500 MB"),
        (999_000_000.0, "999 MB"),
        (999_500_000.0, "1.0 GB"),
        (1_500_000_000.0, "1.5 GB"),
        (9_900_000_000.0, "9.9 GB"),
        (50_000_000_000.0, "50 GB"),
        (1e-10, "0.0 KB"),
        (0.001, "0.0 KB"),
        (0.5, "0.0 KB"),
        (999_000_000_000.0, "999 GB"),
        (999_500_000_000.0, "1.0 TB"),
        (1_500_000_000_000.0, "1.5 TB"),
        (999_500_000_000_000.0, "1.0 PB"),
        (1_500_000_000_000_000.0, "1.5 PB"),
        (1e18, "1000 PB"),
        (1e20, ">99999 PB"),
        (-100.0, "-0.1 KB"),
    ] {
        assert_eq!(format_speed(input), expected, "input={input}");
    }

    assert_eq!(format_storage_available(0), "0.0 KB");
    assert_eq!(format_storage_available(9_949_999_999), "9.9 GB");
    assert_eq!(format_storage_available(9_950_000_000), "9.9 GB");
    assert_eq!(format_storage_available(9_999_999_999), "<10 GB");
    assert_eq!(format_storage_available(10_000_000_000), "10.0 GB");
    assert_eq!(format_storage_available(19_500_000_000), "19.5 GB");
    assert_eq!(format_storage_available(u64::MAX), "18.4 EB");
}

#[test]
fn test_storage_display_update_uses_label_and_alert_state() {
    for (previous_label, previous_bytes, current_label, current_bytes, expected) in [
        (None, None, "19.5 GB", Some(19_500_000_000), true),
        (
            Some("19.5 GB"),
            Some(19_500_000_000),
            "19.5 GB",
            Some(19_500_000_000),
            false,
        ),
        (
            Some("10.0 GB"),
            Some(10_010_000_000),
            "10.0 GB",
            Some(9_990_000_000),
            true,
        ),
        (
            Some("9.9 GB"),
            Some(9_950_000_000),
            "9.9 GB",
            Some(9_940_000_000),
            false,
        ),
        (
            Some("19.5 GB"),
            Some(19_500_000_000),
            "20 GB",
            Some(20_000_000_000),
            true,
        ),
    ] {
        assert_eq!(
            storage_display_needs_update(
                previous_label,
                previous_bytes,
                current_label,
                current_bytes,
            ),
            expected,
            "previous_label={previous_label:?}, previous_bytes={previous_bytes:?}, current_label={current_label}, current_bytes={current_bytes:?}"
        );
    }
}

#[test]
fn test_render_svg_icon_valid() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" fill="currentColor"/></svg>"#;
    let result = tray_render::render_svg_icon(svg, 16, (255, 255, 255));

    assert!(!result.is_empty());
    assert_eq!(result.len(), 16 * 16 * 4);
}

#[test]
#[should_panic(expected = "Failed to parse SVG")]
fn test_render_svg_icon_invalid_panics() {
    tray_render::render_svg_icon("not valid svg", 16, (255, 255, 255));
}

#[test]
fn test_icon_buffer_reuse() {
    let font = load_system_font().expect("test font required");
    let mut renderer = tray_render::TrayRenderer::new();
    let mut buffer: Vec<u8> = Vec::with_capacity(4 * 800 * APP_SIZING.icon_height as usize);
    let initial_capacity = buffer.capacity();

    let (width1, height1, _) = renderer.render_tray_icon_into(
        &font,
        &mut buffer,
        &tray_render::RenderConfig {
            mem_percent: 60.0,
            down_str: "1.0 KB",
            up_str: "0.5 KB",
            show_net: true,
            show_alerts: false,
            ..base_render_config()
        },
    );
    assert!(width1 > 0);
    assert_eq!(height1, APP_SIZING.icon_height);
    assert!(!buffer.is_empty());

    let capacity_after_first = buffer.capacity();
    assert!(capacity_after_first >= initial_capacity);

    let (width2, height2, _) = renderer.render_tray_icon_into(
        &font,
        &mut buffer,
        &tray_render::RenderConfig {
            cpu_usage: 70.0,
            mem_percent: 80.0,
            down_str: "2.0 KB",
            up_str: "1.0 KB",
            show_net: true,
            show_alerts: false,
            ..base_render_config()
        },
    );
    assert!(width2 > 0);
    assert_eq!(height2, APP_SIZING.icon_height);
    assert!(buffer.capacity() >= capacity_after_first);
}

#[test]
fn test_alert_colors_all_segments() {
    let font = load_system_font().expect("test font required");
    let mut buffer: Vec<u8> = Vec::new();
    let mut renderer = tray_render::TrayRenderer::new();

    for (cpu_usage, show_alerts, expected_alert) in [
        (50.0, true, false),
        (80.0, true, false),
        (81.0, true, true),
        (81.0, false, false),
    ] {
        let (_, _, has_alert) = renderer.render_tray_icon_into(
            &font,
            &mut buffer,
            &tray_render::RenderConfig {
                cpu_usage,
                show_alerts,
                ..base_render_config()
            },
        );
        assert_eq!(
            has_alert, expected_alert,
            "cpu_usage={cpu_usage}, show_alerts={show_alerts}"
        );
    }

    for (storage_available_bytes, show_alerts, expected_alert) in [
        (Some(10_000_000_001), true, false),
        (Some(10_000_000_000), true, false),
        (Some(9_999_999_999), true, true),
        (Some(9_999_999_999), false, false),
        (None, true, false),
    ] {
        let (_, _, has_alert) = renderer.render_tray_icon_into(
            &font,
            &mut buffer,
            &tray_render::RenderConfig {
                cpu_usage: 0.0,
                mem_percent: 0.0,
                storage_available_bytes,
                show_cpu: false,
                show_mem: false,
                show_storage: true,
                show_alerts,
                ..base_render_config()
            },
        );
        assert_eq!(
            has_alert, expected_alert,
            "storage_available_bytes={storage_available_bytes:?}, show_alerts={show_alerts}"
        );
    }
}

#[test]
fn test_storage_alert_uses_available_space_instead_of_percentage() {
    let font = load_system_font().expect("test font required");
    let mut buffer = Vec::new();
    let mut renderer = tray_render::TrayRenderer::new();

    for (available, available_bytes, expected_alert) in [
        ("100 GB", Some(100_000_000_000), false),
        ("10.0 GB", Some(10_000_000_000), false),
        ("9.9 GB", Some(9_900_000_000), true),
        ("0.0 KB", None, false),
    ] {
        let (_, _, has_alert) = renderer.render_tray_icon_into(
            &font,
            &mut buffer,
            &tray_render::RenderConfig {
                cpu_usage: 0.0,
                mem_percent: 0.0,
                storage_available_str: available,
                storage_available_bytes: available_bytes,
                show_cpu: false,
                show_mem: false,
                show_storage: true,
                show_gpu: false,
                show_net: false,
                show_alerts: true,
                ..base_render_config()
            },
        );
        assert_eq!(
            has_alert, expected_alert,
            "available={available}, available_bytes={available_bytes:?}"
        );
    }
}

#[test]
fn test_sizing_scaled() {
    for (scale, expected) in [
        (2.0, (116, 256, 150, 10, 36, 44, 38.0)),
        (0.5, (29, 64, 38, 3, 9, 11, 9.5)),
        (0.333, (19, 43, 25, 2, 6, 7, 19.0 * 0.333)),
    ] {
        assert_sizing(tray_render::SIZING_LINUX.scaled(scale), expected);
    }
}

#[test]
#[should_panic(expected = "scale must be finite and between 0 and 4")]
fn test_sizing_scaled_panics_on_zero() {
    let _ = tray_render::SIZING_LINUX.scaled(0.0);
}

#[test]
#[should_panic(expected = "scale must be finite and between 0 and 4")]
fn test_sizing_scaled_panics_on_infinity() {
    let _ = tray_render::SIZING_LINUX.scaled(f32::INFINITY);
}

#[test]
#[should_panic(expected = "scale must be finite and between 0 and 4")]
fn test_sizing_scaled_panics_above_safe_limit() {
    let _ = tray_render::SIZING_LINUX.scaled(4.01);
}

#[test]
fn test_get_update_interval_ms() {
    for (value, expected) in [
        (None, UPDATE_INTERVAL_MS),
        (Some("1234"), 1234),
        (Some("abc"), UPDATE_INTERVAL_MS),
        (Some("0"), UPDATE_INTERVAL_MS),
    ] {
        with_update_interval_env(value, || {
            assert_eq!(get_update_interval_ms(), expected, "value={value:?}");
        });
    }
}

#[test]
fn test_render_with_all_segments_disabled() {
    let font = load_system_font().expect("test font required");
    let mut buffer = Vec::new();
    let mut renderer = tray_render::TrayRenderer::new();

    let (width, height, has_alert) = renderer.render_tray_icon_into(
        &font,
        &mut buffer,
        &tray_render::RenderConfig {
            gpu_usage: 50.0,
            show_cpu: false,
            show_mem: false,
            show_gpu: false,
            show_net: false,
            ..base_render_config()
        },
    );

    assert!(!has_alert);
    assert_render_size(&buffer, width, height, APP_SIZING.edge_padding * 2);
}

#[test]
fn test_render_with_long_network_strings() {
    let font = load_system_font().expect("test font required");
    let mut buffer = Vec::new();
    let mut renderer = tray_render::TrayRenderer::new();
    let long_down = "9".repeat(512);
    let long_up = "8".repeat(512);

    let (width, height, has_alert) = renderer.render_tray_icon_into(
        &font,
        &mut buffer,
        &tray_render::RenderConfig {
            cpu_usage: 0.0,
            mem_percent: 0.0,
            down_str: &long_down,
            up_str: &long_up,
            show_cpu: false,
            show_mem: false,
            show_net: true,
            ..base_render_config()
        },
    );

    let expected_width =
        APP_SIZING.edge_padding * 2 + (APP_SIZING.segment_width_net * 2) + APP_SIZING.segment_gap;

    assert!(!has_alert);
    assert_render_size(&buffer, width, height, expected_width);
}

#[test]
fn test_render_all_default_visible_metrics_width() {
    let font = load_system_font().expect("test font required");
    let mut buffer = Vec::new();
    let mut renderer = tray_render::TrayRenderer::new();

    let (storage_only_width, _, _) = renderer.render_tray_icon_into(
        &font,
        &mut buffer,
        &tray_render::RenderConfig {
            show_cpu: false,
            show_mem: false,
            show_storage: true,
            ..base_render_config()
        },
    );
    let rendered_storage_segment_width = storage_only_width - APP_SIZING.edge_padding * 2;

    let (width, height, has_alert) = renderer.render_tray_icon_into(
        &font,
        &mut buffer,
        &tray_render::RenderConfig {
            show_storage: true,
            show_gpu: true,
            show_net: true,
            ..base_render_config()
        },
    );

    let percent_segments = 3;
    let network_segments = 2;
    let segment_count = percent_segments + 1 + network_segments;
    let expected_width = APP_SIZING.edge_padding * 2
        + (APP_SIZING.segment_width * percent_segments)
        + rendered_storage_segment_width
        + (APP_SIZING.segment_width_net * network_segments)
        + (APP_SIZING.segment_gap * (segment_count - 1));

    assert!(!has_alert);
    assert_render_size(&buffer, width, height, expected_width);
}

#[test]
fn test_storage_visual_geometry_matches_network_and_fits_extremes() {
    let font = load_system_font().expect("test font required");
    let labels = [
        "0.0 KB", "999 KB", "1.0 MB", "999 MB", "1.0 GB", "<10 GB", "38.7 GB", "999 GB", "1.0 TB",
        "999 TB", "1.0 PB", "999 PB", "1.0 EB", "18.4 EB",
    ];

    for sizing in [tray_render::SIZING_MACOS, tray_render::SIZING_LINUX] {
        let mut renderer = tray_render::TrayRenderer::new();
        let mut buffer = Vec::new();

        let (network_width, network_height, _) = renderer.render_tray_icon_into(
            &font,
            &mut buffer,
            &tray_render::RenderConfig {
                sizing,
                show_cpu: false,
                show_mem: false,
                show_storage: false,
                show_gpu: false,
                show_net: true,
                show_alerts: false,
                down_str: "0.0 KB",
                up_str: "0.0 KB",
                ..base_render_config()
            },
        );
        let rendered_network_segment_width =
            (network_width - sizing.edge_padding * 2 - sizing.segment_gap) / 2;
        assert_eq!(network_height, sizing.icon_height);
        let network_visible_gap = visible_gap_between_icon_and_text(
            &buffer,
            network_width,
            network_height,
            sizing.edge_padding,
            rendered_network_segment_width,
            sizing.icon_height,
        );
        for label in labels {
            let (width, height, has_alert) = renderer.render_tray_icon_into(
                &font,
                &mut buffer,
                &tray_render::RenderConfig {
                    sizing,
                    storage_available_str: label,
                    show_cpu: false,
                    show_mem: false,
                    show_storage: true,
                    show_gpu: false,
                    show_net: false,
                    show_alerts: false,
                    ..base_render_config()
                },
            );
            assert!(!has_alert);
            assert_eq!(height, sizing.icon_height);
            assert_eq!(buffer.len(), (width * height * 4) as usize);

            let rendered_storage_segment_width = width - sizing.edge_padding * 2;
            let storage_available_width =
                (rendered_storage_segment_width - sizing.icon_height) as f32;
            let storage_scale = tray_render::fit_text_scale(
                &font,
                label,
                sizing.font_size,
                storage_available_width,
            );
            let storage_label_width: f32 = font
                .layout(label, storage_scale, rusttype::point(0.0, 0.0))
                .map(|glyph| glyph.unpositioned().h_metrics().advance_width)
                .sum();
            assert!(storage_label_width <= storage_available_width + 0.01);
            let storage_visible_gap = visible_gap_between_icon_and_text(
                &buffer,
                width,
                height,
                sizing.edge_padding,
                rendered_storage_segment_width,
                sizing.icon_height,
            );
            assert!(
                storage_visible_gap.abs_diff(network_visible_gap) <= 1,
                "storage label {label:?} visible gap {storage_visible_gap} must match network gap {network_visible_gap} within one raster pixel"
            );

            let ink_columns = buffer
                .as_chunks::<4>()
                .0
                .iter()
                .enumerate()
                .filter_map(|(index, pixel)| (pixel[3] > 0).then_some(index as u32 % width));
            let (leftmost, rightmost) =
                ink_columns.fold((width, 0), |(left, right), x| (left.min(x), right.max(x)));
            assert!(leftmost >= sizing.edge_padding);
            assert!(rightmost < width - sizing.edge_padding);
        }
    }
}

#[test]
fn test_render_buffer_matches_rgba_dimensions() {
    // The monitor loop skips a frame when the rendered buffer is not exactly
    // 4 bytes per pixel; this pins the renderer invariant that check relies on.
    let font = load_system_font().expect("test font required");
    let mut buffer = Vec::new();
    let mut renderer = tray_render::TrayRenderer::new();

    for config in [
        base_render_config(),
        tray_render::RenderConfig {
            show_storage: true,
            show_gpu: true,
            show_net: true,
            ..base_render_config()
        },
    ] {
        let (width, height, _) = renderer.render_tray_icon_into(&font, &mut buffer, &config);
        assert_eq!(
            buffer.len(),
            (4 * width * height) as usize,
            "render buffer must be 4 bytes per pixel for {width}x{height}"
        );
    }
}

#[test]
fn test_metric_icon_order() {
    assert_eq!(
        tray_render::metric_icon_order_for_tests(),
        [
            tray_render::IconType::Memory,
            tray_render::IconType::Cpu,
            tray_render::IconType::Gpu,
            tray_render::IconType::Storage,
        ]
    );
}

#[test]
fn test_detect_language_does_not_panic() {
    let lang = i18n::detect_language();
    let t = lang.translations();
    assert!(!t.quit.is_empty());
}

#[test]
fn test_detect_language_from_locales_recognizes_supported_tags() {
    for (locales, expected) in [
        (&["es-ES"][..], i18n::Language::Spanish),
        (&["ES_mx"][..], i18n::Language::Spanish),
        (&["pt-BR"][..], i18n::Language::Portuguese),
        (&["zh-Hans"][..], i18n::Language::Chinese),
        (&["pl-PL"][..], i18n::Language::Polish),
        (&["pl"][..], i18n::Language::Polish),
        (&["PL_pl"][..], i18n::Language::Polish),
        (&["en-US"][..], i18n::Language::English),
    ] {
        assert_eq!(
            i18n::detect_language_from_locales(locales.iter().copied()),
            expected,
            "locales={locales:?}"
        );
    }
}

#[test]
fn test_detect_language_from_locales_uses_preference_order() {
    for (locales, expected) in [
        (&["fr-FR", "es-ES"][..], i18n::Language::Spanish),
        (&["en-US", "es-ES"][..], i18n::Language::English),
        (&["de-DE", "pl-PL", "en-US"][..], i18n::Language::Polish),
        (&["fr-FR", "de-DE", ""][..], i18n::Language::English),
    ] {
        assert_eq!(
            i18n::detect_language_from_locales(locales.iter().copied()),
            expected,
            "locales={locales:?}"
        );
    }
}

#[test]
fn test_all_languages_have_translations() {
    let languages = [
        i18n::Language::English,
        i18n::Language::Spanish,
        i18n::Language::Portuguese,
        i18n::Language::Chinese,
        i18n::Language::Polish,
    ];
    for lang in languages {
        let t = lang.translations();
        assert!(!t.start_at_login.is_empty());
        assert!(!t.show_memory.is_empty());
        assert!(!t.show_cpu.is_empty());
        assert!(!t.show_storage.is_empty());
        assert!(!t.storage_disk.is_empty());
        assert!(!t.storage_disk_startup.is_empty());
        assert!(!t.storage_disk_not_connected.is_empty());
        assert!(!t.show_gpu.is_empty());
        assert!(!t.show_network.is_empty());
        assert!(!t.show_alert_colors.is_empty());
        assert!(!t.quit.is_empty());
        assert!(!t.system_monitor.is_empty());
        assert!(!t.thermal_status.is_empty());
        assert!(!t.thermal_unavailable.is_empty());
        assert!(!t.thermal_nominal.is_empty());
        assert!(!t.thermal_fair.is_empty());
        assert!(!t.thermal_serious.is_empty());
        assert!(!t.thermal_critical.is_empty());
    }
}

#[test]
fn test_menu_translations_match_supported_locales() {
    let expected = [
        (
            i18n::Language::English,
            [
                "Start at login",
                "Show memory",
                "Show CPU",
                "Show storage",
                "Show GPU",
                "Show network",
                "Show warning colors",
                "Quit",
                "System monitor",
            ],
        ),
        (
            i18n::Language::Spanish,
            [
                "Abrir al iniciar sesión",
                "Mostrar memoria",
                "Mostrar CPU",
                "Mostrar almacenamiento",
                "Mostrar GPU",
                "Mostrar uso de red",
                "Mostrar colores de advertencia",
                "Salir",
                "Monitor del sistema",
            ],
        ),
        (
            i18n::Language::Portuguese,
            [
                "Iniciar com o sistema",
                "Mostrar memória",
                "Mostrar CPU",
                "Mostrar armazenamento",
                "Mostrar GPU",
                "Mostrar rede",
                "Mostrar cores de aviso",
                "Sair",
                "Monitor do sistema",
            ],
        ),
        (
            i18n::Language::Chinese,
            [
                "登录时启动",
                "显示内存",
                "显示 CPU",
                "显示存储空间",
                "显示 GPU",
                "显示网络",
                "显示警告颜色",
                "退出",
                "系统监控器",
            ],
        ),
        (
            i18n::Language::Polish,
            [
                "Uruchamiaj po zalogowaniu",
                "Pokaż pamięć",
                "Pokaż CPU",
                "Pokaż dysk",
                "Pokaż GPU",
                "Pokaż sieć",
                "Pokaż kolory ostrzeżeń",
                "Zakończ",
                "Monitor systemu",
            ],
        ),
    ];

    for (language, expected) in expected {
        let translations = language.translations();
        let actual = [
            translations.start_at_login,
            translations.show_memory,
            translations.show_cpu,
            translations.show_storage,
            translations.show_gpu,
            translations.show_network,
            translations.show_alert_colors,
            translations.quit,
            translations.system_monitor,
        ];

        assert_eq!(actual, expected, "language={language:?}");
    }
}

#[test]
fn test_thermal_translations_match_supported_locales() {
    let expected = [
        (
            i18n::Language::English,
            [
                "Thermal status",
                "unavailable",
                "nominal",
                "fair",
                "serious",
                "critical",
            ],
        ),
        (
            i18n::Language::Spanish,
            [
                "Estado térmico",
                "no disponible",
                "normal",
                "elevado",
                "alto",
                "crítico",
            ],
        ),
        (
            i18n::Language::Portuguese,
            [
                "Estado térmico",
                "indisponível",
                "normal",
                "elevado",
                "alto",
                "crítico",
            ],
        ),
        (
            i18n::Language::Chinese,
            ["温度状态", "不可用", "正常", "偏高", "高", "危急"],
        ),
        (
            i18n::Language::Polish,
            [
                "Stan termiczny",
                "niedostępny",
                "normalny",
                "podwyższony",
                "wysoki",
                "krytyczny",
            ],
        ),
    ];

    for (language, expected) in expected {
        let translations = language.translations();
        let actual = [
            translations.thermal_status,
            translations.thermal_unavailable,
            translations.thermal_nominal,
            translations.thermal_fair,
            translations.thermal_serious,
            translations.thermal_critical,
        ];

        assert_eq!(actual, expected, "language={language:?}");
    }
}

#[test]
fn test_english_defaults() {
    let t = i18n::Language::English.translations();
    assert_eq!(t.quit, "Quit");
    assert_eq!(t.start_at_login, "Start at login");
    assert_eq!(t.show_memory, "Show memory");
    assert_eq!(t.show_cpu, "Show CPU");
    assert_eq!(t.show_storage, "Show storage");
    assert_eq!(t.show_gpu, "Show GPU");
    assert_eq!(t.show_network, "Show network");
    assert_eq!(t.show_alert_colors, "Show warning colors");
    assert_eq!(t.system_monitor, "System monitor");
    assert_eq!(t.thermal_status, "Thermal status");
    assert_eq!(t.thermal_unavailable, "unavailable");
}

#[cfg(target_os = "macos")]
#[test]
fn test_thermal_menu_status_text_is_localized_for_every_locale() {
    let expected = [
        (
            i18n::Language::English,
            [
                "Thermal status: nominal",
                "Thermal status: fair",
                "Thermal status: serious",
                "Thermal status: critical",
                "Thermal status: unavailable",
            ],
        ),
        (
            i18n::Language::Spanish,
            [
                "Estado térmico: normal",
                "Estado térmico: elevado",
                "Estado térmico: alto",
                "Estado térmico: crítico",
                "Estado térmico: no disponible",
            ],
        ),
        (
            i18n::Language::Portuguese,
            [
                "Estado térmico: normal",
                "Estado térmico: elevado",
                "Estado térmico: alto",
                "Estado térmico: crítico",
                "Estado térmico: indisponível",
            ],
        ),
        (
            i18n::Language::Chinese,
            [
                "温度状态: 正常",
                "温度状态: 偏高",
                "温度状态: 高",
                "温度状态: 危急",
                "温度状态: 不可用",
            ],
        ),
        (
            i18n::Language::Polish,
            [
                "Stan termiczny: normalny",
                "Stan termiczny: podwyższony",
                "Stan termiczny: wysoki",
                "Stan termiczny: krytyczny",
                "Stan termiczny: niedostępny",
            ],
        ),
    ];

    for (language, expected) in expected {
        let translations = language.translations();
        let actual = [
            thermal_status_text(translations, thermal::ThermalStatus::Nominal),
            thermal_status_text(translations, thermal::ThermalStatus::Fair),
            thermal_status_text(translations, thermal::ThermalStatus::Serious),
            thermal_status_text(translations, thermal::ThermalStatus::Critical),
            thermal_status_text(translations, thermal::ThermalStatus::Unavailable),
        ];

        assert_eq!(actual, expected, "language={language:?}");
    }
}

#[test]
fn network_speed_ignores_loopback_tunnels_and_virtual_bridges() {
    for name in [
        "lo",
        "lo0",
        "utun0",
        "utun12",
        "tun0",
        "tap1",
        "wg0",
        "ipsec0",
        "gif0",
        "stf0",
        "tailscale0",
        "docker0",
        "docker_gwbridge",
        "podman0",
        "podman12",
        "br-3f2a9c1d4e5b",
        "veth12ab34c",
        "virbr0",
        "vmnet8",
        "vmenet0",
        "vboxnet0",
        "bridge100",
        "bridge101",
        "Loopback Pseudo-Interface 1",
    ] {
        assert!(
            !counts_toward_network_speed(name),
            "{name} should be excluded"
        );
    }
}

#[test]
fn network_speed_keeps_physical_and_peer_to_peer_links() {
    for name in [
        "en0",
        "en7",
        "eth0",
        "enp3s0",
        "wlan0",
        "wlp2s0",
        "bridge0",
        "awdl0",
        "llw0",
        "ap1",
        "anpi0",
        "ppp0",
        "Wi-Fi",
        "Ethernet",
        "lowpan0",
        "tunnel-like",
        "wgx",
        "podman-like",
        "docker_gwbridge-uplink",
        "vmenet-uplink",
    ] {
        assert!(
            counts_toward_network_speed(name),
            "{name} should be counted"
        );
    }
}

#[test]
fn network_totals_skip_excluded_interfaces() {
    let interfaces = [
        ("lo0", 19_000_000_000, 19_000_000_000),
        ("en0", 5_000, 700),
        ("utun4", 4_000, 600),
        ("en5", 250, 50),
        ("bridge100", 1_000, 1_000),
        ("vmenet0", 2_000, 2_000),
        ("docker_gwbridge", 3_000, 3_000),
        ("podman0", 4_000, 4_000),
    ];
    assert_eq!(sum_interface_totals(interfaces), (5_250, 750));
}

#[test]
fn network_totals_saturate_instead_of_overflowing() {
    let interfaces = [("en0", u64::MAX, 1), ("en1", 10, u64::MAX)];
    assert_eq!(sum_interface_totals(interfaces), (u64::MAX, u64::MAX));
}

#[test]
fn network_totals_are_zero_with_only_loopback() {
    assert_eq!(sum_interface_totals([("lo0", 123, 456)]), (0, 0));
}
