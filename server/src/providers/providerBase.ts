import { DataType } from '../dataTypes';

export interface IProvider {
  name: string;
  type: DataType;
  useAdapter: boolean;
  enable(): void;
  disable(): void;
  pullData(): void;
  pushDataFn: (value: number[]) => void;
  pullAdapterDataFn: () => void;
  onAdapterDataPush(data: unknown): void;
}

export abstract class Provider implements IProvider {
  public name: string;

  protected isEnabled = false;

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  pushDataFn = (_: number[]) => console.error('Keyboard not connected');
  pullAdapterDataFn = () => console.error('Adapter not connected');

  protected constructor(readonly type: DataType, readonly useAdapter: boolean = false) {
    this.name = DataType[this.type];
  }

  enable() {
    console.log(`Provider ${this.name} enabled`);
    this.isEnabled = true;
  }

  disable() {
    console.log(`Provider ${this.name} disabled`);
    this.isEnabled = false;
  }

  pullData() {
    if (this.useAdapter) {
      this.pullAdapterDataFn();
    }
  }

  onAdapterDataPush(data: unknown) {
    console.log(`Provider ${this.name} data received from adapter:`, data);
  }

  logUnsupportedAdapterData(data: unknown) {
    console.error(`Data type ${typeof data} is not supported by Provider ${this.name}`);
  }
}
