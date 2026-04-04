# air-controller

`AirController` 是一个基于 `Tauri + React` 的跨平台空调控制桌面应用。

支持目标：

1. Windows
2. macOS
3. Linux（优先 GNOME）

## 当前能力

1. 主控制页：刷新状态、开机、关机、温度加减、执行自动开机
2. 配置页：保存基础配置、保存或删除 Token
3. Token 仅保存到系统密钥链，不写入配置文件
4. 支持导入旧版 `appsettings.json`
5. 支持导出不含 Token 的新配置
6. 支持托盘菜单、关闭隐藏、最小化隐藏
7. 支持系统自启动开关

## 项目结构

```text
src/
  app/
  pages/
  lib/
  styles/
src-tauri/
  src/
```

## 本地开发

前置要求：

1. Node.js 20+
2. Rust stable
3. Linux 下需要 GTK/WebKit 开发依赖

## 安装流程

### Linux

适用于 `Ubuntu / GNOME` 的推荐安装步骤：

1. 安装系统依赖：

```bash
sudo apt-get update
sudo apt-get install -y \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libwebkit2gtk-4.1-dev \
  build-essential \
  pkg-config \
  patchelf
```

2. 安装 Rust：

```bash
curl https://sh.rustup.rs -sSf | sh -s -- -y
. "$HOME/.cargo/env"
```

3. 安装前端依赖：

```bash
npm install
```

4. 本地检查：

```bash
npm run check:all
```

5. 启动开发模式：

```bash
npm exec tauri dev
```

如果当前环境没有图形会话，可以先用无头方式验证：

```bash
npm run check:headless
```

### Windows

推荐在 `Windows 10/11` 上使用 PowerShell：

1. 安装 Node.js 20+
2. 安装 Rust stable：

```powershell
winget install Rustlang.Rustup
rustup default stable
```

3. 安装 Visual Studio C++ 构建工具：

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools
```

需要确保已安装 `Desktop development with C++` 工作负载。

4. 安装前端依赖：

```powershell
npm install
```

5. 本地检查：

```powershell
npm run check:web
npm run check:rust
```

6. 启动开发模式：

```powershell
npm exec tauri dev
```

如果后续使用 release 产物安装，通常会得到 `.msi` 或 `.exe` 安装包。

安装后建议确认：

1. 应用能正常启动
2. 托盘图标正常显示
3. 自启动设置可生效

### macOS

推荐在较新的 `macOS` 版本上使用终端：

1. 安装 Xcode Command Line Tools：

```bash
xcode-select --install
```

2. 安装 Rust：

```bash
curl https://sh.rustup.rs -sSf | sh -s -- -y
source "$HOME/.cargo/env"
```

3. 安装 Node.js 20+

如果使用 Homebrew：

```bash
brew install node
```

4. 安装前端依赖：

```bash
npm install
```

5. 本地检查：

```bash
npm run check:web
npm run check:rust
```

6. 启动开发模式：

```bash
npm exec tauri dev
```

如果后续使用 release 产物安装，通常会得到 `.app` 或 `.dmg`。

首次打开时如果系统提示来源不明：

1. 到“系统设置 -> 隐私与安全性”允许打开
2. 或在 Finder 中右键应用后选择“打开”

## 构建与运行

上面的步骤面向开发环境。

如果是最终用户安装，建议直接使用 GitHub Release 提供的安装包，而不是从源码启动。

## 配置存储

可配置参数不会写在仓库里的固定文件中，而是写入系统配置目录下的 `config.json`。

当前 Rust 实现位置：`src-tauri/src/config.rs`

配置文件内容包括：

1. `baseUrl`
2. `climateEntityId`
3. `launchOnSystemStartup`
4. `autoPowerOnOnStartup`
5. `startupDelaySeconds`
6. `retryCount`
7. `defaultTemperature`
8. `minTemperature`
9. `maxTemperature`
10. `temperatureStep`

敏感信息不会写入这个文件：

1. `Token` 只保存到系统密钥链
2. 不会落到 `config.json`
3. 导出配置时默认也不会带出 token

按当前实现，配置文件通常在这些位置：

### Linux

通常是：`~/.config/air-controller/config.json`

### Windows

通常在当前用户的 `AppData/Roaming` 下，例如：

`C:\Users\<用户名>\AppData\Roaming\opencode\air-controller\config\config.json`

### macOS

通常在用户的 `Application Support` 下，例如：

`~/Library/Application Support/air-controller/config.json`

如果你想确认程序实际写到了哪里，后面我也可以继续给应用加一个“打开配置目录”按钮。

### 最终用户安装

#### Linux

发布后通常会提供 `.deb`、`.AppImage` 或其他 Linux 包。

以 `.deb` 为例：

```bash
sudo dpkg -i air-controller_<version>_amd64.deb
sudo apt-get install -f
```

安装完成后：

1. 从应用菜单启动 `AirController`
2. 首次进入配置页填写服务地址、空调 ID 和访问令牌
3. 确认系统密钥服务可用，否则 token 无法保存

#### Windows

发布后通常会提供 `.msi` 或 `.exe`。

安装步骤通常为：

1. 双击安装包
2. 按向导完成安装
3. 从开始菜单启动 `AirController`
4. 首次进入配置页填写服务地址、空调 ID 和访问令牌

#### macOS

发布后通常会提供 `.dmg`。

安装步骤通常为：

1. 打开 `.dmg`
2. 将 `AirController.app` 拖入 `Applications`
3. 从启动台或 `Applications` 打开应用
4. 首次进入配置页填写服务地址、空调 ID 和访问令牌

如果发布版本尚未签名或 notarize，首次打开可能需要手动放行。

前端构建：

```bash
npm run build
```

Rust 检查：

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Tauri 开发模式：

```bash
npm exec tauri dev
```

## 旧配置迁移

当前支持导入旧版 WPF 项目的 `appsettings.json`。

迁移规则：

1. 普通配置字段迁入新配置文件
2. 明文 `LongLivedToken` 迁入系统密钥链
3. 旧版 `EncryptedLongLivedToken` 不做跨平台解密迁移，需要重新填写 Token

## 已验证

1. `npm run build` 通过
2. `cargo check --manifest-path src-tauri/Cargo.toml` 通过
3. `xvfb-run -a npm exec tauri dev` 可启动应用进程
4. `npm run check:all` 可作为本地统一检查入口

## CI / 发布

仓库包含两个 workflow：

1. `CI`
   执行前端构建和 Linux `cargo check`
2. `Release Build`
   支持手动触发，或在推送 `v*` tag 时执行多平台打包

建议发布流程：

1. 更新 `src-tauri/tauri.conf.json` 和 `package.json` 版本号
2. 提交代码并打 tag，例如 `v0.1.0`
3. 推送 tag 后等待 `Release Build` workflow 生成草稿 release
4. 检查产物后再发布

也可以直接用脚本同步版本号：

```bash
npm run release:sync-version -- 0.1.0
```

如果手动触发 `Release Build`：

1. 填写 `tag_name`，例如 `v0.1.0`
2. 可选填写 `release_name`

## 打包说明

本项目当前使用的是占位图标，只适合开发和 CI 验证。

正式发布前建议补齐：

1. `src-tauri/icons/` 下的正式图标资源
2. Windows 的 `.ico`
3. macOS 的 `.icns`
4. Linux 的 `.png`

详细检查项见：

1. `docs/release-checklist.md`
2. `docs/icon-assets.md`
3. `docs/manual-test-gnome.md`
4. `docs/developer-guide.md`
5. `docs/user-guide.md`
6. `docs/release-process.md`

同时建议在真实桌面环境下验证：

1. 托盘菜单
2. 关闭隐藏
3. 最小化隐藏
4. 自启动
5. 密钥链存储

## 平台注意事项

Windows：

1. 自启动依赖当前用户登录环境
2. 正式发布前建议补签名与图标

macOS：

1. 正式分发通常需要签名与 notarization
2. Keychain 权限提示需要实机确认

Linux：

1. 当前优先验证 GNOME
2. 托盘行为可能受桌面环境差异影响
3. Secret Service 需要系统密钥服务可用

## 待继续完善

1. 三平台实机验证
2. GNOME 托盘行为回归验证
3. 正式图标资源
4. Windows / macOS 发布签名流程

## 无头验证

如果当前 Linux 环境没有图形会话，可以通过 `xvfb` 做基础启动验证：

```bash
xvfb-run -a sh -lc '. "$HOME/.cargo/env" && npm exec tauri dev'
```

这能验证：

1. 前端开发服务可启动
2. Rust/Tauri 可完成编译
3. 桌面应用进程可被拉起
