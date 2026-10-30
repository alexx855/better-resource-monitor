cask "better-resource-monitor" do
  version "1.1.9"
  sha256 "33db6ee34d9d2de48aa56bfe214bb21a2ccf8b0de5ab82e96791c750c9c2dbc0"

  url "https://github.com/alexx855/better-resource-monitor/releases/download/v#{version}/Better-Resource-Monitor-#{version}-macOS-universal.zip"
  name "Better Resource Monitor"
  desc "Menu bar system resource monitor"
  homepage "https://github.com/alexx855/better-resource-monitor"

  livecheck do
    url :url
    strategy :github_latest
  end

  depends_on macos: :ventura

  app "Better Resource Monitor.app"

  zap trash: [
    "~/Library/Application Support/dev.alexpedersen.better-resource-monitor",
    "~/Library/Caches/dev.alexpedersen.better-resource-monitor",
    "~/Library/Preferences/dev.alexpedersen.better-resource-monitor.plist",
    "~/Library/WebKit/dev.alexpedersen.better-resource-monitor",
  ]
end
