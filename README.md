<p align="center">
  <img src="silicon-monitor.jpeg" alt="Silicon Monitor" width="830" height="372">
</p>

<p align="center">
  <strong>A native menu bar/tray system monitor for macOS and Linux.</strong>
</p>

<p align="center">
  <a href="https://www.apple.com/macos/"><img src="https://img.shields.io/badge/macOS-15.0%2B_(Sequoia)-0078d7?logo=apple&logoColor=white&style=flat-square" alt="macOS 15.0+ (Sequoia)"></a>
  <img src="https://img.shields.io/badge/Apple_Silicon-M--Series-orange?logo=apple&logoColor=white&style=flat-square" alt="Apple Silicon M-Series">
  <a href="https://ubuntu.com/"><img src="https://img.shields.io/badge/Ubuntu-25.04+-E95420?logo=ubuntu&logoColor=white&style=flat-square" alt="Ubuntu 25.04+"></a>
  <img src="https://img.shields.io/badge/NVIDIA-GPU_Support-76B900?logo=nvidia&logoColor=white&style=flat-square" alt="NVIDIA GPU Support">
  <a href="https://github.com/alexx855/silicon-monitor/releases/latest"><img src="https://img.shields.io/badge/Download-Releases-000000?logo=github&logoColor=white&style=flat-square" alt="Download Releases"></a>
  <img src="https://img.shields.io/badge/License-MIT-blue?style=flat-square" alt="MIT License">
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#why">Why</a> •
  <a href="#comparison">Comparison</a> •
  <a href="#installation">Installation</a>
</p>

---

## Features

* **Zero Impact** — Written in Rust. Minimal CPU (< 0.1%) & Memory (~15MB) usage.
* **Native Design** — Adapts to menu bar appearance at startup. macOS menu bar adapts to wallpaper color, not system theme. Restart app after wallpaper change.
* **Cross-Platform** — Works on macOS (Apple Silicon) and Ubuntu Linux (NVIDIA GPUs).
* **GPU Monitoring** — Apple Silicon GPU residency on macOS, NVIDIA utilization on Linux.
* **Fully Configurable** — Toggle CPU, GPU, Memory, or Network stats instantly.
* **Battery Efficient** — Smart polling that won't drain your laptop.
* **Privacy Focused** — 100% local. No analytics. No network requests.

## Why?

I built this out of necessity. After switching from Ubuntu to macOS, I missed the simplicity of the **GNOME Resource Monitor** extension—just a clean line of text showing me exactly what my system was doing.

I tried everything else:

* **Electron apps** were too heavy (~150MB+ RAM for a text label? That's the "Electron Tax").
* **Paid apps** were overkill and expensive ($15+ with paid upgrades).
* **Free alternatives** often lacked proper **GPU** support or didn't match the OS aesthetics.

**Silicon Monitor** is the answer. It uses **native platform APIs** to monitor your GPU with zero overhead—IOReport on macOS for Apple Silicon, NVML on Linux for NVIDIA GPUs.

### How it works

* **Hybrid Architecture**: Uses `sysinfo` crate for standard metrics (CPU, Memory, Network).
* **macOS GPU Monitoring**: Uses `IOReport` FFI to access private macOS APIs for accurate GPU residency.
* **Linux GPU Monitoring**: Uses `nvml-wrapper` for NVIDIA GPU utilization via NVML.
* **Theme Aware**: Samples menu bar color at startup for optimal performance. On macOS, menu bar adapts to wallpaper (not system theme), so restart app after wallpaper change. On Linux, uses gsettings.

It calculates **active residency** on macOS instead of just "utilization," giving you true insight into your GPU's workload. On Linux, it provides standard NVIDIA utilization metrics. It runs without `sudo`, and looks exactly like a native system component.

## Comparison

| Feature | Silicon Monitor | Stats | iStat Menus | Electron Apps |
| :--- | :--- | :--- | :--- | :--- |
| **Price** | ✅ Free | ✅ Free | ❌ $14.99 | ❌ Varies |
| **Open Source** | ✅ MIT | ✅ MIT | ❌ Proprietary | ⚠️ Varies |
| **Idle Memory** | ✅ ~15 MB | ⚠️ ~45 MB | ⚠️ ~60 MB | ❌ 150-400 MB |
| **CPU (Idle)** | ✅ < 0.1% | ✅ < 0.5% | ✅ < 0.5% | ❌ 1.0 - 3.0% |
| **App Size** | ✅ < 7 MB | ⚠️ ~30 MB | ⚠️ ~40 MB | ❌ > 100 MB |
| **GPU Metrics** | ✅ Native | ✅ IOReport | ✅ Proprietary | ❌ Limited |
| **Cross-Platform** | ✅ macOS + Linux | ❌ macOS only | ❌ macOS only | ⚠️ Varies |

## Installation

### macOS

**Download** the latest `.dmg` from [Releases](../../releases).

### Ubuntu

**Download** the latest `.deb` from [Releases](../../releases) and install:

```bash
sudo dpkg -i silicon-monitor_*.deb
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
git clone https://github.com/alexx855/silicon-monitor.git
cd silicon-monitor
pnpm install
pnpm tauri build
```

## Linux Troubleshooting

**Desktop Environment Support:**
- **GNOME Shell**: ⚠️ Requires `ubuntu-appindicators` extension (doesn't display tray icons natively)
- **KDE Plasma**: ✅ Native SNI support (works out of the box)
- **XFCE**: ✅ Native system tray support
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

---

## Credits

- [Tauri](https://github.com/tauri-apps/tauri) — Cross-platform app framework
- [Phosphor Icons](https://github.com/phosphor-icons) — Icon set used in the tray

---

<p align="center">
  Made with ❤️ by <a href="https://alexpedersen.dev/">Alex Pedersen</a>
</p>
