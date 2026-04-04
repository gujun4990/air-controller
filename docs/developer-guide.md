# Developer Guide

## Overview

`AirController` 是一个基于 `Tauri + React + TypeScript + Rust` 的桌面应用，用于控制服务中的空调设备。

核心边界：

1. React 负责界面与交互
2. Rust 负责配置、系统集成、密钥链、自启动、托盘和 HA API 调用
3. Token 不写入配置文件，只进系统密钥链

## Repository Structure

```text
src/
  app/
  pages/
  lib/
  styles/
src-tauri/
  src/
  icons/
docs/
```

关键文件：

1. `src/app/App.tsx`
2. `src/pages/MainPage.tsx`
3. `src/pages/ConfigPage.tsx`
4. `src-tauri/src/commands.rs`
5. `src-tauri/src/config.rs`
6. `src-tauri/src/ha_client.rs`
7. `src-tauri/src/secure_store.rs`
8. `src-tauri/src/startup.rs`
9. `src-tauri/src/tray.rs`

## Development Setup

### Linux

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
curl https://sh.rustup.rs -sSf | sh -s -- -y
. "$HOME/.cargo/env"
npm install
```

### Windows

1. 安装 Node.js 20+
2. 安装 Rust stable
3. 安装 Visual Studio Build Tools，并启用 `Desktop development with C++`
4. 执行 `npm install`

### macOS

1. 安装 Xcode Command Line Tools
2. 安装 Rust stable
3. 安装 Node.js 20+
4. 执行 `npm install`

## Local Commands

```bash
npm run build
npm run check:web
npm run check:rust
npm run check:all
npm run release:sync-version -- 0.1.1
npm exec tauri dev
```

无图形环境下可用：

```bash
npm run check:headless
```

## Configuration Model

配置文件写入系统配置目录下的 `config.json`。

字段：

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

敏感数据：

1. `Token` 只存系统密钥链
2. 导出配置默认不含 token

## Development Workflow

1. 修改前端页面或 Rust 模块
2. 执行 `npm run check:all`
3. 如需桌面启动验证，执行 `npm exec tauri dev`
4. 如无图形环境，执行 `npm run check:headless`
5. 提交前更新相关文档

## Release Workflow

1. 更新版本号
2. 运行 `npm run check:all`
3. 在 GNOME 或目标平台做手工验证
4. 推送 tag，例如 `v0.1.0`
5. 等待 `Release Build` workflow 产出草稿 release

更多细节：

1. `docs/release-checklist.md`
2. `docs/icon-assets.md`
3. `docs/manual-test-gnome.md`
