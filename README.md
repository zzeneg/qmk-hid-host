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

1. Download and start [qmk-hid-host.exe](dist/qmk-hid-host.exe)
2. The app will create a configuration json file in the same folder
3. Edit that config with your options and restart the app

## Changelog

- 2023-07-30 - rewritten to Rust
