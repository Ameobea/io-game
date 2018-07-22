import { Socket } from 'phoenix-socket';

const wasm = import('../game-engine/build/game_engine');

wasm.then(engine => {
  const msg = new Uint8Array([0, 1, 2, 3, 4]);
  engine.handle_message(msg);

  ////////

  console.log('Initializing WS connection to game server...');
  const socket = new Socket('ws://localhost:4000/socket');
  socket.onError = console.error;
  socket.onConnError = console.error;
  socket.connect();

  const game = socket.channel('game:first');
  const join = game.join();
  console.log(join);
  join
    .receive('ok', () => console.log('Connected to lobby!'))
    .receive('error', (reasons: any) => console.error('create failed', reasons));
});
