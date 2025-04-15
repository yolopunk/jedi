# Jedi Hosts Manager

> A powerful hosts file management tool that makes it easy to manage and switch between different environment configurations.

English | [简体中文](README.zh-CN.md)

## 📷 Screenshot

![Jedi Hosts Manager Screenshot](public/screenshot.png)

*Note: This is a placeholder image. Please replace it with an actual screenshot of your application.*

## 📖 Introduction

Jedi Hosts Manager is a cross-platform application built with Tauri and Vue 3, designed for developers to simplify hosts file management. It allows you to create, edit, and switch between different hosts configuration groups, making development, testing, and deployment processes more efficient.

The name "Jedi" is inspired by the Jedi Knights from Star Wars, symbolizing how this tool helps developers master different network configurations with the ease of a Jedi.

## ✨ Features

- **Group Management**: Create hosts configuration groups by project or environment
- **One-Click Switching**: Quickly switch between different configurations
- **Global Toggle**: Enable/disable all hosts configurations with one click
- **Individual Entry Control**: Enable/disable specific hosts entries
- **System Tray**: Minimize to system tray, always available
- **Cross-Platform Support**: Works on Windows, macOS, and Linux
- **Modern Interface**: Beautiful UI based on Vuetify
- **Lightweight**: Low resource usage, fast startup

## 🚀 Installation

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [pnpm](https://pnpm.io/) (v8+)
- [Rust](https://www.rust-lang.org/) (v1.70+)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)

### Build Steps

1. Clone the repository

```bash
git clone https://github.com/yourusername/jedi.git
cd jedi
```

2. Install dependencies

```bash
pnpm install
```

3. Run in development mode

```bash
pnpm tauri dev
```

4. Build for production

```bash
pnpm tauri build
```

## 🔧 Usage Guide

### Basic Operations

1. **Add Group**: Click the "+" button to create a new hosts configuration group
2. **Add Entry**: Click the "Add" button in a group to add a new hosts entry
3. **Enable/Disable Entry**: Use the switch next to each entry to control its status
4. **Global Toggle**: Use the main switch at the top to control all hosts configurations

### Hosts File Format

Jedi uses special format markers to manage content in the hosts file:

```
# === JEDI HOSTS MANAGER ===
# +default+
127.0.0.1 localhost
# +development+
10.50.128.32 docker.dev.io
10.50.128.32 api.dev.io
# === END JEDI HOSTS MANAGER ===
```

Jedi only manages the content between these markers and does not modify other parts of the file.

## 🔒 Permissions

Jedi requires administrator privileges to modify the hosts file. On Windows, it will request UAC elevation; on macOS and Linux, you may need to enter an administrator password.

## 📄 License

This project is licensed under the MIT License.
