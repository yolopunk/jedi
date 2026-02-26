# Jedi Toolkit

> A multi-functional desktop toolkit for developers to boost productivity.

English | [简体中文](README.md)

## 📷 Screenshots

### Main Interface - Hosts Manager
![Hosts Manager Interface](public/images/screenshots/hosts-main.png)

### Knowledge Wall Gallery
![Knowledge Wall Gallery](public/images/screenshots/wallpapers-grid.png)

### Wallpaper Preview
![Wallpaper Preview](public/images/screenshots/wallpapers-preview.png)

### Podcast Library
![Podcast Library](public/images/screenshots/podcastic-library.png)

### Playing Podcast
![Playing Podcast](public/images/screenshots/podcastic-playing.png)

### App Settings
![App Settings](public/images/screenshots/settings-general.png)

### Dark Theme
![Dark Theme](public/images/screenshots/dark-theme-view.png)

## 📖 Introduction

Jedi Toolkit is a cross-platform desktop application built with Tauri v2, Vue 3 and Vuetify. It provides practical tools for daily development, including Hosts management, Knowledge Wall wallpapers, Xiaoyuzhou podcasts, system info, and settings.

The name "Jedi" is inspired by the Jedi Knights from Star Wars, symbolizing how this tool helps developers master different development environment configurations with the ease of a Jedi.

## ✨ Features

### Hosts Manager

- **Group Management**: Create hosts configuration groups by project or environment, keep your configs organized
- **One-Click Switching**: Quickly switch between different environments, say goodbye to manual editing
- **Global Toggle**: Enable/disable all hosts configurations with one click, fast mode switching
- **Individual Entry Control**: Precise control over each hosts entry, flexible for any scenario
- **Quick Domain Access**: Click to open domains directly in browser, save your time

### Knowledge Wall

- **Category & tag filtering**: Quickly find your favorite wallpaper themes, discover beauty
- **Preview & set wallpaper**: Preview details and apply with one click, refresh your desktop
- **Rich content view**: Read and view image/text content in detail

### Xiaoyuzhou Podcast

- **Subscription management**: Easily add, refresh, and unsubscribe from podcasts
- **OPML import**: One-click import existing subscriptions, seamless migration
- **Playback control**: Play latest episodes and quickly locate current playback position
- **Show notes**: Complete show notes display with one-click link navigation

### System & Experience

- **System Info Bar**: Real-time monitoring of CPU, memory, network and other key metrics
- **Theme Modes**: Support for dark, light, and system-following themes
- **System Tray**: Minimize to tray for quick access anytime
- **Multi-language**: Switch between Chinese and English UI freely
- **Cross-Platform Support**: Full support for Windows, macOS, and Linux
- **Modern Interface**: Consistent visual design and smooth interactions based on Vuetify 3

## ⚡ Quick Start

1. **Download & Install**: Download the installer for your platform from [GitHub Releases](https://github.com/yolopunk/jedi/releases)
2. **Run & Enjoy**: Launch the app immediately after installation, no configuration needed
3. **Get Started**:
   - Add your development environment configs in Hosts Manager
   - Browse Knowledge Wall to find your favorite desktop background
   - Import OPML to start subscribing to podcasts

### Who is it for?

- **Frontend Developers**: Frequently switch hosts between local/test/production environments
- **Full-stack Engineers**: Need to manage domain resolution for multiple projects
- **Podcast Enthusiasts**: Love listening to tech podcasts on Xiaoyuzhou
- **Efficiency Seekers**: Want one tool to handle multiple development tasks

## 🚀 Installation

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [pnpm](https://pnpm.io/) (v8+)
- [Rust](https://www.rust-lang.org/) (v1.70+)
- [Tauri CLI](https://tauri.app/v2/guides/getting-started/prerequisites) (v2.x)

### Development Steps

1. Clone the repository

```bash
git clone https://github.com/yolopunk/jedi.git
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

### Download and Install

You can also download pre-compiled installation packages directly from the [GitHub Releases](https://github.com/yolopunk/jedi/releases) page.

- Windows: `.msi` installer
- macOS: `.dmg` installer
- Linux: `.AppImage` or `.deb` package

## 🔧 Usage Guide

### Hosts Manager

1. **Add Group**: Click the "+" button to create a new hosts configuration group
2. **Add Entry**: Click the "Add Entry" button in a group to add a new hosts entry
3. **Enable/Disable Entry**: Use the switch next to each entry to control its status
4. **Global Toggle**: Use the main switch at the top to control all hosts configurations
5. **Access Domain**: Click on a domain to directly access it in the browser

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

### Knowledge Wall

1. **Browse & filter**: Use categories and tags to filter
2. **Preview wallpaper**: Open a wallpaper card for full details
3. **Set wallpaper**: Apply with one click

### Xiaoyuzhou Podcast

1. **Add subscriptions**: Input RSS or import OPML
2. **Browse shows**: View and search in the library
3. **Play & locate**: Play latest and locate the current playing

## 🔐 Permissions

Jedi requires administrator privileges to modify the hosts file. On Windows, it will request UAC elevation; on macOS and Linux, you may need to enter an administrator password.

## 💻 Technology Stack

- **Frontend**: Vue 3 + TypeScript + Vuetify 3
- **Backend**: Rust + Tauri v2
- **Build Tool**: Vite
- **Package Manager**: pnpm

## 📚 Project Structure

```
├── src/               # Frontend source code
│   ├── api/           # API calls
│   ├── assets/        # Static resources
│   ├── components/    # Vue components
│   ├── types/         # TypeScript type definitions
│   ├── utils/         # Utility functions
│   ├── App.vue        # Main application component
│   └── main.ts        # Application entry
├── src-tauri/         # Tauri/Rust backend
│   ├── src/           # Rust source code
│   │   ├── api/       # API implementations
│   │   ├── config/    # Configuration related
│   │   └── utils/     # Utility functions
│   ├── Cargo.toml     # Rust dependencies
│   └── tauri.conf.json # Tauri configuration
├── public/            # Public resources
├── index.html         # HTML template
├── package.json       # Project configuration
└── vite.config.ts     # Vite configuration
```

## 👨‍💻 Contribution Guide

Contributions to the project are welcome! If you want to contribute code, please follow these steps:

1. Fork this repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📝 License

This project is licensed under the MIT License.
