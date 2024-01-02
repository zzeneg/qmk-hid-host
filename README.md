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

#### Manually

1. Download and start [qmk-hid-host.exe](dist/qmk-hid-host.exe)
2. If needed, edit config in `qmk-hid-host.json` and restart the app.

#### As a Windows service

Windows service is supported using [WinSW](https://github.com/winsw/winsw) application. For full options or in case of any issues please refer to [documentation](https://github.com/winsw/winsw/tree/master?tab=readme-ov-file#documentation).

1. Download all files from [dist](dist/) folder.
2. Open console and run `winsw install`, followed by `winsw start`. Service should automatically start with windows.
3. If needed, edit config in `qmk-hid-host.json` and restart the service using `winsw stop` and `winsw start`.

## Changelog

- 2024-01-02 - support RUST_LOG, run as windows service
- 2023-07-30 - rewritten to Rust
