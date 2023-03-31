import { DataType } from '../dataTypes';
import { Provider } from './providerBase';
import config from './../config';

class LayoutProvider extends Provider {
  constructor() {
    super(DataType.Layout, true);
  }

  override onAdapterDataPush = (data: unknown): void => {
    super.onAdapterDataPush(data);
    if (typeof data === 'string' && config.layouts.includes(data)) {
      this.pushDataFn([config.layouts.indexOf(data)]);
    } else {
      this.logUnsupportedAdapterData(data);
    }
  };
}

export default LayoutProvider;
