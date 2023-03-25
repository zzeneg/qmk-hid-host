import { clearInterval } from 'timers';
import { cpuTemperature, currentLoad, graphics, mem } from 'systeminformation';
import { DataType } from '../dataTypes';
import { Provider } from './providerBase';

class SystemInfoProvider extends Provider {
  private interval?: NodeJS.Timer;
  private values?: number[];

  constructor() {
    super(DataType.SystemInfo);
  }

  override enable(): void {
    super.enable();
    this.interval = setInterval(() => {
      this.getValues().then(values => {
        if (JSON.stringify(values) !== JSON.stringify(this.values)) {
          this.values = values;
          this.pushDataFn([...this.values]);
        }
      });
    }, 1000);
  }

  override disable(): void {
    super.disable();
    this.values = undefined;
    clearInterval(this.interval);
  }

  override pullData(): void {
    this.getValues().then(values => this.pushDataFn(values));
  }

  private async getValues(): Promise<number[]> {
    const data = await Promise.all([cpuTemperature(), currentLoad(), graphics(), mem()]);
    const cpuLoad = Math.round(data[1].currentLoad || 0);
    const gpu = data[2].controllers[0];
    const memory = Math.round((data[3].used / data[3].total) * 100);

    return [cpuLoad, data[0].main || 0, gpu?.temperatureGpu || 0, gpu?.utilizationGpu || 0, memory || 0];
  }
}

export default SystemInfoProvider;
