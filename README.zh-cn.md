<p align="center">
  <img src="https://better-resource-monitor.alexpedersen.dev/better-resource-monitor.png" alt="Better Resource Monitor" width="830" height="86">
</p>


<h1 align="center">Better Resource Monitor</h1>

<p align="center">
  <strong>在 Mac 菜单栏查看 CPU、内存、GPU 和网络使用情况。</strong>
</p>

<!-- README-LANG-START -->

<p align="center">
  <a href="README.md">English</a> •
  <a href="README.es.md">Español</a> •
  <a href="README.pt-br.md">Português (Brasil)</a> •
  简体中文
</p>

<!-- README-LANG-END -->


<p align="center">
  <a href="https://apps.apple.com/app/better-resource-monitor/id6758237306" target="_blank"><img src="https://better-resource-monitor.alexpedersen.dev/badges/appstore-zh-cn.webp" alt="在 Mac App Store 下载" width="270" height="65"></a>
  <a href="https://github.com/alexx855/better-resource-monitor/releases" target="_blank"><img src="https://better-resource-monitor.alexpedersen.dev/badges/macos-zh-cn.webp" alt="从 GitHub Releases 下载 macOS 版" width="270" height="65"></a>
</p>

## 为什么选择它

Better Resource Monitor 适合那些只想用一种简单方式看看自己的 Mac 状态的人。

它会把 CPU、内存、GPU 和网络使用情况直接放在菜单栏里，让你不用打开活动监视器，也不用钻进系统工具，就能更快发现异常负载。

它也被设计得足够轻量，这样监视器本身不会变成问题的一部分。

如果你在寻找 iStat Menus 的替代方案，这里就是你的起点：轻量级日常监控，低开销且不带遥测。

## FAQ：iStat Menus 替代方案

### Better Resource Monitor 能替代 iStat Menus 吗？

可以，在你想要日常稳定可读监控时。iStat Menus 适合深度系统控制；Better Resource Monitor 更适合只关注 CPU、内存、GPU 和网络等核心指标的轻量日常使用。

### 是否免费？

是，Better Resource Monitor 免费且采用 MIT 许可证。Mac App Store 版本与 GitHub 版本是同一款应用。

### 会发送用户数据或遥测吗？

不会。Better Resource Monitor 不发起任何网络请求，不采集分析，也不发送遥测。

### 会明显影响电池续航吗？

不会。它的目标是以很低的资源占用常驻后台，适合日常使用。


## 安装

从 <a href="https://apps.apple.com/app/better-resource-monitor/id6758237306" target="_blank">Mac App Store</a> 获取（包含自动更新），或从 <a href="https://github.com/alexx855/better-resource-monitor/releases" target="_blank">GitHub Releases</a> 下载 `.dmg` 文件（不含自动更新；每个版本需手动下载更新）。

### 兼容性

支持运行 macOS Ventura 13 或更高版本的 Intel Mac 和 Apple Silicon Mac。

### 从源码构建

你需要 <a href="https://v2.tauri.app/start/prerequisites/" target="_blank">Tauri v2 前置条件</a> 和 <a href="https://pnpm.io/" target="_blank">pnpm</a>。

```bash
git clone https://github.com/alexx855/better-resource-monitor.git
cd better-resource-monitor
pnpm install
pnpm tauri build
```

### 开发

```bash
# 以开发模式运行并热重载
pnpm tauri dev

# 运行测试
cd src-tauri && cargo test

# 运行覆盖率测试 (需要 cargo-llvm-cov)
cargo install cargo-llvm-cov
cd src-tauri && cargo llvm-cov --lib --html --output-dir coverage/
```

## 对比

快捷跳转：

- [Better Resource Monitor 与 iStat Menus 对比](https://better-resource-monitor.alexpedersen.dev/zh-cn/comparison/vs-istat-menus/)
- [Better Resource Monitor 与 Stats 对比](https://better-resource-monitor.alexpedersen.dev/zh-cn/comparison/vs-stats/)
- [Better Resource Monitor 与 Eul 对比](https://better-resource-monitor.alexpedersen.dev/zh-cn/comparison/vs-eul/)

<table>
  <thead>
    <tr>
      <th width="20%">功能</th>
      <th width="20%">Better Resource Monitor</th>
      <th width="20%"><a href="https://better-resource-monitor.alexpedersen.dev/zh-cn/comparison/vs-stats/">Stats</a></th>
      <th width="20%"><a href="https://better-resource-monitor.alexpedersen.dev/zh-cn/comparison/vs-eul/">Eul</a></th>
      <th width="20%"><a href="https://better-resource-monitor.alexpedersen.dev/zh-cn/comparison/vs-istat-menus/">iStat Menus</a></th>
    </tr>
  </thead>
  <tbody>
    <tr><th scope="row">Mac App Store</th><td align="center">是 (功能完整)</td><td align="center">否</td><td align="center">受限</td><td align="center">受限</td></tr>
    <tr><th scope="row">管理员密码 / 权限</th><td align="center">无需 (沙盒)</td><td align="center">需要 root 辅助工具</td><td align="center">无需</td><td align="center">需要 root 辅助工具</td></tr>
    <tr><th scope="row">GPU API 稳定性</th><td align="center">公开 API</td><td align="center">私有 API</td><td align="center">私有 API</td><td align="center">专有</td></tr>
    <tr><th scope="row">内存占用</th><td align="center">~15 MB</td><td align="center">~50 MB</td><td align="center">~40 MB</td><td align="center">~100+ MB</td></tr>
    <tr><th scope="row">CPU / 能源影响</th><td align="center">&lt; 0.1%</td><td align="center">~1%</td><td align="center">高 (M 系列)</td><td align="center">~1%</td></tr>
    <tr><th scope="row">应用大小</th><td align="center">&lt; 7 MB</td><td align="center">~25 MB</td><td align="center">~5 MB</td><td align="center">~65 MB</td></tr>
    <tr><th scope="row">隐私/遥测</th><td align="center">100% 离线</td><td align="center">离线</td><td align="center">离线</td><td align="center">包含分析</td></tr>
    <tr><th scope="row">状态</th><td align="center">活跃</td><td align="center">活跃</td><td align="center">停止维护</td><td align="center">活跃</td></tr>
    <tr><th scope="row">语言</th><td align="center">Rust</td><td align="center">Swift / C++</td><td align="center">Swift</td><td align="center">Obj-C / Swift</td></tr>
    <tr><th scope="row">价格</th><td align="center">免费</td><td align="center">免费</td><td align="center">免费</td><td align="center">$14.99</td></tr>
    <tr><th scope="row">许可证</th><td align="center">MIT</td><td align="center">MIT</td><td align="center">MIT</td><td align="center">专有</td></tr>
  </tbody>
</table>

> 第三方数据为粗略估算。实际情况可能有所不同。

## 致谢


- <a href="https://github.com/phosphor-icons" target="_blank">Phosphor Icons</a> - 托盘中使用的图标集
- <a href="https://alexpedersen.dev/" target="_blank">Alex Pedersen</a> - 维护者
