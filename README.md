<p align="center">
  <img src="/better-resource-monitor.png" alt="Better Resource Monitor" width="830" height="43">
</p>

<h1 align="center">Better Resource Monitor</h1>

<p align="center">
  <strong>A menu bar/tray system monitor for macOS and Linux.</strong>
</p>

<p align="center">
  <a href="https://apps.apple.com/app/better-resource-monitor/id6758237306"><img src="https://developer.apple.com/app-store/marketing/guidelines/images/badge-download-on-the-mac-app-store.svg" alt="Download on the Mac App Store" height="40"></a>
</p>

<p align="center">
  <a href="https://github.com/alexx855/better-resource-monitor/releases/download/v1.0.0/Better.Resource.Monitor_1.0.0_aarch64.dmg"><img src="https://img.shields.io/badge/macOS-Direct_Download-D14715?logo=apple&logoColor=white&style=for-the-badge" alt="macOS Direct Download"></a>
  &nbsp;&nbsp;
  <a href="https://github.com/alexx855/better-resource-monitor/releases/download/v1.0.0/better-resource-monitor_1.0.0_amd64.deb"><img src="https://img.shields.io/badge/Ubuntu-Direct_Download-D14715?logo=ubuntu&logoColor=white&style=for-the-badge" alt="Ubuntu Direct Download"></a>
</p>

<p align="center">
  <a href="#features">Features</a> <span>•</span>
  <a href="#why">Why</a> <span>•</span>
  <a href="#comparison">Comparison</a> <span>•</span>
  <a href="#installation">Installation</a>
</p>



## Features

* **Cross-Platform** - macOS (Apple Silicon) and Linux (Ubuntu with NVIDIA GPUs).
* **Lightweight** - Written in Rust. < 0.1% CPU, ~15MB RAM.
* **GPU Monitoring** - Apple Silicon residency via IOReport. NVIDIA utilization via NVML.
* **Theme Aware** - Auto-detects light/dark mode.
* **Configurable** - Toggle CPU, GPU, Memory, or Network stats with a click.
* **Private** - 100% local. No analytics, network requests, or telemetry.

## Why?

I wanted <a href="https://github.com/0ry0n/Resource_Monitor" target="_blank" rel="noopener noreferrer"><strong>GNOME Resource Monitor</strong></a>'s simplicity on macOS. Paid apps charge $15+ for basic stats. Free alternatives lack GPU support. This uses platform-native GPU APIs (IOReport on macOS, NVML on Linux).

### How it works

| Component | macOS | Linux |
| :--- | :--- | :--- |
| **CPU/Memory/Network** | `sysinfo` crate | `sysinfo` crate |
| **GPU Metrics** | IOReport FFI (private APIs) | NVML via `nvml-wrapper` |
| **Theme Detection** | Menu bar color sampling | gsettings |

On macOS, it calculates **active residency** instead of just "utilization" - giving true insight into GPU workload. Runs without `sudo` and looks like a system component.

### macOS Version Differences

| Feature | App Store | Direct Download |
| :--- | :---: | :---: |
| **CPU Monitoring** | ✅ | ✅ |
| **Memory Monitoring** | ✅ | ✅ |
| **Network Monitoring** | ✅ | ✅ |
| **GPU Monitoring** | ❌ | ✅ |
| **Automatic Updates** | ✅ | ❌ |

> **Why no GPU in App Store?** GPU monitoring requires Apple's private `IOReport` framework. Apple rejects apps using private APIs during App Store review. The direct download version is notarized but not sandboxed, so it has full GPU access.
>
> [Download direct version with GPU support](https://github.com/alexx855/better-resource-monitor/releases)

## Comparison

| | Better Resource Monitor | Stats | iStat Menus |
| :--- | :---: | :---: | :---: |
| **Price** | Free | Free | $14.99 |
| **License** | MIT | MIT | Proprietary |
| **Memory** | ~15 MB | ~45 MB | ~60 MB |
| **CPU** | < 0.1% | < 0.5% | < 0.5% |
| **App Size** | < 7 MB | ~30 MB | ~40 MB |
| **GPU** | IOReport / NVML | IOReport | Proprietary |
| **Linux** | Yes | No | No |

## Installation

### macOS

**App Store (Recommended):**
Download from the [Mac App Store](https://apps.apple.com/app/better-resource-monitor/id6758237306). Automatic updates, but no GPU monitoring.

**Direct Download (GPU Support):**
Download the latest `.dmg` from <a href="https://github.com/alexx855/better-resource-monitor/releases" target="_blank" rel="noopener noreferrer">Releases</a>. Includes GPU monitoring for Apple Silicon.

### Ubuntu

**Download** the latest `.deb` from <a href="https://github.com/alexx855/better-resource-monitor/releases" target="_blank" rel="noopener noreferrer">Releases</a> and install:

```bash
sudo dpkg -i better-resource-monitor_*.deb
```

**GPU Monitoring (Optional):** Requires proprietary NVIDIA drivers. Without them, GPU monitoring is hidden.

### Build from Source

#### Prerequisites

Install [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your platform, plus [pnpm](https://pnpm.io/).

#### Build

```bash
git clone https://github.com/alexx855/better-resource-monitor.git
cd better-resource-monitor
pnpm install
pnpm tauri build
```

### Development

```bash
# Run in development mode with hot reload
pnpm tauri dev

# Run tests
cd src-tauri && cargo test

# Run tests with coverage (requires cargo-llvm-cov)
cargo install cargo-llvm-cov
cd src-tauri && cargo llvm-cov --lib --html --output-dir coverage/
```

## Credits

- [Astro](https://astro.build/) - Framework for the website
- [Tauri](https://tauri.app/) - Framework for building the app
- [Phosphor Icons](https://github.com/phosphor-icons) - Icon set used in the tray
- [Alex Pedersen](https://alexpedersen.dev/) - Creator and maintainer
