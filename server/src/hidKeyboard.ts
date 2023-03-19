import { DataType } from './dataTypes';
import { IProvider } from './providers/providerBase';
import * as hid from 'node-hid';
import config from './config.json';

class HidKeyboard {
  private keyboard: hid.HID | undefined;

  onDisconnectFn: (() => void) | undefined;
  onConnectFn: (() => void) | undefined;

  constructor(private readonly providers: IProvider[]) {
    providers.forEach(provider => (provider.pushDataFn = data => this.sendDataToKeyboard(provider, data)));
  }

  isConnected() {
    return !!this.keyboard;
  }

  connect() {
    this.tryConnect();
  }

  private tryConnect(retryDelay = 1, prevRetryDelay = 0) {
    if (this.isConnected()) {
      return;
    }

    const devices = hid.devices();
    const device = devices.find(
      x =>
        x.usage === parseInt(config.device.usage) &&
        x.usagePage === parseInt(config.device.usagePage) &&
        x.productId === parseInt(config.device.productId)
    );
    if (device?.path) {
      console.log(`Found device with path ${device.path}`);
      try {
        this.keyboard = new hid.HID(device.path);
        this.onConnect();
      } catch {
        this.onDisconnect();
      }
    } else {
      const delay = retryDelay + prevRetryDelay;
      console.error(`Keyboard not found, trying to connect in ${delay} seconds...`);
      setTimeout(() => this.tryConnect(delay, retryDelay), delay * 1000);
    }
  }

  private sendDataToKeyboard = (provider: IProvider, data: number[]) => {
    if (!this.keyboard) {
      return;
    }

    // add type as a first byte
    data.unshift(provider.type);
    // add report id (node-hid feature)
    data.unshift(0);
    // 32 is max length of a packet
    const shortData = data.slice(0, 32);
    try {
      console.log('Sending to keyboard', shortData);
      this.keyboard.write(shortData);
    } catch {
      this.onDisconnect();
    }
  };

  private onConnect() {
    console.log(`Connected to keyboard`);
    this.keyboard?.on('data', data => this.onKeyboardData(data));
    this.keyboard?.on('error', err => this.onHidError(err));
    this.providers.forEach(x => x.enable());
    this.onConnectFn?.();
  }

  private onDisconnect() {
    console.log(`Disconnected from keyboard`);
    this.providers.forEach(x => x.disable());
    this.keyboard?.close();
    this.keyboard = undefined;
    this.onDisconnectFn?.();
    this.tryConnect();
  }

  private onHidError(err: unknown) {
    console.error('HID error:', err);
    this.onDisconnect();
  }

  private onKeyboardData(data: number[] | Buffer) {
    console.log('Received data from keyboard', data);
    const [dataType] = data;
    console.log(`Keyboard requested ${DataType[dataType] ?? dataType}`);
    const provider = this.providers.find(x => x.type === dataType);
    if (provider) {
      provider?.pullData();
    } else {
      console.error('Provider not found');
    }
  }
}

export default HidKeyboard;
