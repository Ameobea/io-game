import { Socket } from 'phoenix-socket';

const wasm = import('./game_engine');
import { getCanvas, clearCanvas } from './renderMethods';
import { initEventHandlers } from './inputWrapper';

const canvas = getCanvas();

export const timer = timeMs => new Promise(f => setTimeout(f, timeMs));

console.log('Initializing WS connection to game server...');
const socket = new Socket('ws://localhost:4000/socket');
export const gameSocket = socket.channel('game:first');

// making socket read proto instead of json

const prevOnConnOpen = socket.onConnOpen;
socket.onConnOpen = function() {
  this.conn.binaryType = 'arraybuffer';
  prevOnConnOpen.apply(this, arguments);
};

const prevOnConnMessage = socket.onConnMessage;

const setRawMessageHandler = (engine: typeof import('./game_engine')) => {
  socket.onConnMessage = function(rawMessage) {
    if (!(rawMessage.data instanceof ArrayBuffer)) {
      return prevOnConnMessage.apply(this, arguments);
    }
    let msg = engine.decode_socket_message(rawMessage.data);
    if (!msg) {
      console.error('Error parsing protobuf message from the server!');
      return;
    }
    let { topic, event, status, _ref: ref } = msg;

    this.log(`receive: ${status || ''} ${topic} ${event} ${(ref && '(' + ref + ')') || ''}`);
    this.channels
      .filter(function(channel) {
        return channel.isMember(topic);
      })
      .forEach(function(channel) {
        return channel.trigger(event, { status }, ref);
      });
    this.stateChangeCallbacks.message.forEach(function(callback) {
      return callback(msg);
    });
  };
};

// end making socket read proto instead of json

wasm
  .then(async engine => {
    (window as any).handle_message = engine.handle_message;
    // Initialize internal game state and provide better error messages when the underlying Rust
    // code panics.
    engine.init();
    initEventHandlers(engine);

    const tick = () => {
      clearCanvas();
      engine.tick();
      requestAnimationFrame(tick);
    };

    tick();

    await timer(500);
    const msg1 = engine.temp_gen_server_message_1();
    console.log(msg1);
    engine.handle_message(msg1);

    await timer(750);
    const msg2 = engine.temp_gen_server_message_2();
    console.log(msg2);
    console.log(engine.handle_message);
    engine.handle_message(msg2);

    ////////

    socket.onError = console.error;
    socket.onConnError = console.error;
    socket.connect();

    const join = gameSocket.join();
    join
      .receive('ok', () => console.log('Connected to lobby!'))
      .receive('error', (reasons: any) => console.error('create failed', reasons));

    (window as any).alex = () => {
      gameSocket.push('move_up');
    };
    gameSocket.on('temp_gen_server_message_1_res', res => {
      console.log(res);
      engine.handle_message(res.msg);
    });
    (window as any).alex2 = () => {
      gameSocket.push('temp_gen_server_message_1');
    };
  })
  .catch(err => console.error(`Error while loading Wasm module: ${err}`));
