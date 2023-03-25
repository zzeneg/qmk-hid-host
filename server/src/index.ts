import { Server } from 'socket.io';
import { IProvider } from './providers/providerBase';
import TimeProvider from './providers/timeProvider';
import VolumeProvider from './providers/volumeProvider';
import LayoutProvider from './providers/layoutProvider';
import MediaArtistProvider from './providers/mediaArtistProvider';
import MediaTitleProvider from './providers/mediaTitleProvider';
import HidKeyboard from './hidKeyboard';
import config from './config';
// import SystemInfoProvider from './providers/systemInfoProvider';

const providers: IProvider[] = [
  new TimeProvider(),
  new VolumeProvider(),
  new LayoutProvider(),
  new MediaArtistProvider(),
  new MediaTitleProvider(),
  // new SystemInfoProvider(), // disabled by default
];

const hidKeyboard = new HidKeyboard(providers);

const io = new Server();

io.on('connection', socket => {
  console.log('Adapter connected');

  providers
    .filter(x => x.useAdapter)
    .forEach(provider => {
      socket.on(provider.name, data => provider.onAdapterDataPush?.(data));
      provider.pullAdapterDataFn = () => socket.emit(provider.name);
    });

  if (hidKeyboard.isConnected()) {
    io.emit('hid-connected');
  }
});

io.listen(config.socket.port);

console.log('SocketIO server started');

hidKeyboard.onConnectFn = () => io.emit('hid-connected');
hidKeyboard.onDisconnectFn = () => io.emit('hid-disconnected');
hidKeyboard.connect();
