# Cinema Black v1.0.0

首个正式版本：Rust + 原生 AppKit，指定屏幕影院黑窗。

- 拖动到目标屏幕后，用绿色全屏按钮、双击或 ⌘F 全屏黑屏。
- 菜单栏可选择显示器；全屏后先 ⌘F 恢复窗口再换屏。
- Esc / ⌘Q 退出，其他屏幕可继续使用。
- 提供 `--list-screens` / `--screen N --fullscreen`。
- 无第三方依赖、无后台服务、不修改系统亮度/显示设置，不访问网络。

下载 `cinema-black-v1.0.0-macos-arm64.zip`，解压后打开 `Cinema Black.app`。Apple Silicon macOS 12+；本机 macOS 26.7.1 验证。ad-hoc 签名，未公证；首次下载可能需右键打开确认。

7 项 Rust 测试、严格 Clippy、原生副屏全屏覆盖/黑色图层验证、签名验证通过。LCD 漏光和系统级提示仍受设备及 macOS 控制。详见仓库 VALIDATION.md。
