# Icon Assets

当前仓库中的 `src-tauri/icons/icon.png` 是占位图，只用于开发和 CI。

正式发布前建议准备以下资源：

1. `src-tauri/icons/32x32.png`
2. `src-tauri/icons/128x128.png`
3. `src-tauri/icons/128x128@2x.png`
4. `src-tauri/icons/icon.icns`
5. `src-tauri/icons/icon.ico`
6. `src-tauri/icons/icon.png`

建议规范：

1. 使用统一视觉母版导出不同尺寸
2. 深色和浅色桌面背景下都能识别
3. 托盘场景优先保证轮廓清晰
4. 小尺寸下避免细碎文本和复杂渐变

替换资源后，建议重新执行：

```bash
npm run check:web
npm run check:rust
```
