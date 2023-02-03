import { DataType } from '../dataTypes';
import { Provider } from './providerBase';

class MediaTitleProvider extends Provider {
  constructor() {
    super(DataType.MediaTitle, true);
  }

  override onAdapterDataPush = (data: unknown): void => {
    super.onAdapterDataPush(data);
    if (typeof data === 'string') {
      // cut extra data which cannot be transferred
      const value = new TextEncoder().encode(data).slice(0, 29);
      console.log(`Shortened value ${new TextDecoder().decode(value)}`);
      this.pushDataFn([value.length, ...value]);
    } else {
      this.logUnsupportedAdapterData(data);
    }
  };
}

export default MediaTitleProvider;
