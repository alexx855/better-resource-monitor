<p align="center">
  <img src="https://better-resource-monitor.alexpedersen.dev/better-resource-monitor.png" alt="Better Resource Monitor" width="830" height="86">
</p>


<h1 align="center">Better Resource Monitor</h1>

<p align="center">
  <strong>Monitor CPU, memory, storage, GPU, and network from your Mac menu bar.</strong>
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
  <a href="#installation">Installation</a> <span>•</span>
  <a href="#comparison">Comparison</a>
</p>

## Why

Better Resource Monitor is for people who just want a quick, low-noise way to keep an eye on their Mac.

It keeps CPU, memory, storage, GPU, and network usage in the menu bar, so you can catch unusual load without stopping what you're doing or opening Activity Monitor.

If you're evaluating an iStat Menus alternative, this is the same place to do it: a lightweight monitor for everyday metrics with low overhead and no telemetry.

## Best fit

Choose Better Resource Monitor if you want a lightweight Mac menu bar monitor for a quick daily view of CPU, memory, storage, GPU, and network usage. It is free, open source, sandboxed, works offline, and does not need an admin password or root helper. It runs on Intel and Apple Silicon Macs with macOS 13 or newer.

Choose Stats or iStat Menus instead if you need fan, temperature, battery, deep sensor history, or broader hardware controls. Better Resource Monitor intentionally stays focused on the core metrics people check every day.

## IStat Menus alternative FAQ

### Can Better Resource Monitor replace iStat Menus?

Yes, when your goals are visibility and daily stability. iStat Menus is excellent for deep system control; Better Resource Monitor is for keeping the core numbers easy to read all day with less setup.

### Is it free?

Yes. Better Resource Monitor is free and MIT licensed. The Mac App Store build and GitHub build are the same app.

### Does it send telemetry or collect usage data?

No. Better Resource Monitor has zero network requests. No analytics and no telemetry are sent.

### Does it run quietly on a battery-powered Mac?

Yes. It is built to sit in the background with very low impact so it stays usable during daily work.

If this is useful, star the repo so other Mac users can find it.

## Installation

Get it from the <a href="https://apps.apple.com/app/better-resource-monitor/id6758237306" target="_blank">Mac App Store</a> (includes automatic updates) or grab the `.dmg` from <a href="https://github.com/alexx855/better-resource-monitor/releases" target="_blank">GitHub Releases</a> (no automatic updates; download and update manually each version).

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

## Comparison

Quick starts:

- [Better Resource Monitor vs iStat Menus](https://better-resource-monitor.alexpedersen.dev/comparison/vs-istat-menus/)
- [Better Resource Monitor vs Stats](https://better-resource-monitor.alexpedersen.dev/comparison/vs-stats/)
- [Better Resource Monitor vs Eul](https://better-resource-monitor.alexpedersen.dev/comparison/vs-eul/)

<table>
  <thead>
    <tr>
      <th width="20%">Feature</th>
      <th width="20%">Better Resource Monitor</th>
      <th width="20%"><a href="https://better-resource-monitor.alexpedersen.dev/comparison/vs-stats/">Stats</a></th>
      <th width="20%"><a href="https://better-resource-monitor.alexpedersen.dev/comparison/vs-eul/">Eul</a></th>
      <th width="20%"><a href="https://better-resource-monitor.alexpedersen.dev/comparison/vs-istat-menus/">iStat Menus</a></th>
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

## Credits


- <a href="https://github.com/phosphor-icons" target="_blank">Phosphor Icons</a> - Icon set used in the tray
- <a href="https://alexpedersen.dev/" target="_blank">Alex Pedersen</a> - Maintainer
