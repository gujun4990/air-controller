# Release Process

## 目标

本流程用于发布 `AirController` 的桌面安装包，并通过 GitHub Actions 自动生成多平台产物。

## 发布前准备

1. 确认功能改动已经完成
2. 执行本地检查：

```bash
npm run check:all
```

3. 如当前环境没有图形会话，可额外执行：

```bash
npm run check:headless
```

4. 在真实桌面环境下做手工验证，至少覆盖：
   - 托盘菜单
   - 关闭隐藏
   - 最小化隐藏
   - 配置保存
   - Token 保存
   - Home Assistant 开关机与调温
   - 自动开机

详细检查项见：

1. `docs/release-checklist.md`
2. `docs/manual-test-gnome.md`

## 更新版本号

发布前需要同步更新两个位置：

1. `package.json`
2. `src-tauri/tauri.conf.json`

例如从 `0.1.0` 升到 `0.1.1`。

也可以直接使用脚本同步：

```bash
npm run release:sync-version -- 0.1.1
```

## 更新变更记录

在 `CHANGELOG.md` 中增加新版本条目，说明：

1. 新增功能
2. 修复内容
3. 风险说明

## 提交代码

建议先做一次常规提交：

```bash
git add .
git commit -m "release: prepare v0.1.1"
```

## 创建发布标签

使用语义化版本标签，例如：

```bash
git tag v0.1.1
git push origin main --tags
```

推送 `v*` 标签后，GitHub Actions 会自动触发：

1. `CI`
2. `Release Build`

## Release Build Workflow

`Release Build` 会在以下平台打包：

1. Linux
2. macOS
3. Windows

它会：

1. 安装依赖
2. 构建前端
3. 构建 Tauri 应用
4. 创建 GitHub Draft Release
5. 上传安装包产物

对应 workflow 文件：

`/.github/workflows/release.yml`

## 手动触发发布

如果不想通过 git tag 触发，也可以在 GitHub Actions 页面手动运行 `Release Build`。

需要填写：

1. `tag_name`，例如 `v0.1.1`
2. 可选 `release_name`

## 发布后检查

当 Draft Release 生成后，建议：

1. 下载 Linux 包做一次安装验证
2. 下载 Windows 包做一次启动验证
3. 下载 macOS 包做一次启动验证
4. 检查图标、应用名、版本号是否正确
5. 检查托盘和自启动是否正常

## 正式发布

确认产物正常后：

1. 完善 GitHub Release 说明
2. 补充 changelog 摘要
3. 从 Draft 改为正式发布

## 当前限制

正式对外发布前，还建议补齐这些内容：

1. 正式图标资源
2. Windows 签名
3. macOS 签名与 notarization
4. GNOME 实机托盘回归测试
