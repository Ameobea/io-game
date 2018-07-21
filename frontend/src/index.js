import { Socket } from 'phoenix-socket';

const wasm = import('../game-engine/build/game_engine');

const initEngine = engine => {
  engine.greet('Rendered from Rust via WebAssembly!');

  ////////

  console.log('Initializing WS connection to game server...');
  const socket = new Socket('/socket');
  socket.onError = console.error;
  socket.onConnError = console.error;
  socket.connect();

  const game = socket.channel('game:first');
  const join = game.join();
  console.log(join);
  join
    .receive('ok', function() {
      console.log('Connected to lobby!');
    })
    .receive('error', reasons => console.error('create failed', reasons));
};

wasm.then(initEngine);
