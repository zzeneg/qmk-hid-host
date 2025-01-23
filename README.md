# QMK HID Host

Host component for communicating with QMK keyboards using Raw HID feature.

Requires support on keyboard side, currently is supported by [stront](https://github.com/zzeneg/stront).

## Architecture

Application is written in Rust which gives easy access to HID libraries, low-level Windows/Linux APIs and cross-platform compatibility.

## Supported platforms/providers

|              | Windows            | Linux                           | MacOS              |
| ------------ | ------------------ | ------------------------------- | ------------------ |
| Time         | :heavy_check_mark: | :heavy_check_mark:              | :heavy_check_mark: |
| Volume       | :heavy_check_mark: | :heavy_check_mark: (PulseAudio) | :heavy_check_mark: |
| Input layout | :heavy_check_mark: | :heavy_check_mark: (X11)        | :heavy_check_mark: |
| Media info   | :heavy_check_mark: | :heavy_check_mark: (D-Bus)      |                    |
| Relay        | :heavy_check_mark: | :heavy_check_mark:              | :heavy_check_mark: |

MacOS is partially supported, as I don't own any Apple devices, feel free to raise PRs.

## Relay mode (device-to-device communication) - experimental

This allows for communication between two or more devices. `qmk-hid-host` only receives information from any device and broadcasts it to all devices. The actual sending and receiving should be configured in devices' firmware, but you have to set first byte in the data array - `0xCC` for sending and `0xCD` for receiving.

Example for syncing layers between two devices:

### Data type enum (common between `qmk-hid-host` and all devices)

```c
typedef enum {
    _TIME = 0xAA, // random value that does not conflict with VIA, must match companion app
    _VOLUME,
    _LAYOUT,
    _MEDIA_ARTIST,
    _MEDIA_TITLE,

    _RELAY_FROM_DEVICE = 0xCC,
    _RELAY_TO_DEVICE,
} hid_data_type;
```

### Source device

```c
typedef enum {
    _LAYER = 0,
} relay_data_type;

layer_state_t layer_state_set_user(layer_state_t state) {
    uint8_t data[32];
    memset(data, 0, 32);
    data[0] = _RELAY_FROM_DEVICE;
    data[1] = _LAYER;
    data[2] = get_highest_layer(state);
    raw_hid_send(data, 32);

    return state;
}
```

#### Destination device

```c
typedef enum {
    _LAYER = 0,
} relay_data_type;

void raw_hid_receive_kb(uint8_t *data, uint8_t length) {
    if (data[0] == _RELAY_TO_DEVICE) {
        switch (data[1]) {
            case _LAYER:
                layer_move(data[2]);
                break;
        }
    }
}
```

## How to run it

All files are available in [latest release](https://github.com/zzeneg/qmk-hid-host/releases/tag/latest).

### Configuration

Default configuration is set to [stront](https://github.com/zzeneg/stront). For other keyboards you need to modify the configuration file (`qmk-hid-host.json`).

- `devices` section contains a list of keyboards
  - `productId` - `pid` from your keyboard's `info.json`
  - `name` - keyboard's name (optional, visible only in logs)
  - `usage` and `usagePage` - optional, override only if `RAW_USAGE_ID` and `RAW_USAGE_PAGE` were redefined in firmware
- `layouts` - list of supported keyboard layouts in two-letter format (app sends layout's index, not name)
- `reconnectDelay` - delay between reconnecting attempts in milliseconds (optional, default is 5000)

#### Minimal config

```json
{
  "devices": [
    {
      "productId": "0x0844"
    }
  ],
  "layouts": ["en"]
}
```

Configuration is read from file `qmk-hid-host.json` in the current working directory. If it is not found, then the default configuration is written to this file.
You can specify a different location for the configuration file by using `--config (-c)` command line option. For example:

```
qmk-hid-host -c $HOME/.config/qmk-hid-host/config.json
```

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

### MacOS

1. Download `qmk-hid-host`
2. Modify `qmk-hid-host.json`
3. Add your layouts, for example:

   ```json
   "layouts": ["ABC", "Russian"],
   ```

   if you don't know what layout are installed in you system, run qmk-hid-host with the layouts listed above, change lang and look at terminal output:

   ```
   INFO qmk_hid_host::providers::layout::macos: new layout: 'ABC', layout list: ["ABC", "Russian"]
   INFO qmk_hid_host::providers::layout::macos: new layout: 'Russian', layout list: ["ABC", "Russian"]
   ```

   "new layout:" is what you need

4. start `qmk-hid-host` from directory where your `qmk-hid-host.json` is located
5. If you `qmk-hid-host` stuck at `Waiting for keyboard...` there are two common mistakes:
   1. You're wrong with productId in your config
   2. Close Vial app and try again

## Development

1. Install Rust
2. Run `cargo run`
3. If needed, edit `qmk-hid-host.json` in root folder and run again

## Changelog

- 2024-10-03 - add support for multiple devices, restructure config
- 2024-09-15 - add MacOS support
- 2024-02-06 - add Linux support
- 2024-01-21 - remove run as windows service, add silent version instead
- 2024-01-02 - support RUST_LOG, run as windows service
- 2023-07-30 - rewritten to Rust
