<p align="center">
  <img src="https://better-resource-monitor.alexpedersen.dev/better-resource-monitor.png" alt="Better Resource Monitor" width="830" height="43">
</p>

<h1 align="center">Better Resource Monitor</h1>

<p align="center">
  <strong>A menu bar/tray system monitor for macOS and Linux.</strong>
</p>

<p align="center">
  <a href="https://apps.apple.com/app/better-resource-monitor/id6758237306"><img src="https://better-resource-monitor.alexpedersen.dev/badges/appstore.webp" alt="Download on the Mac App Store" width="268" height="65"></a>
  <a href="https://github.com/alexx855/better-resource-monitor/releases"><img src="https://better-resource-monitor.alexpedersen.dev/badges/macos.webp" alt="Download macOS on GitHub Releases" width="268" height="65"></a>
  <a href="https://github.com/alexx855/better-resource-monitor/releases"><img src="https://better-resource-monitor.alexpedersen.dev/badges/ubuntu.webp" alt="Download Ubuntu on GitHub Releases" width="268" height="65"></a>
</p>

<p align="center">
  <a href="#features">Features</a> <span>•</span>
  <a href="#why">Why</a> <span>•</span>
  <a href="#comparison">Comparison</a> <span>•</span>
  <a href="#installation">Installation</a>
</p>



## Features

* **Cross-Platform** - macOS and Linux (with optional NVIDIA GPU support).
* **Lightweight** - Written in Rust. < 0.1% CPU, ~15MB RAM (measured on Apple M1).
* **GPU Monitoring** - Apple Silicon residency via IOReport. NVIDIA utilization via NVML.
* **Theme Aware** - Auto-detects light/dark mode.
* **Configurable** - Toggle CPU, GPU, Memory, or Network stats via the right-click menu (GPU shown when hardware is available).
* **Private** - 100% local. No analytics, network requests, or telemetry.

## Why?

I wanted <a href="https://github.com/0ry0n/Resource_Monitor" target="_blank" rel="noopener noreferrer"><strong>GNOME Resource Monitor</strong></a>'s simplicity on macOS. Paid apps charge $15+ for basic stats. Free alternatives lack GPU support. This uses platform-native GPU APIs (IOReport on macOS, NVML on Linux).

### How it works

| Component | macOS | Linux |
| :--- | :--- | :--- |
| **CPU/Memory/Network** | `sysinfo` crate | `sysinfo` crate |
| **GPU Metrics** | IOReport FFI (private APIs) | NVML via `nvml-wrapper` |
| **Theme Detection** | Template icon (native macOS adaptation) | gsettings |

On macOS, it calculates **active residency** instead of just "utilization" - giving true insight into GPU workload. Runs without `sudo` and looks like a system component.

### macOS Version Differences

<table>
  <thead>
    <tr>
      <th width="34%">Feature</th>
      <th width="33%">App Store</th>
      <th width="33%">GitHub Download</th>
    </tr>
  </thead>
  <tbody>
    <tr><td><strong>CPU Monitoring</strong></td><td align="center">Yes</td><td align="center">Yes</td></tr>
    <tr><td><strong>Memory Monitoring</strong></td><td align="center">Yes</td><td align="center">Yes</td></tr>
    <tr><td><strong>Network Monitoring</strong></td><td align="center">Yes</td><td align="center">Yes</td></tr>
    <tr><td><strong>GPU Monitoring</strong></td><td align="center">No</td><td align="center">Yes</td></tr>
    <tr><td><strong>Automatic Updates</strong></td><td align="center">Yes</td><td align="center">No</td></tr>
  </tbody>
</table>

> **Why no GPU in App Store?** GPU monitoring requires Apple's private `IOReport` framework. Apple rejects apps using private APIs during App Store review. The GitHub download version is notarized but not sandboxed, so it has full GPU access.
>
> [Download from GitHub Releases with GPU support](https://github.com/alexx855/better-resource-monitor/releases)

## Comparison

<table>
  <thead>
    <tr>
      <th width="25%"></th>
      <th width="25%">Better Resource Monitor</th>
      <th width="25%">Stats</th>
      <th width="25%">iStat Menus</th>
    </tr>
  </thead>
  <tbody>
    <tr><td><strong>Price</strong></td><td align="center">Free</td><td align="center">Free</td><td align="center">$14.99</td></tr>
    <tr><td><strong>License</strong></td><td align="center">MIT</td><td align="center">MIT</td><td align="center">Proprietary</td></tr>
    <tr><td><strong>Memory</strong></td><td align="center">~15 MB</td><td align="center">~45 MB</td><td align="center">~60 MB</td></tr>
    <tr><td><strong>CPU</strong></td><td align="center">&lt; 0.1%</td><td align="center">&lt; 0.5%</td><td align="center">&lt; 0.5%</td></tr>
    <tr><td><strong>App Size</strong></td><td align="center">&lt; 7 MB (.app)</td><td align="center">~30 MB</td><td align="center">~40 MB</td></tr>
    <tr><td><strong>GPU</strong></td><td align="center">IOReport / NVML</td><td align="center">IOReport</td><td align="center">Proprietary</td></tr>
    <tr><td><strong>Linux</strong></td><td align="center">Yes</td><td align="center">No</td><td align="center">No</td></tr>
  </tbody>
</table>

> Third-party figures are approximate and may vary by version and system configuration.

## Installation

### macOS

**App Store (Recommended):**
Download from the [Mac App Store](https://apps.apple.com/app/better-resource-monitor/id6758237306). Automatic updates, but no GPU monitoring.

**GitHub Download (GPU Support):**
Download the latest `.dmg` from <a href="https://github.com/alexx855/better-resource-monitor/releases" target="_blank" rel="noopener noreferrer">GitHub Releases</a>. Includes GPU monitoring for Apple Silicon.

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
