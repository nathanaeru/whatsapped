# WhatsApped

A lightweight, high-performance WhatsApp Web client wrapper for Windows, engineered using Rust and the Tauri v2 framework.

## Features

- **Ultra-Lightweight:** Consumes significantly less memory and resources compared to the official electron-based desktop client.
- **System Tray Integration:** Minimize to tray seamlessly on closing (`X`), keeping your taskbar clean while staying active in the background. Also it can be set to open on startup.
- **Native Notifications:** Fully hooked into the Windows Action Center for native system flyout banners.
- **Smart Link Routing:** Internal WhatsApp links and deep-links route natively inside the app, while external web links securely bounce out to your system's default browser.
- **Deep Linking Protocol Support:** Registers `whatsapp://` and `wapped://` schemes at the OS level to automatically catch chat intents from your default web browsers.

## Installation

You can download the latest automated build directly from our repository's production pipeline:

1. Go to the [Releases](https://github.com/nathanaeru/whatsapped/releases) section.
2. Download the latest `WhatsApped_*_x64-setup.exe` installer.
3. Run the installer.
4. Enjoy!