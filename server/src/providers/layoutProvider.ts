import { DataType } from '../dataTypes';
import { Provider } from './providerBase';

enum Layout {
  en = 0,
  ru,
}

function isLayout(x: string): x is keyof typeof Layout {
  return Object.keys(Layout).includes(x);
}

class LayoutProvider extends Provider {
  constructor() {
    super(DataType.Layout, true);
  }

  override onAdapterDataPush = (data: unknown): void => {
    super.onAdapterDataPush(data);
    if (typeof data === 'string' && isLayout(data)) {
      this.pushDataFn([Layout[data]]);
    } else {
      this.logUnsupportedAdapterData(data);
    }
  };
}

export default LayoutProvider;
