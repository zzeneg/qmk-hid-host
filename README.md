# QMK HID Host

Host component for communicating with QMK keyboards using Raw HID feature.

Requires support on keyboard side, currently is supported by [stront](https://github.com/zzeneg/stront).

## Architecture

Application is written in Rust which gives easy access to HID libraries, low-level Windows APIs and potential cross-platform compatibility. Currently only Windows is supported, as I don't use other operating systems. Please feel free to raise any PRs with Linux/MacOS support.

## Supported providers

- Time
- Volume
- Input layout
- Media artist and song title

## How to run it

Download all files from [latest release](https://github.com/zzeneg/qmk-hid-host/releases/tag/latest).

#### Configuration

Default configuration is set to [stront](https://github.com/zzeneg/stront). For other keyboards you need to modify `qmk-hid-host.json`.

- `device` section contains information about keyboard. All values are **decimal**, make sure to convert them from hex using a [converter](https://tools.keycdn.com/hex-converter).
  - `productId` - `pid` from your keyboard's `info.json`
  - `usage` and `usagePage` - default values from QMK (`RAW_USAGE_ID` and `RAW_USAGE_PAGE`). No need to modify them unless they were redefined in firmware
- `layouts` - list of supported keyboard layouts in two-letter format
- `reconnectDelay` - delay between reconnecting attempts in milliseconds

#### Manual/Debug mode

1. Start `qmk-hid-host.exe`
2. If needed, edit config and restart the app

#### Silent mode

When you verified that the application works with your keyboard, you can use `qmk-hid-host.silent.exe` instead (like add it to Startup). It does not have a console or logs, and can be killed only from Task Manager.

## Development

1. Install Rust
2. Run `cargo run`
3. If needed, edit `qmk-hid-host.json` in root folder and run again

## Changelog

- 2024-01-21 - remove run as windows service
- 2024-01-02 - support RUST_LOG, run as windows service
- 2023-07-30 - rewritten to Rust
