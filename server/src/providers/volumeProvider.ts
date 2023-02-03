import { DataType } from '../dataTypes';
import { Provider } from './providerBase';

class VolumeProvider extends Provider {
  constructor() {
    super(DataType.Volume, true);
  }

  override onAdapterDataPush(data: unknown): void {
    super.onAdapterDataPush(data);
    if (typeof data === 'number') {
      this.pushDataFn([data]);
    } else {
      this.logUnsupportedAdapterData(data);
    }
  }
}

export default VolumeProvider;
