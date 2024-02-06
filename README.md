# QMK HID Host

Host component for communicating with QMK keyboards using Raw HID feature.

Requires support on keyboard side, currently is supported by [stront](https://github.com/zzeneg/stront).

## Architecture

Application is written in Rust which gives easy access to HID libraries, low-level Windows/Linux APIs and cross-platform compatibility.

## Supported platforms/providers

|              | Windows            | Linux                           |
| ------------ | ------------------ | ------------------------------- |
| Time         | :heavy_check_mark: | :heavy_check_mark:              |
| Volume       | :heavy_check_mark: | :heavy_check_mark: (PulseAudio) |
| Input layout | :heavy_check_mark: | :heavy_check_mark: (X11)        |
| Media info   | :heavy_check_mark: | :heavy_check_mark: (D-Bus)      |

MacOS is not supported, as I don't own any Apple devices, feel free to raise PRs.

## How to run it

All files are available in [latest release](https://github.com/zzeneg/qmk-hid-host/releases/tag/latest).

### Configuration

Default configuration is set to [stront](https://github.com/zzeneg/stront). For other keyboards you need to modify `qmk-hid-host.json`.

- `device` section contains information about keyboard. All values are **decimal**, make sure to convert them from hex using a [converter](https://tools.keycdn.com/hex-converter).
  - `productId` - `pid` from your keyboard's `info.json`
  - `usage` and `usagePage` - default values from QMK (`RAW_USAGE_ID` and `RAW_USAGE_PAGE`). No need to modify them unless they were redefined in firmware
- `layouts` - list of supported keyboard layouts in two-letter format (app sends layout's index, not name)
- `reconnectDelay` - delay between reconnecting attempts in milliseconds

### Windows

#### Manual/Debug mode

1. Start `qmk-hid-host.exe`
2. If needed, edit config and restart the app

#### Silent mode

When you verified that the application works with your keyboard, you can use `qmk-hid-host.silent.exe` instead (like add it to Startup). It does not have a console or logs, and can be killed only from Task Manager.

### Linux

1. Update `udev` rules by running script (remember to update `idVendor` and `idProduct` to your values first):

   ```sh
   sudo sh -c 'echo "KERNEL==\"hidraw*\", SUBSYSTEM==\"hidraw\", ATTRS{idVendor}==\"feed\", ATTRS{idProduct}==\"0844\", MODE=\"0666\"" > /etc/udev/rules.d/99-qmkhidhost.rules'
   ```

   [More info](https://get.vial.today/manual/linux-udev.html)

2. Reconnect keyboard
3. Start `qmk-hid-host`, add it to autorun if needed

## Development

1. Install Rust
2. Run `cargo run`
3. If needed, edit `qmk-hid-host.json` in root folder and run again

## Changelog

- 2024-02-06 - add Linux support
- 2024-01-21 - remove run as windows service, add silent version instead
- 2024-01-02 - support RUST_LOG, run as windows service
- 2023-07-30 - rewritten to Rust
