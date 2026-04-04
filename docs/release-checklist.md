# Release Checklist

## 版本准备

1. 更新 `package.json` 版本号
2. 更新 `src-tauri/tauri.conf.json` 版本号
3. 确认 `README.md` 中的运行方式和平台说明仍然准确

## 资源准备

1. 替换 `src-tauri/icons/icon.png` 占位图标
2. 补齐正式发布图标：
   - Windows: `.ico`
   - macOS: `.icns`
   - Linux: `.png`
3. 检查应用名称、包标识和图标是否一致

## 本地校验

1. `npm run check:web`
2. `npm run check:rust`
3. 无图形环境可选：`npm run check:headless`
4. 在真实桌面环境下至少手工验证一次：
   - 主界面开机/关机/调温/刷新
   - 配置保存
   - Token 保存到密钥链
   - 导入旧配置
   - 托盘菜单
   - 关闭隐藏
   - 最小化隐藏
   - 自启动设置

## CI / Release

1. 确认 `CI` workflow 通过
2. 推送测试 tag，例如 `v0.1.0-rc1`
3. 确认 `Release Build` workflow 三个平台都产出包
4. 下载产物进行基本冒烟测试

## 平台专项检查

### Windows

1. 自启动是否生效
2. 密钥链保存是否正常
3. 托盘菜单和窗口恢复是否正常

### macOS

1. Keychain 权限提示是否正常
2. 登录启动是否正常
3. 发布前是否需要签名和 notarization

### Linux

1. 先验证 GNOME
2. Secret Service 是否可用
3. 托盘是否可见且菜单可操作
4. `XDG autostart` 是否生效

## 发布收尾

1. 创建正式 tag，例如 `v0.1.0`
2. 检查 GitHub Draft Release 内容
3. 补充 changelog 或发布说明
4. 再发布正式版本
