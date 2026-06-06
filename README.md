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
  Free, open source, and useful? <a href="https://github.com/alexx855/better-resource-monitor/stargazers">Star the repo</a> so other Mac users can find it.
</p>

<p align="center">
  <img src="https://better-resource-monitor.alexpedersen.dev/images/simplicity-en.png" alt="Better Resource Monitor showing CPU, memory, GPU, and network usage in the macOS menu bar" width="830">
</p>

<p align="center">
  CPU, memory pressure, GPU, and network signals stay visible without opening Activity Monitor.
</p>

<p align="center">
  <a href="#why">Why</a> <span>•</span>
  <a href="#comparison">Comparison</a> <span>•</span>
  <a href="#installation">Installation</a> <span>•</span>
  <a href="https://github.com/alexx855/better-resource-monitor/stargazers">Star on GitHub</a> <span>•</span>
  <a href="https://github.com/alexx855/better-resource-monitor/releases">Releases</a> <span>•</span>
  <a href="https://github.com/alexx855/better-resource-monitor/issues">Issues</a>
</p>

## Why

Activity Monitor is great when you are already investigating a problem. Better Resource Monitor is for the earlier moment: when you just want to know what your Mac is doing without opening another window.

It keeps the common signals visible in the menu bar:

- CPU load
- Memory pressure
- GPU usage
- Network activity

That makes it easier to spot unusual load while you are working, building, gaming, or testing apps.

It is built to stay light enough to leave running all day: under 0.1% CPU on Apple Silicon, around 15 MB of memory, and zero network requests.

The technical choices support that goal: it installs like a normal Mac app, stays sandboxed, ships on the Mac App Store with full features, uses public APIs, and avoids admin-password prompts or background root helpers.

If this is the kind of small Mac utility you want maintained, star the repo. It helps track interest and makes future updates easier to justify.

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

Get it from the <a href="https://apps.apple.com/app/better-resource-monitor/id6758237306" target="_blank">Mac App Store</a> for the signed app and automatic updates.

GitHub Releases currently provide source releases. A direct-download `.dmg` is planned for users who prefer installing outside the Mac App Store.

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
