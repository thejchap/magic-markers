# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with
code in this repository.

## Overview

This is an embedded Rust project for an ESP32-C6 microcontroller that creates a
"magic markers" system. It controls LED smart bulbs using Crayola markers with
RFID tags. When a marker is detected by an RFID reader, the system changes the
color of a connected Tasmota smart bulb to match the marker's color.

## Architecture

### Hardware Components

- **ESP32-C6**: Main microcontroller (target: `riscv32imac-unknown-none-elf`)
- **MFRC522 RFID Reader**: Connected via I2C (address 0x28, GPIO1=SCL,
  GPIO2=SDA)
- **LED**: Status indicator on GPIO7
- **Button**: User input on GPIO9 with pull-up resistor
- **Tasmota Smart Bulb**: Connected via WiFi network

### Software Architecture

- **Embassy Framework**: Async runtime for embedded Rust
- **WiFi Access Point**: ESP32 creates "magic-markers" network (192.168.2.1)
- **Task-based Design**: Multiple async tasks handle different responsibilities:
  - `rfid_task`: Reads RFID tags and maps to colors
  - `led_task`: Controls status LED with flash patterns
  - `button_task`: Handles button press for white light
  - `connection_task`: Manages WiFi access point
  - `bulb_commands_task`: Sends HTTP commands to Tasmota bulb
  - `net_task`: Handles network stack

### Networking

- ESP32 runs as WiFi access point (SSID: "magic-markers", password:
  "magic-markers")
- Tasmota bulb connects with static IP 192.168.2.2
- Commands sent via HTTP POST to `http://192.168.2.2/cm?cmnd=<command>`

### Color System

12 predefined marker colors with specific RFID UIDs and HSB color mappings. Each
marker has a unique 7-byte UID pattern.

## Development Commands

### Build

```bash
cargo build                    # Debug build
cargo build --release          # Release build
```

### Flash to Device

```bash
cargo run                      # Build and flash debug version
cargo run --release            # Build and flash release version
```

### Toolchain

- Uses stable Rust with `riscv32imac-unknown-none-elf` target
- Custom linker scripts and build configuration in `build.rs`
- Optimized for size (`opt-level = "s"`) even in debug builds
