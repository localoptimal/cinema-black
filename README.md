# Cinema Black

Rust 编写的原生 macOS 影院关灯小程序，没有第三方依赖、后台服务、网络访问或亮度修改。

1. 打开 `Cinema Black.app`，把黑色窗口拖到想遮住的屏幕。
2. 点击绿色全屏按钮、双击黑色区域，或按 **⌘F**。窗口所在屏幕变为纯黑；其他屏幕不受程序遮挡。
3. **Esc 或 ⌘Q 退出**。⌘F 恢复窗口。

也可以在菜单栏“选择屏幕 Displays”中选屏幕，再全屏。全屏期间请先按 ⌘F 恢复窗口再换屏幕。显示器热插拔由 macOS 原生全屏/窗口迁移处理；重新启动即可刷新屏幕菜单。

命令行：

```sh
cinema-black --list-screens
cinema-black --screen 2 --fullscreen
```

首版支持 **Apple Silicon Mac，macOS 12+**。Intel/Windows/Linux 暂不支持。使用系统 AppKit 全屏，是否为每台显示器提供独立空间取决于 macOS 的“显示器具有单独的空间”设置。需要独立遮屏时请启用该设置；程序不会替你修改它。

这是黑色遮罩，不是关闭显示器电源。LCD 的背光仍可能漏光；不会假装实现物理关灯。鼠标移动到该屏幕时仍可能显示光标；系统安全提示、Mission Control 等系统界面也可能暂时出现。

## 构建与验证

```sh
cargo test --offline
cargo build --release --offline
target/release/cinema-black --list-screens
target/release/cinema-black --smoke-test
./tools/package.sh
```

发布包经过 ad-hoc 签名，未做 Apple 公证；首次从互联网下载后可能需要右键打开并确认。不要关闭系统安全机制。原生界面只使用主线程，静态画面无计时器/轮询/渲染循环；由 WindowServer 合成黑色窗口。

许可：MIT。
