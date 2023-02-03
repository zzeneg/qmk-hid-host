import { clearInterval } from 'timers';
import { DataType } from '../dataTypes';
import { Provider } from './providerBase';

class TimeProvider extends Provider {
  private interval?: NodeJS.Timer;
  private hours?: number;
  private minutes?: number;

  constructor() {
    super(DataType.Time);
  }

  override enable(): void {
    super.enable();
    this.interval = setInterval(() => {
      const now = new Date();
      const currentMinutes = now.getMinutes();
      const currentHours = now.getHours();
      if (this.minutes !== currentMinutes || this.hours !== currentHours) {
        this.hours = currentHours;
        this.minutes = currentMinutes;
        this.pushDataFn([this.hours, this.minutes]);
      }
    }, 1000);
  }

  override disable(): void {
    super.disable();
    this.hours = undefined;
    this.minutes = undefined;
    clearInterval(this.interval);
  }

  override pullData(): void {
    const now = new Date();
    this.pushDataFn([now.getHours(), now.getMinutes()]);
  }
}

export default TimeProvider;
