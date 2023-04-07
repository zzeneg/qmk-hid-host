# QMK HID Host

Host component for communicating with QMK keyboards using Raw HID feature. Requires support on keyboard side, currently is supported by [stront](https://github.com/zzeneg/stront).

## Architecture

Host contains two parts - server and adapters, which communicate between each other using [socket.io](https://socket.io). Server is a cross-platform application written in node.js, and adapters should be OS-specific to be able to use OS functions. Currently only Windows adapter written in C# exists.

## How to run it

1. Download and open [dist](/dist) folder.
2. Open server config file `QMK.HID.Host.Server.json`, make sure `device` values matches your keyboard, update `layouts` if needed.
3. Connect your keyboard.
4. Start `QMK.HID.Host.Server.exe` and `QMK.HID.Host.Adapter.Windows.exe` (order does not matter).
