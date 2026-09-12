# RasFocus PC 🛡️

A FocusMe-style website blocker for Windows. Built with **Rust + Tauri + React**.

Blocks sites at the OS level via the Windows hosts file — works across **all browsers** (Chrome, Firefox, Edge, Brave, etc.)

## Features

| Category | Details |
|---|---|
| 🔞 Adult Content | 30+ major adult sites |
| 📱 Social Media | Facebook, Instagram, TikTok, Twitter, Reddit, Discord... |
| 📺 YouTube Shorts | Block Shorts without blocking full YouTube |
| ▶️ YouTube Full | Block all of YouTube |
| 🎰 Gambling | Major betting/casino sites |
| ✏️ Custom Sites | Add any domain |
| 🕐 Schedule | Auto-block during set hours |

## How it works

Modifies `C:\Windows\System32\drivers\etc\hosts` to redirect blocked domains to `127.0.0.1`. No browser extensions needed.

> ⚠️ **Must be run as Administrator** to write to the hosts file.

## Build from source

```bash
# Install dependencies
npm install

# Dev mode
npm run tauri dev

# Build installer
npm run tauri build
```

### Requirements
- Node.js 18+
- Rust (stable) — https://rustup.rs
- Windows (for the actual blocking; cross-compile via GitHub Actions)

## Download

See [Releases](../../releases) for the latest `.exe` installer.

## Stack

- **Backend:** Rust (hosts file manipulation, state management)
- **Frontend:** React + TypeScript + Vite
- **Desktop:** Tauri v2
- **Installer:** NSIS / MSI via `tauri-action`
