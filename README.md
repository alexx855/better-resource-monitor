<p align="center">
  <img src="https://better-resource-monitor.alexpedersen.dev/better-resource-monitor.png" alt="Better Resource Monitor" width="830" height="86">
</p>


<h1 align="center">Better Resource Monitor</h1>

<p align="center">
  <strong>See what your Mac is doing from the menu bar.</strong>
</p>

<!-- README-LANG-START -->

<p align="center">
  English •
  <a href="README.es.md">Español</a> •
  <a href="README.pt-br.md">Português (Brasil)</a> •
  <a href="README.zh-cn.md">简体中文</a>
</p>

<!-- README-LANG-END -->

<p align="center">
  <a href="https://apps.apple.com/app/better-resource-monitor/id6758237306" target="_blank"><img src="https://better-resource-monitor.alexpedersen.dev/badges/appstore.webp" alt="Download on the Mac App Store" width="270" height="65"></a>
  <a href="https://github.com/alexx855/better-resource-monitor/releases" target="_blank"><img src="https://better-resource-monitor.alexpedersen.dev/badges/macos.webp" alt="Download macOS on GitHub Releases" width="270" height="65"></a>
</p>

<p align="center">
  <a href="#why">Why</a> <span>•</span>
  <a href="#comparison">Comparison</a> <span>•</span>
  <a href="#installation">Installation</a>
</p>

## Why

Better Resource Monitor is for people who just want a quick, low-noise way to keep an eye on their Mac.

It keeps CPU, memory, GPU, and network usage in the menu bar, so you can catch unusual load without stopping what you're doing or opening Activity Monitor.

It is built to stay light enough to leave running all day: under 0.1% CPU on Apple Silicon, around 15 MB of memory, and zero network requests.

The technical side is there to support that experience. It installs like a normal Mac app, stays sandboxed, ships on the Mac App Store with full features, and avoids admin-password prompts or background root helpers.

## Comparison

<table>
  <thead>
    <tr>
      <th width="20%">Feature</th>
      <th width="20%">Better Resource Monitor</th>
      <th width="20%"><a href="https://better-resource-monitor.alexpedersen.dev/vs-stats/">Stats</a></th>
      <th width="20%"><a href="https://better-resource-monitor.alexpedersen.dev/vs-eul/">Eul</a></th>
      <th width="20%"><a href="https://better-resource-monitor.alexpedersen.dev/vs-istat-menus/">iStat Menus</a></th>
    </tr>
  </thead>
  <tbody>
    <tr><th scope="row">Mac App Store</th><td align="center">Yes (full features)</td><td align="center">No</td><td align="center">Limited</td><td align="center">Limited</td></tr>
    <tr><th scope="row">Admin Password / Privileges</th><td align="center">None (sandboxed)</td><td align="center">Requires root helper</td><td align="center">None</td><td align="center">Requires root helper</td></tr>
    <tr><th scope="row">GPU API Stability</th><td align="center">Public API</td><td align="center">Private API</td><td align="center">Private API</td><td align="center">Proprietary</td></tr>
    <tr><th scope="row">Memory Footprint</th><td align="center">~15 MB</td><td align="center">~50 MB</td><td align="center">~40 MB</td><td align="center">~100+ MB</td></tr>
    <tr><th scope="row">CPU / Energy Impact</th><td align="center">&lt; 0.1%</td><td align="center">~1%</td><td align="center">High (M-series)</td><td align="center">~1%</td></tr>
    <tr><th scope="row">App Size</th><td align="center">&lt; 7 MB</td><td align="center">~25 MB</td><td align="center">~5 MB</td><td align="center">~65 MB</td></tr>
    <tr><th scope="row">Privacy/Telemetry</th><td align="center">100% offline</td><td align="center">Offline</td><td align="center">Offline</td><td align="center">Analytics</td></tr>
    <tr><th scope="row">Status</th><td align="center">Active</td><td align="center">Active</td><td align="center">Unmaintained</td><td align="center">Active</td></tr>
    <tr><th scope="row">Language</th><td align="center">Rust</td><td align="center">Swift / C++</td><td align="center">Swift</td><td align="center">Obj-C / Swift</td></tr>
    <tr><th scope="row">Price</th><td align="center">Free</td><td align="center">Free</td><td align="center">Free</td><td align="center">$14.99</td></tr>
    <tr><th scope="row">License</th><td align="center">MIT</td><td align="center">MIT</td><td align="center">MIT</td><td align="center">Proprietary</td></tr>
  </tbody>
</table>

> Third-party numbers are rough estimates. Your mileage may vary.

For more detail, read the <a href="https://better-resource-monitor.alexpedersen.dev/vs-stats/">Stats comparison</a> or the <a href="https://better-resource-monitor.alexpedersen.dev/vs-istat-menus/">iStat Menus comparison</a>.

## Installation

Get it from the <a href="https://apps.apple.com/app/better-resource-monitor/id6758237306" target="_blank">Mac App Store</a> (includes automatic updates) or grab the `.dmg` from <a href="https://github.com/alexx855/better-resource-monitor/releases" target="_blank">GitHub Releases</a>.

### Compatibility

Works on Intel Macs and Apple Silicon Macs running macOS Ventura 13 or newer.

### Build from Source

You'll need <a href="https://v2.tauri.app/start/prerequisites/" target="_blank">Tauri v2 prerequisites</a> and <a href="https://pnpm.io/" target="_blank">pnpm</a>.

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


- <a href="https://github.com/phosphor-icons" target="_blank">Phosphor Icons</a> - Icon set used in the tray
- <a href="https://alexpedersen.dev/" target="_blank">Alex Pedersen</a> - Maintainer
