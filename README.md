<h1 align="center">Silicon Resources Monitor</h1>

<p align="center">
  <strong>A native macOS menu bar system monitor for Apple Silicon.</strong><br>
  Blazing-fast. Native. 100% Rust.
</p>

<p align="center">
  <img src="src/assets/sillicon-resources-monitor.png" alt="Silicon Resources Monitor" width="600">
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#why">Why</a> •
  <a href="#comparison">Comparison</a> •
  <a href="#installation">Installation</a>
</p>

---

## Features

*   **Zero Impact** — Written in Rust. Minimal CPU & Memory usage.
*   **Native Design** — Template icons adapt automatically to your wallpaper/theme.
*   **Apple Silicon Ready** — First-class support for M-series GPU monitoring.
*   **Fully Configurable** — Toggle CPU, GPU, Memory, or Network stats instantly.
*   **Battery Efficient** — Smart polling that won't drain your MacBook.

## Why?

I built this out of necessity. After switching from Ubuntu to macOS, I missed the simplicity of the **GNOME Resource Monitor** extension—just a clean line of text showing me exactly what my system was doing.

I tried everything else:
*   **Electron apps** were too heavy (~100MB+ RAM for a text label?).
*   **Paid apps** were overkill and expensive ($15+).
*   **Free alternatives** often lacked proper **Apple Silicon GPU** support or didn't match the OS aesthetics.

**Silicon Resources Monitor** is the answer. It uses **native macOS APIs** (IOReport) to monitor your Apple Silicon GPU with zero overhead, uses ~2MB of RAM, and looks exactly like a native system component in both light and dark modes. It is simply the best way to monitor your M1/M2/M3/M4/M5 Mac.

## Comparison

| Feature | Silicon Resources Monitor | iStat Menus | Stats | MenuMeters |
| :--- | :--- | :--- | :--- | :--- |
| **Price** | ✅ Free | ❌ $14.99 | ✅ Free | ✅ Free |
| **Open Source** | ✅ MIT | ❌ Proprietary | ✅ MIT | ✅ GPL |
| **Resources (RAM)** | ✅ ~2MB | ⚠️ ~50MB | ⚠️ ~80MB | ✅ ~15MB |
| **App Size** | ✅ < 5MB | ⚠️ ~40MB | ⚠️ ~30MB | ✅ ~10MB |
| **Energy Impact** | ✅ Very Low | ⚠️ Medium | ⚠️ Medium | ✅ Low |
| **GPU Monitoring** | ✅ Native (Residency) | ✅ Native | ⚠️ Limited | ❌ No |
| **Theme Support** | ✅ Auto | ✅ Auto | ✅ Manual | ⚠️ Limited |
| **Network Speed** | ✅ | ✅ | ✅ | ✅ |

## Installation

**Download** the latest binary from [Releases](../../releases).

**Build from Source**:
```bash
git clone https://github.com/alexx855/silicon-resources-monitor.git
cd silicon-resources-monitor
npm install
npm run tauri build
```

---

<p align="center">
  Made with ❤️ by <a href="https://alexpedersen.dev/">Alex Pedersen</a>
</p>
