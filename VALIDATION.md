# v1.0.0 本机验证（2026-10-11）

环境：Apple Silicon arm64，macOS 26.7.1，Rust 1.97.1；部署目标 macOS 12.0。

- `cargo test --offline`：7 / 7 通过。
- `cargo clippy --offline -- -D warnings`：通过，无警告。
- Release 构建：无第三方 crate，无网络下载。
- App bundle `plutil -lint` / `codesign --verify --strict`：通过。
- `--list-screens`：识别 U32J59x（1920×1080 points，2×，实际 3840×2160 像素）、C27F390（1920×1080 points，1×）、DELL E2016HL（1600×900 points，1×）。不把逻辑尺寸冒充物理分辨率。
- 本机主屏窗口构建：不透明黑色背景，自动退出。
- `--screen 2 --smoke-test`：实际窗口归属 C27F390，黑色图层、不透明均为 true；自动退出。
- `--screen 3 --fullscreen --smoke-test`：实际窗口归属 DELL E2016HL，原生 fullscreen=true；窗口 frame 与该屏幕 frame 一致，covers-display=true；图层颜色所有色分量为 0、alpha=1；自动退出恢复。

以上是原生窗口属性/几何和生命周期验证，不是照度计测得的光学黑位。当前仅在这台 Mac 和这些已连接显示器上实测；macOS 12 的最低部署版本经过二进制检查，但未在旧系统实机回归。不宣称降低 LCD 背光或关闭显示器。
