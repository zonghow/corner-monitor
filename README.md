# Corner Monitor（Tauri + React）

一个贴在屏幕角落的系统监控小挂件，支持拖拽吸附、托盘配置与颜色/布局切换。

<img width="890" height="714" alt="CleanShot 2026-02-02 at 14 48 11@2x" src="https://github.com/user-attachments/assets/b823bf93-2dc6-419a-9dbd-0543e0b1a149" />

## 安装

**使用 brew**
```bash
brew install zonghow/homebrew-corner-monitor/corner-monitor
```
卸载
```bash
brew uninstall corner-monitor
```

**dmg 安装**

[Releases Page](https://github.com/zonghow/corner-monitor/releases) 下载最新的安装包

然后在终端运行脚本后打开
```bash
xattr -cr /Applications/Corner\ Monitor.app/
```

## 功能

- 角落监控：CPU / 内存 / 网络实时显示
- 拖拽吸附：拖到任意屏幕后松开，自动吸附到最近角落（以屏幕边缘为基准）
- 多屏支持：根据窗口所在屏幕自动吸附
- 布局切换：右键点击窗口切换横/竖布局（托盘也可切换）
- 颜色切换：托盘“颜色”菜单快速切换文字颜色

## 运行与开发

```bash
# 安装依赖
pnpm install

# 启动开发
pnpm tauri dev

# 构建
pnpm tauri build
```

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
