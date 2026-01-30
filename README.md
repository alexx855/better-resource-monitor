<p align="center">
  <img src="https://raw.githubusercontent.com/alexx855/silicon-monitor/main/www/public/better-resource-monitor.png" alt="Better Resource Monitor" width="830" height="43">
</p>

<h1 align="center">Better Resource Monitor</h1>

<p align="center">
  <strong>A menu bar/tray system monitor for macOS and Linux.</strong>
</p>

<p align="center">
  <a href="https://github.com/alexx855/better-resource-monitor/releases/download/v1.0.0/Better.Resource.Monitor_1.0.0_aarch64.dmg" aria-label="Download macOS .dmg"><img src="https://img.shields.io/badge/macOS-Download-E95420?logo=apple&logoColor=white&style=for-the-badge" alt="Download macOS .dmg"></a>
  <a href="https://github.com/alexx855/better-resource-monitor/releases/download/v1.0.0/better-resource-monitor_1.0.0_amd64.deb" aria-label="Download Ubuntu .deb"><img src="https://img.shields.io/badge/Ubuntu-Download-E95420?logo=ubuntu&logoColor=white&style=for-the-badge" alt="Download Ubuntu .deb"></a>

</p>

<p align="center">
  <a href="#features">Features</a> <span style="color: #e95420; font-size: 1.2em;">•</span>
  <a href="#why">Why</a> <span style="color: #e95420; font-size: 1.2em;">•</span>
  <a href="#comparison">Comparison</a> <span style="color: #e95420; font-size: 1.2em;">•</span>
  <a href="#installation">Installation</a>
</p>



## Features

* **Cross-Platform** — Same app on macOS and Linux. Apple Silicon Macs and Ubuntu with NVIDIA GPUs.
* **Zero Impact** — Written in Rust. < 0.1% CPU, ~15MB RAM. Lighter than a browser tab.
* **GPU Monitoring** — Apple Silicon residency via IOReport. NVIDIA utilization via NVML. No hacks.
* **Theme Aware** — Auto-detects light/dark mode. Blends seamlessly with your menu bar.
* **Configurable** — Toggle CPU, GPU, Memory, or Network stats with a click.
* **Privacy First** — 100% local. No analytics. No network requests. No telemetry.

## Why?

I built this out of necessity. I wanted the simplicity of **GNOME Resource Monitor**—just a clean line of text in the menu bar showing exactly what my system was doing. But I needed it on macOS too.

Everything else fell short:

* **Paid apps** — Overkill. $15+ with subscription upgrades for basic stats.
* **Free alternatives** — Missing GPU support or looking out of place.

**Better Resource Monitor** fills the gap. One app, two platforms, zero compromise. It uses **platform APIs** for GPU monitoring—IOReport on macOS for Apple Silicon, NVML on Linux for NVIDIA GPUs—with no overhead.

### How it works

| Component | macOS | Linux |
| :--- | :--- | :--- |
| **CPU/Memory/Network** | `sysinfo` crate | `sysinfo` crate |
| **GPU Metrics** | IOReport FFI (private APIs) | NVML via `nvml-wrapper` |
| **Theme Detection** | Menu bar color sampling | gsettings |

On macOS, it calculates **active residency** instead of just "utilization"—giving true insight into GPU workload. Runs without `sudo` and looks like a system component.

## Comparison

| Feature | Better Resource Monitor | Stats | iStat Menus |
| :--- | :---: | :---: | :---: |
| **Price** | ✅ Free | ✅ Free | ❌ $14.99 |
| **Open Source** | ✅ MIT | ✅ MIT | ❌ Proprietary |
| **Memory Usage** | ✅ ~15 MB | ⚠️ ~45 MB | ⚠️ ~60 MB |
| **CPU Usage** | ✅ < 0.1% | ✅ < 0.5% | ✅ < 0.5% |
| **App Size** | ✅ < 7 MB | ⚠️ ~30 MB | ⚠️ ~40 MB |
| **GPU Monitoring** | ✅ System | ✅ IOReport | ✅ Proprietary |
| **macOS** | ✅ Apple Silicon | ✅ | ✅ |
| **Linux** | ✅ Ubuntu/Debian | ❌ | ❌ |
| **Theme Aware** | ✅ Auto-detect | ✅ | ✅ |

## Installation

### macOS

**Download** the latest `.dmg` from [Releases](../../releases).

### Ubuntu

**Download** the latest `.deb` from [Releases](../../releases) and install:

```bash
sudo dpkg -i better-resource-monitor_*.deb
```

**GPU Monitoring (Optional):** To enable NVIDIA GPU monitoring, ensure you have the proprietary NVIDIA drivers installed:

```bash
# Check if NVIDIA driver is installed
nvidia-smi

# If not installed, install via:
sudo ubuntu-drivers autoinstall
```

If NVIDIA drivers are not available, the app will still work—GPU monitoring will simply be hidden.

### Build from Source

#### Prerequisites

**macOS:**
- Xcode Command Line Tools
- Rust toolchain
- pnpm

**Ubuntu:**
```bash
# Install build dependencies
sudo apt update
sudo apt install -y \
    build-essential \
    curl \
    wget \
    file \
    libwebkit2gtk-4.1-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    patchelf

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install pnpm
npm install -g pnpm
```

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

## Linux Troubleshooting

**Desktop Environment Support:**
- **GNOME Shell**: ⚠️ Requires `ubuntu-appindicators` extension (doesn't display tray icons natively)
- **KDE Plasma**: ✅ SNI support (works out of the box)
- **XFCE**: ✅ System tray support
- **Other DEs**: Depends on SNI/tray support

### GPU Monitoring on Linux

NVIDIA GPU monitoring requires the proprietary NVIDIA drivers. To verify:

```bash
nvidia-smi
```

If the command is not found, install NVIDIA drivers:

```bash
sudo ubuntu-drivers autoinstall
```

The app will work without GPU monitoring—it will simply be hidden from the display.



## Credits

- [Phosphor Icons](https://github.com/phosphor-icons) — Icon set used in the tray




