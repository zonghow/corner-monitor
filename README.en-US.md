# Corner Monitor (Tauri + React)

A system monitoring widget that sticks to the corner of the screen, supporting CPU / memory / disk / network monitoring, drag-to-snap, tray configuration, and color/layout switching.

<img width="890" height="714" alt="CleanShot 2026-02-02 at 14 48 11@2x" src="https://github.com/user-attachments/assets/b823bf93-2dc6-419a-9dbd-0543e0b1a149" />

## Installation

**Using brew**
```bash
brew install zonghow/homebrew-corner-monitor/corner-monitor
```
Uninstall
```bash
brew uninstall corner-monitor
```

**DMG Installation**

Download the latest installer from the [Releases Page](https://github.com/zonghow/corner-monitor/releases)

Then run the following command in the terminal and open it
```bash
xattr -cr /Applications/Corner\ Monitor.app/
```

## Features

- Corner monitoring: real-time display of CPU / memory / disk / network
- Drag-to-snap: drag to any screen and release, automatically snaps to the nearest corner (based on screen edges)
- Multi-monitor support: automatically snaps based on the screen the window is on
- Layout switching: right-click the window to switch between horizontal/vertical layout (also switchable via tray)
- Quick action: double-click the window to open macOS Activity Monitor
- Color switching: use the "Color" tray menu to quickly switch text color

## Running & Development

```bash
# Install dependencies
pnpm install

# Start development
pnpm tauri dev

# Build
pnpm tauri build
```

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
