import { readFileSync } from 'fs';
import configFile from './config.json';

let config: typeof configFile | undefined = undefined;

const getConfig = () => {
  try {
    const file = readFileSync('./QMK.HID.Host.Server.json');
    config = JSON.parse(file.toString());
  } catch {
    console.error('config.json is missing, using default configuration.');
  }

  return configFile;
};

export default config || getConfig();
