# 🩺 EasyAppoint

Priority-based appointment scheduling for modern medical practices.

**Emergency > Urgent > Routine**

[![Release](https://github.com/MZAzeemi/easyappoint/actions/workflows/release.yml/badge.svg)](https://github.com/MZAzeemi/easyappoint/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.70+-blue)](https://www.rust-lang.org/)

## 📋 About

EasyAppoint is a command-line appointment scheduling system for medical practices. It uses a priority queue to ensure emergency patients are seen first, urgent cases second, and routine appointments fill remaining slots.

## ✨ Features

- 🎯 **Priority Queue** – Emergency > Urgent > Routine
- ⚡ **Smart Fallback** – Finds next available slot within flexibility window
- 🕐 **CLI-First** – Fast, lightweight, no cloud, no tracking
- 📊 **Batch Processing** – Handle dozens of requests at once
- 🔄 **Easy Rescheduling** – Cancel and rebook instantly
- 📦 **Single Binary** – Zero dependencies, download and run

## 🚀 Quick Start

```bash
# Linux (x86_64)
wget https://github.com/MZAzeemi/easyappoint/releases/latest/download/easyappoint-x86_64-unknown-linux-musl -O easyappoint
chmod +x easyappoint
./easyappoint

# Linux (ARM64) - Raspberry Pi
wget https://github.com/MZAzeemi/easyappoint/releases/latest/download/easyappoint-aarch64-unknown-linux-musl -O easyappoint
chmod +x easyappoint
./easyappoint


## 🎮 Usage

--- Main Menu ---
1. Setup doctor calendar
2. Generate time slots
3. Submit appointment request
4. Process all requests
5. View available slots
6. View confirmed appointments
7. Cancel appointment
8. Run demo
9. Exit

## 📦 Download

| Platform | Download |
|----------|----------|
| Linux x86_64 | [easyappoint-x86_64](https://github.com/MZAzeemi/easyappoint/releases) |
| Linux ARM64 | [easyappoint-aarch64](https://github.com/MZAzeemi/easyappoint/releases) |
| Windows | [easyappoint.exe](https://github.com/MZAzeemi/easyappoint/releases) |
| macOS | [easyappoint-macos](https://github.com/MZAzeemi/easyappoint/releases) |


## 🌐 Website

https://MZAzeemi.github.io/easyappoint

## 📄 License

MIT © MZAzeemi
